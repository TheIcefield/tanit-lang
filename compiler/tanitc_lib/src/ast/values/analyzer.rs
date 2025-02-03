use super::{CallParam, Value, ValueType};
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};
use crate::ast::{types::Type, Ast};

use tanitc_messages::Message;

impl Value {
    fn check_call_args(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        let (identifier, arguments) = if let ValueType::Call {
            identifier,
            arguments,
        } = &mut self.value
        {
            (identifier, arguments)
        } else {
            return Err(Message::new(
                self.location,
                "Expected call node, but provided another",
            ));
        };

        if Analyzer::is_built_in_identifier(identifier) {
            return Ok(());
        }

        if let Ok(mut ss) = analyzer.check_identifier_existance(identifier) {
            match &mut ss.data {
                SymbolData::FunctionDef { args, .. } => {
                    if arguments.len() > args.len() {
                        return Err(Message::new(self.location, &format!(
                            "Too many arguments passed in function \"{}\", expected: {}, actually: {}",
                            identifier, args.len(), arguments.len())));
                    }

                    if arguments.len() < args.len() {
                        return Err(Message::new(self.location, &format!(
                            "Too few arguments passed in function \"{}\", expected: {}, actually: {}",
                            identifier, args.len(), arguments.len())));
                    }

                    let mut positional_skiped = false;
                    for call_arg in arguments.iter_mut() {
                        let arg_clone = call_arg.clone();
                        match arg_clone {
                            CallParam::Notified(param_id, param_value) => {
                                positional_skiped = true;

                                // check if such parameter declared in the function
                                let mut param_found = false;
                                let param_id = param_id.get_string();
                                for (param_index, (func_param_name, func_param_type)) in
                                    args.iter().enumerate()
                                {
                                    if *func_param_name == param_id {
                                        param_found = true;

                                        let param_type = param_value.get_type(analyzer);
                                        if *func_param_type != param_type {
                                            analyzer.error(Message::new(
                                                self.location, &format!(
                                                "Mismatched type for parameter \"{}\". Expected \"{}\", actually: \"{}\"",
                                                func_param_name, func_param_type, param_type))
                                            );
                                        }

                                        let modified_param =
                                            CallParam::Positional(param_index, param_value.clone());
                                        *call_arg = modified_param;
                                    }
                                }
                                if !param_found {
                                    analyzer.error(Message::new(
                                        self.location,
                                        &format!(
                                            "No parameter named \"{}\" in function \"{}\"",
                                            param_id, identifier
                                        ),
                                    ))
                                }
                            }
                            CallParam::Positional(..) => {
                                if positional_skiped {
                                    return Err(Message::new(
                                        self.location,
                                        "Positional parameters must be passed before notified",
                                    ));
                                }
                            }
                        }
                    }

                    /* Check parameters */
                    for i in args.iter() {
                        for j in arguments.iter() {
                            let j_type = j.get_type(analyzer);
                            if j_type != i.1 {
                                return Err(Message::new(self.location, "Mismatched types"));
                            }
                        }
                    }
                    Ok(())
                }
                _ => Err(Message::new(self.location, "No such function found")),
            }
        } else {
            Err(Message::new(self.location, "No such identifier found"))
        }
    }
}

impl Analyze for CallParam {
    fn get_type(&self, analyzer: &mut Analyzer) -> Type {
        match self {
            Self::Notified(_, expr) | Self::Positional(_, expr) => expr.get_type(analyzer),
        }
    }

    fn analyze(&mut self, _analyzer: &mut Analyzer) -> Result<(), Message> {
        Ok(())
    }
}

impl Analyze for Value {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        match &mut self.value {
            ValueType::Integer(_) => Ok(()),

            ValueType::Decimal(_) => Ok(()),

            ValueType::Text(_) => Ok(()),

            ValueType::Identifier(id) => {
                let _ = analyzer.check_identifier_existance(id)?;
                Ok(())
            }

            ValueType::Call { arguments, .. } => {
                for arg in arguments.iter_mut() {
                    arg.analyze(analyzer)?;
                }

                if self.check_call_args(analyzer).is_err() {
                    return Err(Message::new(self.location, "Wrong call arguments"));
                }

                Ok(())
            }

            ValueType::Struct {
                identifier,
                components: value_comps,
            } => {
                let ss = analyzer.check_identifier_existance(identifier)?;

                if let SymbolData::StructDef {
                    components: struct_comps,
                } = &ss.data
                {
                    if value_comps.len() != struct_comps.len() {
                        return Err(Message::new(
                            self.location,
                            &format!(
                                "Struct \"{}\" consists of {} fields, but {} were supplied",
                                identifier,
                                struct_comps.len(),
                                value_comps.len()
                            ),
                        ));
                    }

                    for comp_id in 0..value_comps.len() {
                        let value_comp = value_comps.get(comp_id).unwrap();
                        let value_comp_type = value_comp.1.get_type(analyzer);
                        let struct_comp_type = struct_comps.get(comp_id).unwrap();

                        if value_comp_type != *struct_comp_type {
                            return Err(Message::new(
                                self.location,
                                &format!(
                                    "Field named \"{}\" is {}, but initialized like {}",
                                    value_comp.0, struct_comp_type, value_comp_type
                                ),
                            ));
                        }
                    }
                } else {
                    return Err(Message::new(
                        self.location,
                        &format!("Cannot find struct named \"{}\" in this scope", identifier),
                    ));
                }

                Ok(())
            }

            ValueType::Tuple { components } => {
                for comp in components.iter_mut() {
                    comp.analyze(analyzer)?;
                }

                Ok(())
            }

            ValueType::Array { components } => {
                if components.is_empty() {
                    return Ok(());
                }

                let comp_type = components[0].get_type(analyzer);

                for comp in components.iter().enumerate() {
                    let current_comp_type = comp.1.get_type(analyzer);
                    if comp_type != current_comp_type {
                        return Err(Message::new(
                            self.location,
                            &format!(
                                "Array type is declared like {}, but {}{} element has type {}",
                                comp_type,
                                comp.0 + 1,
                                match comp.0 % 10 {
                                    0 => "st",
                                    1 => "nd",
                                    2 => "rd",
                                    _ => "th",
                                },
                                current_comp_type
                            ),
                        ));
                    }
                }

                Ok(())
            }
        }
    }

    fn get_type(&self, analyzer: &mut Analyzer) -> Type {
        match &self.value {
            ValueType::Text(_) => Type::Ref {
                is_mut: false,
                ref_to: Box::new(Type::Str),
            },
            ValueType::Decimal(_) => Type::F32,
            ValueType::Integer(_) => Type::I32,
            ValueType::Identifier(id) => {
                if let Some(ss) = analyzer.get_symbols(id) {
                    for s in ss.iter().rev() {
                        if analyzer.scope.0.starts_with(&s.scope.0) {
                            if let SymbolData::VariableDef { var_type, .. } = &s.data {
                                return var_type.clone();
                            }
                        }
                    }
                }
                analyzer.error(Message::new(
                    self.location,
                    &format!("No variable found with name \"{}\"", id),
                ));
                Type::new()
            }
            ValueType::Struct { identifier, .. } => Type::Custom(identifier.to_string()),
            ValueType::Tuple { components } => {
                let mut comp_vec = Vec::<Type>::new();
                for comp in components.iter() {
                    comp_vec.push(comp.get_type(analyzer));
                }
                Type::Tuple {
                    components: comp_vec,
                }
            }
            ValueType::Array { components } => {
                let len = components.len();
                if len == 0 {
                    return Type::Array {
                        size: None,
                        value_type: Box::new(Type::Auto),
                    };
                }

                Type::Array {
                    size: Some(Box::new(Ast::from(Self {
                        location: self.location,
                        value: ValueType::Integer(len),
                    }))),
                    value_type: Box::new(components[0].get_type(analyzer)),
                }
            }
            ValueType::Call { identifier, .. } => {
                if let Ok(ss) = analyzer.check_identifier_existance(identifier) {
                    if let SymbolData::FunctionDef { return_type, .. } = &ss.data {
                        return return_type.clone();
                    }
                }
                Type::new()
            }
        }
    }
}
