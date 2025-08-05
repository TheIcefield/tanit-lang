use std::{cmp::Ordering, collections::BTreeMap};

use tanitc_ast::ast::{
    values::{CallArg, CallArgKind, Value, ValueKind},
    Ast,
};
use tanitc_attributes::Mutability;
use tanitc_ident::Ident;
use tanitc_lexer::location::Location;
use tanitc_messages::Message;
use tanitc_symbol_table::{
    entry::{FuncDefData, StructFieldData, SymbolKind},
    type_info::{MemberInfo, TypeInfo},
};
use tanitc_ty::{ArraySize, Type};

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_value(&mut self, value: &mut Value) -> Result<(), Message> {
        match &mut value.kind {
            ValueKind::Integer(_) => Ok(()),

            ValueKind::Decimal(_) => Ok(()),

            ValueKind::Text(_) => Ok(()),

            ValueKind::Identifier(id) => {
                if self.has_symbol(*id) {
                    Ok(())
                } else {
                    Err(Message::undefined_id(value.location, *id))
                }
            }

            ValueKind::Call {
                identifier: func_name,
                arguments: call_args,
            } => {
                let Some(func_entry) = self.table.lookup(*func_name) else {
                    return Err(Message::undefined_id(value.location, *func_name));
                };

                let SymbolKind::FuncDef(func_data) = func_entry.kind.clone() else {
                    return Err(Message::undefined_func(value.location, *func_name));
                };

                self.analyze_call(func_entry.name, &func_data, call_args, value.location)?;

                Ok(())
            }

            ValueKind::Struct { .. } => self.analyze_struct_value(value),

            ValueKind::Tuple { components } => {
                for comp in components.iter_mut() {
                    comp.accept_mut(self)?;
                }

                Ok(())
            }

            ValueKind::Array { components } => {
                if components.is_empty() {
                    return Ok(());
                }

                let comp_type = self.get_type(&components[0]);

                for comp in components.iter().enumerate() {
                    let current_comp_type = self.get_type(comp.1);
                    if comp_type.ty != current_comp_type.ty {
                        let comp_index = comp.0 + 1;
                        let suffix = get_ordinal_number_suffix(comp.0);
                        return Err(Message::from_string(
                            value.location,
                            format!(
                                "Array type is declared like {}, but {comp_index}{suffix} element has type {}",
                                comp_type.ty, current_comp_type.ty
                            ),
                        ));
                    }
                }

                Ok(())
            }
        }
    }

    pub fn analyze_call(
        &mut self,
        func_name: Ident,
        func_data: &FuncDefData,
        call_args: &mut [CallArg],
        location: Location,
    ) -> Result<(), Message> {
        if func_name.is_built_in() {
            return Ok(());
        }

        let params = &func_data.parameters;

        self.check_arg_count(func_name, call_args, params, location)?;

        let mut positional_skipped = false;
        for call_arg in call_args.iter_mut() {
            if let Err(err) = self.analyze_arg(func_name, params, call_arg, &mut positional_skipped)
            {
                self.error(err);
            }
        }

        Ok(())
    }

    pub fn analyze_struct_value(&mut self, value: &mut Value) -> Result<(), Message> {
        let ValueKind::Struct {
            identifier: object_name,
            components: value_comps,
        } = &mut value.kind
        else {
            return Err(Message::unreachable(
                value.location,
                format!("Expected ValueKind::Struct, actually: {:?}", value.kind),
            ));
        };

        let mut object = if let Some(entry) = self.table.lookup(*object_name) {
            entry.clone()
        } else {
            return Err(Message::undefined_id(value.location, *object_name));
        };

        if let SymbolKind::AliasDef(alias_data) = &object.kind {
            let ty = &alias_data.ty;
            match ty {
                Type::Custom(id) => {
                    let alias_to_id = Ident::from(id.clone());

                    if let Some(entry) = self.table.lookup(alias_to_id) {
                        object = entry.clone();
                    } else {
                        return Err(Message::undefined_id(value.location, alias_to_id));
                    };
                }
                ty if ty.is_common() => {
                    return Err(Message {
                        location: value.location,
                        text: format!("Common type \"{ty}\" does not have any fields"),
                    })
                }
                _ => {
                    todo!("Unexpected type: {ty}");
                }
            }
        }

        match &object.kind {
            SymbolKind::StructDef(struct_data) => {
                if let Err(mut msg) =
                    self.check_struct_components(value_comps, *object_name, &struct_data.fields)
                {
                    msg.location = value.location;
                    return Err(msg);
                }
            }
            SymbolKind::UnionDef(union_data) => {
                if let Err(mut msg) =
                    self.check_union_components(value_comps, *object_name, &union_data.fields)
                {
                    msg.location = value.location;
                    return Err(msg);
                }

                value.kind = ValueKind::Struct {
                    identifier: *object_name,
                    components: std::mem::take(value_comps),
                };
            }
            _ => {
                return Err(Message::new(
                    value.location,
                    &format!("Cannot find struct or union named \"{object_name}\" in this scope"),
                ));
            }
        }

        Ok(())
    }

    pub fn check_struct_components(
        &mut self,
        value_comps: &[(Ident, Ast)],
        struct_name: Ident,
        struct_comps: &BTreeMap<Ident, StructFieldData>,
    ) -> Result<(), Message> {
        if value_comps.len() != struct_comps.len() {
            return Err(Message::new(
                Location::new(),
                &format!(
                    "Struct \"{struct_name}\" consists of {} fields, but {} were supplied",
                    struct_comps.len(),
                    value_comps.len()
                ),
            ));
        }

        for comp_id in 0..value_comps.len() {
            let value_comp = value_comps.get(comp_id).unwrap();
            let value_comp_name = value_comp.0;
            let value_comp_type = self.get_type(&value_comp.1);
            let struct_comp_type = &struct_comps.get(&value_comp_name).unwrap().ty;

            let mut alias_to = self.find_alias_value(struct_comp_type);

            if value_comp_type.ty == *struct_comp_type {
                alias_to = None;
            }

            if alias_to.is_none() && value_comp_type.ty != *struct_comp_type {
                return Err(Message {
                    location: value_comp.1.location(),
                    text: format!(
                        "Struct field named \"{value_comp_name}\" is {struct_comp_type}, but initialized like {value_comp_type}",
                    ),
                });
            } else if alias_to
                .as_ref()
                .is_some_and(|ty| value_comp_type.ty != *ty)
            {
                return Err(Message {
                    location: value_comp.1.location(),
                    text: format!(
                        "Struct field named \"{value_comp_name}\" is {struct_comp_type} (aka: {}), but initialized like {value_comp_type}",
                        alias_to.unwrap()
                    ),
                });
            }
        }

        Ok(())
    }

    pub fn check_union_components(
        &mut self,
        value_comps: &[(Ident, Ast)],
        union_name: Ident,
        union_comps: &BTreeMap<Ident, StructFieldData>,
    ) -> Result<(), Message> {
        let union_comp_size = union_comps.len();
        let initialized_comp_size = value_comps.len();

        if union_comp_size == 0 && initialized_comp_size > 0 {
            return Err(Message::new(
                Location::new(),
                &format!(
                    "Union \"{union_name}\" has no fields, but were supplied {initialized_comp_size} fields",
                ),
            ));
        }

        if union_comp_size > 0 && initialized_comp_size > 1 {
            return Err(Message::new(
                Location::new(),
                &format!(
                    "Only one union field must be initialized, but {initialized_comp_size} were initialized",
                ),
            ));
        }

        for comp_id in 0..initialized_comp_size {
            let value_comp = value_comps.get(comp_id).unwrap();
            let value_comp_name = value_comp.0;
            let value_comp_type = self.get_type(&value_comp.1);
            let union_comp_data = union_comps.get(&value_comp.0).unwrap();
            let union_comp_type = &union_comp_data.ty;

            let mut alias_to = self.find_alias_value(union_comp_type);

            if value_comp_type.ty == *union_comp_type {
                alias_to = None;
            }

            if alias_to.is_none() && value_comp_type.ty != *union_comp_type {
                return Err(Message::new(
                    Location::new(),
                    &format!(
                        "Union field named \"{value_comp_name}\" is {union_comp_type}, but initialized like {value_comp_type}"
                    ),
                ));
            } else if alias_to
                .as_ref()
                .is_some_and(|ty| value_comp_type.ty != *ty)
            {
                return Err(Message::new(
                        Location::new(),
                        &format!(
                            "Union field named \"{value_comp_name}\" is {union_comp_type} (aka: {}), but initialized like {value_comp_type}",
                            alias_to.unwrap()
                        ),
                    ));
            }
        }

        Ok(())
    }

    pub fn check_tuple_components(
        &mut self,
        value_comps: &[Ast],
        tuple_name: Option<Ident>,
        tuple_comps: &BTreeMap<usize, StructFieldData>,
    ) -> Result<(), Message> {
        if value_comps.len() != tuple_comps.len() {
            return Err(Message::new(
                Location::new(),
                &format!(
                    "Tuple {} consists of {} fields, but {} were supplied",
                    if let Some(tuple_name) = tuple_name {
                        tuple_name.to_string()
                    } else {
                        "".to_string()
                    },
                    tuple_comps.len(),
                    value_comps.len()
                ),
            ));
        }

        for comp_id in 0..value_comps.len() {
            let value_comp = value_comps.get(comp_id).unwrap();
            let value_comp_type = self.get_type(value_comp);
            let tuple_comp_type = &tuple_comps.get(&comp_id).unwrap().ty;

            let mut alias_to = self.find_alias_value(tuple_comp_type);

            if value_comp_type.ty == *tuple_comp_type {
                alias_to = None;
            }

            if alias_to.is_none() && value_comp_type.ty != *tuple_comp_type {
                return Err(Message {
                    location: value_comp.location(),
                    text: format!(
                        "Tuple component with index \"{comp_id}\" is {tuple_comp_type}, but initialized like {value_comp_type}",
                    ),
                });
            } else if alias_to
                .as_ref()
                .is_some_and(|ty| value_comp_type.ty != *ty)
            {
                return Err(Message {
                    location: value_comp.location(),
                    text: format!(
                        "Tuple component with index \"{comp_id}\" is {tuple_comp_type} (aka: {}), but initialized like {value_comp_type}",
                        alias_to.unwrap()
                    ),
                });
            }
        }

        Ok(())
    }

    pub fn get_value_type(&self, val: &Value) -> TypeInfo {
        match &val.kind {
            ValueKind::Text(_) => TypeInfo {
                ty: Type::Ref {
                    ref_to: Box::new(Type::Str),
                    mutability: Mutability::Immutable,
                },
                mutability: Mutability::Immutable,
                ..Default::default()
            },
            ValueKind::Decimal(_) => TypeInfo {
                ty: Type::F32,
                mutability: Mutability::Mutable,
                ..Default::default()
            },
            ValueKind::Integer(_) => TypeInfo {
                ty: Type::I32,
                mutability: Mutability::Mutable,
                ..Default::default()
            },
            ValueKind::Identifier(id) => {
                let Some(entry) = self.table.lookup(*id) else {
                    return TypeInfo {
                        ty: Type::new(),
                        mutability: Mutability::Mutable,
                        ..Default::default()
                    };
                };

                let SymbolKind::VarDef(data) = &entry.kind else {
                    return TypeInfo {
                        ty: Type::new(),
                        mutability: Mutability::Mutable,
                        ..Default::default()
                    };
                };

                let Some(mut type_info) = self.table.lookup_type(&data.var_type) else {
                    return TypeInfo {
                        ty: data.var_type.clone(),
                        mutability: data.mutability,
                        ..Default::default()
                    };
                };

                type_info.mutability = data.mutability;
                type_info
            }
            ValueKind::Struct { identifier, .. } => {
                let ty = Type::Custom(identifier.to_string());
                let Some(mut type_info) = self.table.lookup_type(&ty) else {
                    return TypeInfo {
                        ty,
                        mutability: Mutability::Mutable,
                        ..Default::default()
                    };
                };
                type_info.mutability = Mutability::Mutable;
                type_info
            }
            ValueKind::Tuple { components } => {
                let mut comp_vec = Vec::<Type>::new();
                for comp in components.iter() {
                    comp_vec.push(self.get_type(comp).ty);
                }
                TypeInfo {
                    ty: Type::Tuple(comp_vec.clone()),
                    mutability: Mutability::Mutable,
                    members: {
                        let mut members = BTreeMap::<Ident, MemberInfo>::new();
                        for (comp_idx, comp_type) in comp_vec.iter().enumerate() {
                            members.insert(
                                Ident::from(format!("{comp_idx}")),
                                MemberInfo {
                                    is_public: true,
                                    ty: comp_type.clone(),
                                },
                            );
                        }
                        members
                    },
                    ..Default::default()
                }
            }
            ValueKind::Array { components } => {
                let len = components.len();
                if len == 0 {
                    return TypeInfo {
                        ty: Type::Array {
                            size: ArraySize::Unknown,
                            value_type: Box::new(Type::Auto),
                        },
                        mutability: Mutability::Mutable,
                        members: BTreeMap::new(),
                        ..Default::default()
                    };
                }

                TypeInfo {
                    ty: Type::Array {
                        size: ArraySize::Fixed(len),
                        value_type: Box::new(self.get_type(&components[0]).ty),
                    },
                    mutability: Mutability::Mutable,
                    ..Default::default()
                }
            }
            ValueKind::Call { identifier, .. } => {
                let Some(ss) = self.table.lookup(*identifier) else {
                    return TypeInfo {
                        ty: Type::new(),
                        mutability: Mutability::Mutable,
                        ..Default::default()
                    };
                };

                let SymbolKind::FuncDef(data) = &ss.kind else {
                    return TypeInfo {
                        ty: Type::new(),
                        mutability: Mutability::Mutable,
                        ..Default::default()
                    };
                };

                let Some(mut type_info) = self.table.lookup_type(&data.return_type) else {
                    return TypeInfo {
                        ty: data.return_type.clone(),
                        mutability: Mutability::Mutable,
                        ..Default::default()
                    };
                };

                type_info.mutability = Mutability::Mutable;
                type_info
            }
        }
    }

    fn check_arg_count(
        &self,
        func_name: Ident,
        arguments: &[CallArg],
        parameters: &[(Ident, Type)],
        location: Location,
    ) -> Result<(), Message> {
        let expected_len = parameters.len();
        let actual_len = arguments.len();

        let many_or_few = match actual_len.cmp(&expected_len) {
            Ordering::Greater => "many",
            Ordering::Less => "few",
            Ordering::Equal => "",
        };

        if actual_len != expected_len {
            return Err(Message::new(
                location,
                &format!(
                    "Too {many_or_few} arguments passed in function \"{func_name}\", expected: {expected_len}, actually: {actual_len}",
                ),
            ));
        }

        Ok(())
    }

    fn analyze_positional_arg(
        &self,
        func_name: Ident,
        func_params: &[(Ident, Type)],
        arg_idx: usize,
        arg_value: &Ast,
        positional_skipped: &mut bool,
    ) -> Result<usize, Message> {
        if *positional_skipped {
            return Err(Message::from_string(
                arg_value.location(),
                format!("In function \"{func_name}\" call: positional parameter \"{arg_idx}\" must be passed before notified",
            )));
        }

        let func_param_type = &func_params[arg_idx].1;
        let expr_type = self.get_type(arg_value);

        if expr_type.ty != *func_param_type {
            return Err(Message::from_string(
                arg_value.location(),
                format!("Mismatched types. In function \"{func_name}\" call: positional parameter \"{arg_idx}\" has type \"{expr_type}\" but expected \"{func_param_type}\""),
            ));
        }

        Ok(arg_idx)
    }

    fn analyze_notified_arg(
        &self,
        func_name: Ident,
        func_params: &[(Ident, Type)],
        arg_id: Ident,
        arg_value: &Ast,
        positional_skipped: &mut bool,
    ) -> Result<usize, Message> {
        *positional_skipped = true;
        let location = arg_value.location();

        // check if such parameter declared in the function
        for (param_index, (param_name, param_type)) in func_params.iter().enumerate() {
            if *param_name == arg_id {
                let arg_type = self.get_type(arg_value);
                if *param_type != arg_type.ty {
                    return Err(Message::from_string(
                        location,
                        format!("Mismatched types. In function \"{func_name}\" call: notified parameter \"{arg_id}\" has type \"{arg_type}\" but expected \"{param_type}\"", ),
                    ));
                }

                return Ok(param_index);
            }
        }

        Err(Message::from_string(
            location,
            format!("No parameter named \"{arg_id}\" in function \"{func_name}\""),
        ))
    }

    fn analyze_arg(
        &mut self,
        func_name: Ident,
        func_params: &[(Ident, Type)],
        arg: &mut CallArg,
        positional_skipped: &mut bool,
    ) -> Result<(), Message> {
        let res = match &arg.kind {
            CallArgKind::Notified(arg_id, arg_value) => self.analyze_notified_arg(
                func_name,
                func_params,
                *arg_id,
                arg_value,
                positional_skipped,
            ),
            CallArgKind::Positional(arg_idx, arg_value) => self.analyze_positional_arg(
                func_name,
                func_params,
                *arg_idx,
                arg_value,
                positional_skipped,
            ),
        };

        match res {
            Ok(arg_position) => {
                let arg_value = match &mut arg.kind {
                    CallArgKind::Notified(_, arg_value) => std::mem::take(arg_value),
                    CallArgKind::Positional(_, arg_value) => std::mem::take(arg_value),
                };

                arg.kind = CallArgKind::Positional(arg_position, arg_value);
            }
            Err(err) => self.error(err),
        }

        Ok(())
    }
}

fn get_ordinal_number_suffix(num: usize) -> &'static str {
    match num % 10 {
        0 => "st",
        1 => "nd",
        2 => "rd",
        _ => "th",
    }
}
