use super::{scope::ScopeUnit, symbol::SymbolData, Analyzer};

use tanitc_ast::{
    self, AliasDef, Ast, Block, Branch, BranchKind, CallParam, ControlFlow, ControlFlowKind,
    EnumDef, Expression, ExpressionKind, FunctionDef, ModuleDef, StructDef, TypeSpec, UnionDef,
    Use, Value, ValueKind, VariableDef, VariantDef, VariantField, VisitorMut,
};
use tanitc_ident::Ident;
use tanitc_lexer::{location::Location, token::Lexem};
use tanitc_messages::Message;
use tanitc_ty::Type;

use std::collections::{BTreeMap, HashMap};

impl VisitorMut for Analyzer {
    fn visit_module_def(&mut self, module_def: &mut ModuleDef) -> Result<(), Message> {
        if self.has_symbol(module_def.identifier) {
            return Err(Message::multiple_ids(
                module_def.location,
                module_def.identifier,
            ));
        }

        self.add_symbol(self.create_symbol(module_def.identifier, SymbolData::ModuleDef));

        self.scope.push(ScopeUnit::Module(module_def.identifier));

        if let Some(body) = &mut module_def.body {
            self.visit_block(body)?;
        }

        self.scope.pop();

        Ok(())
    }

    fn visit_struct_def(&mut self, struct_def: &mut StructDef) -> Result<(), Message> {
        if self.has_symbol(struct_def.identifier) {
            return Err(Message::multiple_ids(
                struct_def.location,
                struct_def.identifier,
            ));
        }

        self.scope.push(ScopeUnit::Struct(struct_def.identifier));
        for internal in struct_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        for (field_id, field_ty) in struct_def.fields.iter() {
            self.add_symbol(self.create_symbol(
                *field_id,
                SymbolData::StructField {
                    struct_id: struct_def.identifier,
                    ty: field_ty.get_type(),
                },
            ));
        }

        self.scope.pop();

        self.add_symbol(self.create_symbol(struct_def.identifier, SymbolData::StructDef));

        Ok(())
    }

    fn visit_union_def(&mut self, union_def: &mut UnionDef) -> Result<(), Message> {
        if self.has_symbol(union_def.identifier) {
            return Err(Message::multiple_ids(
                union_def.location,
                union_def.identifier,
            ));
        }

        self.scope.push(ScopeUnit::Union(union_def.identifier));
        for internal in union_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }

        for (field_id, field_ty) in union_def.fields.iter() {
            self.add_symbol(self.create_symbol(
                *field_id,
                SymbolData::UnionField {
                    union_id: union_def.identifier,
                    ty: field_ty.get_type(),
                },
            ));
        }

        self.scope.pop();

        self.add_symbol(self.create_symbol(union_def.identifier, SymbolData::UnionDef));

        Ok(())
    }

    fn visit_variant_def(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        use crate::symbol::VariantFieldKind;

        if self.has_symbol(variant_def.identifier) {
            return Err(Message::multiple_ids(
                variant_def.location,
                variant_def.identifier,
            ));
        }

        self.scope.push(ScopeUnit::Variant(variant_def.identifier));
        for internal in variant_def.internals.iter_mut() {
            internal.accept_mut(self)?;
        }
        self.scope.pop();

        let mut components = HashMap::<Ident, VariantFieldKind>::new();
        for (field_id, field_data) in variant_def.fields.iter() {
            components.insert(
                *field_id,
                match field_data {
                    VariantField::Common => VariantFieldKind::Common,
                    VariantField::StructLike(subfields) => {
                        let mut processed_fields = BTreeMap::<Ident, Type>::new();
                        for field in subfields.iter() {
                            processed_fields.insert(*field.0, field.1.get_type());
                        }

                        VariantFieldKind::StructLike(processed_fields)
                    }
                    VariantField::TupleLike(components) => {
                        let mut processed_components = Vec::<Type>::new();
                        for field in components.iter() {
                            processed_components.push(field.get_type());
                        }
                        VariantFieldKind::TupleLike(processed_components)
                    }
                },
            );
        }

        self.add_symbol(self.create_symbol(variant_def.identifier, SymbolData::VariantDef));

        for (comp_id, comp_data) in components.iter() {
            self.add_symbol(self.create_symbol(
                *comp_id,
                SymbolData::VariantComponent {
                    variant_id: variant_def.identifier,
                    kind: comp_data.clone(),
                },
            ));
        }

        Ok(())
    }

    fn visit_enum_def(&mut self, enum_def: &mut EnumDef) -> Result<(), Message> {
        if self.has_symbol(enum_def.identifier) {
            return Err(Message::multiple_ids(
                enum_def.location,
                enum_def.identifier,
            ));
        }

        let mut counter = 0usize;
        let mut components = Vec::<(Ident, usize)>::new();
        for field in enum_def.fields.iter_mut() {
            if let Some(value) = field.1 {
                counter = *value;
            }

            // mark unmarked enum fields
            *field.1 = Some(counter);

            components.push((*field.0, counter));

            counter += 1;
        }

        self.add_symbol(self.create_symbol(enum_def.identifier, SymbolData::EnumDef));

        self.scope.push(ScopeUnit::Enum(enum_def.identifier));
        for comp in components.iter() {
            self.add_symbol(self.create_symbol(
                comp.0,
                SymbolData::EnumComponent {
                    enum_id: enum_def.identifier,
                    val: comp.1,
                },
            ));
        }
        self.scope.pop();

        Ok(())
    }

    fn visit_func_def(&mut self, func_def: &mut FunctionDef) -> Result<(), Message> {
        if self.has_symbol(func_def.identifier) {
            return Err(Message::multiple_ids(
                func_def.location,
                func_def.identifier,
            ));
        }

        self.scope.push(ScopeUnit::Func(func_def.identifier));

        let mut parameters = Vec::<(Ident, Type)>::new();
        for p in func_def.parameters.iter_mut() {
            if let Ast::VariableDef(node) = p {
                parameters.push((node.identifier, node.var_type.get_type()));
                p.accept_mut(self)?;
            }
        }

        self.scope.pop();

        self.add_symbol(self.create_symbol(
            func_def.identifier,
            SymbolData::FunctionDef {
                parameters,
                return_type: func_def.return_type.get_type(),
                is_declaration: func_def.body.is_some(),
            },
        ));

        self.scope.push(ScopeUnit::Func(func_def.identifier));

        if let Some(body) = &mut func_def.body {
            if let Ast::Block(block) = body.as_mut() {
                for stmt in block.statements.iter_mut() {
                    stmt.accept_mut(self)?;
                }
            }
        }

        self.scope.pop();

        Ok(())
    }

    fn visit_variable_def(&mut self, var_def: &mut VariableDef) -> Result<(), Message> {
        if self.has_symbol(var_def.identifier) {
            return Err(Message::multiple_ids(var_def.location, var_def.identifier));
        }

        self.add_symbol(self.create_symbol(
            var_def.identifier,
            SymbolData::VariableDef {
                var_type: var_def.var_type.get_type(),
                is_mutable: var_def.is_mutable,
                is_initialization: false,
            },
        ));

        Ok(())
    }

    fn visit_alias_def(&mut self, alias_def: &mut AliasDef) -> Result<(), Message> {
        if self.has_symbol(alias_def.identifier) {
            return Err(Message::multiple_ids(
                alias_def.location,
                alias_def.identifier,
            ));
        }

        self.add_symbol(self.create_symbol(alias_def.identifier, SymbolData::Type));

        Ok(())
    }

    fn visit_expression(&mut self, expr: &mut Expression) -> Result<(), Message> {
        let ret = match &mut expr.kind {
            ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } => self.analyze_binary_expr(operation, lhs.as_mut(), rhs.as_mut()),
            ExpressionKind::Unary { node, .. } => self.analyze_unary_expr(node),
            ExpressionKind::Access { lhs, rhs } => self.analyze_access_expr(lhs, rhs),
            ExpressionKind::Conversion { .. } => self.analyze_conversion_expr(),
            ExpressionKind::Term { node, .. } => self.analyze_term_expr(node),
        };

        match ret {
            Ok(Some(processed_node)) => *expr = processed_node,
            Err(mut msg) => {
                msg.location = expr.location;
                return Err(msg);
            }
            _ => {}
        }

        Ok(())
    }

    fn visit_branch(&mut self, branch: &mut Branch) -> Result<(), Message> {
        fn analyze_body(body: &mut Ast, analyzer: &mut Analyzer) -> Result<(), Message> {
            if let Ast::Block(node) = body {
                for stmt in node.statements.iter_mut() {
                    stmt.accept_mut(analyzer)?;
                }
            }

            Ok(())
        }

        fn analyze_condition(condition: &mut Ast, analyzer: &mut Analyzer) -> Result<(), Message> {
            if let Ast::Expression(node) = condition {
                analyzer.visit_expression(node)?;
            }

            Ok(())
        }

        match &mut branch.kind {
            BranchKind::While { body, condition } => {
                let cnt = self.counter();
                self.scope.push(ScopeUnit::Loop(cnt));

                condition.accept_mut(self)?;

                analyze_condition(condition.as_mut(), self)?;
                analyze_body(body.as_mut(), self)?;

                self.scope.pop();

                Ok(())
            }
            BranchKind::Loop { body } => {
                let cnt = self.counter();
                self.scope.push(ScopeUnit::Loop(cnt));

                analyze_body(body.as_mut(), self)?;

                self.scope.pop();

                Ok(())
            }
            BranchKind::If { body, condition } => {
                analyze_condition(condition.as_mut(), self)?;
                analyze_body(body.as_mut(), self)?;

                Ok(())
            }
            BranchKind::Else { body } => {
                analyze_body(body.as_mut(), self)?;

                Ok(())
            }
        }
    }

    fn visit_control_flow(&mut self, cf: &mut ControlFlow) -> Result<(), Message> {
        let is_in_func = {
            let mut flag = false;
            for s in self.scope.iter().rev() {
                if matches!(s, ScopeUnit::Func(_)) {
                    flag = true;
                    break;
                }
            }

            flag
        };

        let is_in_loop = {
            let mut flag = false;
            for s in self.scope.iter().rev() {
                if matches!(s, ScopeUnit::Loop(_)) {
                    flag = true;
                    break;
                }
            }

            flag
        };

        match &mut cf.kind {
            ControlFlowKind::Break { ret } | ControlFlowKind::Return { ret } => {
                if let Some(expr) = ret {
                    expr.accept_mut(self)?;
                }
            }
            _ => {}
        }

        let is_ret = matches!(cf.kind, ControlFlowKind::Return { .. });

        if (!is_ret && !is_in_loop) || (is_ret && !is_in_func) {
            return Err(Message::new(
                cf.location,
                &format!("Unexpected {} statement", cf.kind.to_str()),
            ));
        }

        Ok(())
    }

    fn visit_type_spec(&mut self, _type_spec: &mut TypeSpec) -> Result<(), Message> {
        // if self.has_type(type_spec.get_type()) {
        //     return Err(Message::undefined_type(
        //         type_spec.location,
        //         type_spec..get_type(),
        //     ));
        // }
        Ok(())
    }

    fn visit_use(&mut self, _u: &mut Use) -> Result<(), Message> {
        Ok(())
    }

    fn visit_block(&mut self, block: &mut Block) -> Result<(), Message> {
        if block.is_global {
            self.analyze_global_block(block)?;
        } else {
            self.analyze_local_block(block)?;
        }

        Ok(())
    }

    fn visit_value(&mut self, val: &mut Value) -> Result<(), Message> {
        match &mut val.kind {
            ValueKind::Integer(_) => Ok(()),

            ValueKind::Decimal(_) => Ok(()),

            ValueKind::Text(_) => Ok(()),

            ValueKind::Identifier(id) => {
                if self.has_symbol(*id) {
                    Ok(())
                } else {
                    Err(Message::undefined_id(val.location, *id))
                }
            }

            ValueKind::Call { arguments, .. } => {
                for arg in arguments.iter_mut() {
                    self.analyze_call_param(arg)?;
                }

                if self.check_call_args(val).is_err() {
                    return Err(Message::new(val.location, "Wrong call arguments"));
                }

                Ok(())
            }

            ValueKind::Struct { .. } => self.analyze_struct_value(val),

            ValueKind::Union { .. } => Err(Message::unreachable(
                val.location,
                "ValueKind::Union is not expected here",
            )),

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
                    if comp_type != current_comp_type {
                        return Err(Message::new(
                            val.location,
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
}

impl Analyzer {
    fn check_call_args(&mut self, val: &mut Value) -> Result<(), Message> {
        let (identifier, arguments) = if let ValueKind::Call {
            identifier,
            arguments,
        } = &mut val.kind
        {
            (*identifier, arguments)
        } else {
            return Err(Message::new(
                val.location,
                "Expected call node, but provided another",
            ));
        };

        if identifier.is_built_in() {
            return Ok(());
        }

        if let Some(mut ss) = self.get_first_symbol(identifier) {
            match &mut ss.data {
                SymbolData::FunctionDef { parameters, .. } => {
                    if arguments.len() > parameters.len() {
                        return Err(Message::new(
                            val.location,
                            &format!(
                        "Too many arguments passed in function \"{}\", expected: {}, actually: {}",
                        identifier, parameters.len(), arguments.len()),
                        ));
                    }

                    if arguments.len() < parameters.len() {
                        return Err(Message::new(
                            val.location,
                            &format!(
                        "Too few arguments passed in function \"{}\", expected: {}, actually: {}",
                        identifier, parameters.len(), arguments.len()),
                        ));
                    }

                    let mut positional_skiped = false;
                    for call_arg in arguments.iter_mut() {
                        let arg_clone = call_arg.clone();
                        match arg_clone {
                            CallParam::Notified(arg_id, arg_value) => {
                                positional_skiped = true;

                                // check if such parameter declared in the function
                                let mut param_found = false;
                                for (param_index, (param_name, param_type)) in
                                    parameters.iter().enumerate()
                                {
                                    if *param_name == arg_id {
                                        param_found = true;

                                        let arg_type = self.get_type(&arg_value);
                                        if *param_type != arg_type {
                                            self.error(Message::new(
                                            val.location, &format!(
                                            "Mismatched type for parameter \"{}\". Expected \"{}\", actually: \"{}\"",
                                            param_name, param_type, param_type))
                                        );
                                        }

                                        let modified_param =
                                            CallParam::Positional(param_index, arg_value.clone());
                                        *call_arg = modified_param;
                                    }
                                }
                                if !param_found {
                                    self.error(Message::new(
                                        val.location,
                                        &format!(
                                            "No parameter named \"{}\" in function \"{}\"",
                                            arg_id, identifier
                                        ),
                                    ))
                                }
                            }
                            CallParam::Positional(..) => {
                                if positional_skiped {
                                    return Err(Message::new(
                                        val.location,
                                        "Positional parameters must be passed before notified",
                                    ));
                                }
                            }
                        }
                    }

                    /* Check parameters */
                    for i in parameters.iter() {
                        for j in arguments.iter() {
                            let j_type = self.get_call_param_type(j);
                            if j_type != i.1 {
                                return Err(Message::new(val.location, "Mismatched types"));
                            }
                        }
                    }
                    Ok(())
                }
                _ => Err(Message::new(val.location, "No such function found")),
            }
        } else {
            Err(Message::new(val.location, "No such identifier found"))
        }
    }

    fn analyze_call_param(&self, _cp: &CallParam) -> Result<(), Message> {
        Ok(())
    }

    fn get_type(&self, node: &Ast) -> Type {
        match node {
            Ast::AliasDef(node) => self.get_alias_def_type(node),
            Ast::VariableDef(node) => self.get_var_def_type(node),
            Ast::Expression(node) => self.get_expr_type(node),
            Ast::Value(node) => self.get_value_type(node),
            _ => todo!("GetType"),
        }
    }

    fn get_alias_def_type(&self, alias_def: &AliasDef) -> Type {
        alias_def.value.get_type()
    }

    fn get_var_def_type(&self, var_def: &VariableDef) -> Type {
        var_def.var_type.get_type()
    }

    fn get_expr_type(&self, expr: &Expression) -> Type {
        match &expr.kind {
            ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } => match operation {
                Lexem::Neq | Lexem::Eq | Lexem::Lt | Lexem::Lte | Lexem::Gt | Lexem::Gte => {
                    Type::Bool
                }

                _ => {
                    let lhs_type = self.get_type(lhs);

                    if let Type::Auto = lhs_type {
                        return self.get_type(rhs);
                    }

                    lhs_type
                }
            },
            ExpressionKind::Unary { node, .. } => self.get_type(node),
            ExpressionKind::Conversion { ty, .. } => ty.get_type(),
            ExpressionKind::Access { rhs, .. } => self.get_type(rhs),
            ExpressionKind::Term { ty, .. } => ty.clone(),
        }
    }

    fn get_value_type(&self, val: &Value) -> Type {
        match &val.kind {
            ValueKind::Text(_) => Type::Ref(Box::new(Type::Str)),
            ValueKind::Decimal(_) => Type::F32,
            ValueKind::Integer(_) => Type::I32,
            ValueKind::Identifier(id) => {
                if let Some(ss) = self.get_symbols(id) {
                    for s in ss.iter().rev() {
                        if self.scope.0.starts_with(&s.scope.0) {
                            if let SymbolData::VariableDef { var_type, .. } = &s.data {
                                return var_type.clone();
                            }
                        }
                    }
                }
                Type::new()
            }
            ValueKind::Struct { identifier, .. } => Type::Custom(identifier.to_string()),
            ValueKind::Union { identifier, .. } => Type::Custom(identifier.to_string()),
            ValueKind::Tuple { components } => {
                let mut comp_vec = Vec::<Type>::new();
                for comp in components.iter() {
                    comp_vec.push(self.get_type(comp));
                }
                Type::Tuple(comp_vec)
            }
            ValueKind::Array { components } => {
                let len = components.len();
                if len == 0 {
                    return Type::Array {
                        size: None,
                        value_type: Box::new(Type::Auto),
                    };
                }

                Type::Array {
                    size: None,
                    value_type: Box::new(self.get_type(&components[0])),
                }
            }
            ValueKind::Call { identifier, .. } => {
                if let Some(ss) = self.get_first_symbol(*identifier) {
                    if let SymbolData::FunctionDef { return_type, .. } = &ss.data {
                        return return_type.clone();
                    }
                }
                Type::new()
            }
        }
    }

    fn get_call_param_type(&self, cp: &CallParam) -> Type {
        match cp {
            CallParam::Notified(_, expr) | CallParam::Positional(_, expr) => {
                self.get_type(expr.as_ref())
            }
        }
    }

    #[allow(dead_code)]
    fn convert_expr_node(&mut self, expr_node: &mut Ast) -> Result<(), Message> {
        if let Ast::Expression(node) = expr_node {
            let location = node.location;
            if let ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } = &mut node.kind
            {
                self.convert_expr_node(lhs)?;
                self.convert_expr_node(rhs)?;

                let is_conversion = *operation == Lexem::KwAs;

                let lhs_type = self.get_type(lhs);

                if !is_conversion {
                    let rhs_type = self.get_type(rhs);

                    let func_name_str = format!(
                        "__tanit_compiler__{}_{}_{}",
                        match operation {
                            Lexem::Plus => "add",
                            Lexem::Minus => "sub",
                            Lexem::Star => "mul",
                            Lexem::Slash => "div",
                            Lexem::Percent => "mod",
                            Lexem::LShift => "lshift",
                            Lexem::RShift => "rshift",
                            Lexem::Stick => "or",
                            Lexem::Ampersand => "and",
                            _ => return Err(Message::new(location, "Unexpected operation")),
                        },
                        lhs_type,
                        rhs_type
                    );

                    let func_id = Ident::from(func_name_str);

                    *expr_node = Ast::from(Value {
                        location,
                        kind: ValueKind::Call {
                            identifier: func_id,
                            arguments: vec![
                                CallParam::Positional(0, lhs.clone()),
                                CallParam::Positional(1, rhs.clone()),
                            ],
                        },
                    });
                } else {
                    let rhs_type = if let Ast::Value(Value {
                        kind: ValueKind::Identifier(id),
                        location,
                    }) = rhs.as_ref()
                    {
                        TypeSpec {
                            location: *location,
                            info: tanitc_ast::TypeInfo::default(),
                            ty: Type::from(*id),
                        }
                    } else {
                        return Err(Message::unreachable(location, "expected type-spec"));
                    };
                    *expr_node = Ast::from(Expression {
                        location,
                        kind: ExpressionKind::Binary {
                            operation: Lexem::KwAs,
                            lhs: lhs.clone(),
                            rhs: Box::new(Ast::TypeSpec(rhs_type)),
                        },
                    });
                };
            }
            Ok(())
        } else {
            unreachable!()
        }
    }
}

impl Analyzer {
    fn analyze_unary_expr(&mut self, node: &mut Box<Ast>) -> Result<Option<Expression>, Message> {
        node.accept_mut(self)?;
        Ok(None)
    }

    fn analyze_binary_expr(
        &mut self,
        operation: &Lexem,
        lhs: &mut Ast,
        rhs: &mut Ast,
    ) -> Result<Option<Expression>, Message> {
        rhs.accept_mut(self)?;
        let rhs_type = self.get_type(rhs);

        let does_mutate = *operation == Lexem::Assign
            || *operation == Lexem::SubAssign
            || *operation == Lexem::AddAssign
            || *operation == Lexem::DivAssign
            || *operation == Lexem::ModAssign
            || *operation == Lexem::MulAssign
            || *operation == Lexem::AndAssign
            || *operation == Lexem::OrAssign
            || *operation == Lexem::XorAssign
            || *operation == Lexem::LShiftAssign
            || *operation == Lexem::RShiftAssign;

        if let Ast::VariableDef(node) = lhs {
            if self.has_symbol(node.identifier) {
                return Err(Message::multiple_ids(rhs.location(), node.identifier));
            }

            if Type::Auto == node.var_type.get_type() {
                node.var_type.ty = rhs_type.clone();
            }

            if node.var_type.get_type() != rhs_type {
                self.error(Message::new(
                    rhs.location(),
                    &format!(
                        "Cannot perform operation on objects with different types: {:?} and {:?}",
                        node.var_type.get_type(),
                        rhs_type
                    ),
                ));
            }

            self.add_symbol(self.create_symbol(
                node.identifier,
                SymbolData::VariableDef {
                    var_type: node.var_type.get_type(),
                    is_mutable: node.is_mutable,
                    is_initialization: true,
                },
            ));
        } else if let Ast::Value(node) = lhs {
            match &node.kind {
                ValueKind::Identifier(id) => {
                    if let Some(s) = self.get_first_symbol(*id) {
                        if let SymbolData::VariableDef { is_mutable, .. } = &s.data {
                            if !*is_mutable && does_mutate {
                                self.error(Message::new(
                                    Location::new(),
                                    &format!("Variable \"{}\" is immutable in current scope", id),
                                ));
                            }
                        }
                    }
                }
                ValueKind::Integer(..) | ValueKind::Decimal(..) => {}
                ValueKind::Text(..) => self.error(Message::new(
                    Location::new(),
                    "Cannot perform operation with text in this context",
                )),
                ValueKind::Array { .. } => self.error(Message::new(
                    Location::new(),
                    "Cannot perform operation with array in this context",
                )),
                ValueKind::Tuple { .. } => self.error(Message::new(
                    Location::new(),
                    "Cannot perform operation with tuple in this context",
                )),
                _ => self.error(Message::new(
                    Location::new(),
                    "Cannot perform operation with this object",
                )),
            }
        } else {
            lhs.accept_mut(self)?;
            let lhs_type = self.get_type(lhs);

            if lhs_type != rhs_type {
                self.error(Message::new(
                    rhs.location(),
                    &format!(
                        "Cannot perform operation on objects with different types: {:?} and {:?}",
                        lhs_type, rhs_type
                    ),
                ));
            }
        }

        Ok(None)
    }

    fn analyze_access_expr(
        &mut self,
        lhs: &mut Ast,
        rhs: &mut Ast,
    ) -> Result<Option<Expression>, Message> {
        let mut ids = Vec::<Ident>::new();

        Self::preprocess_access_tree(&mut ids, lhs, rhs)?;

        let scope = self.scope.clone();
        let s = {
            let ss = self.table.access_symbol(&ids, &scope);

            if ss.is_empty() {
                return Err(Message::undefined_id(lhs.location(), *ids.last().unwrap()));
            }

            ss[0].clone()
        };

        let processed_node: Option<Expression> = match s.data {
            SymbolData::EnumComponent { enum_id, val } => Some(Expression {
                location: lhs.location(),
                kind: ExpressionKind::Term {
                    node: Box::new(Ast::Value(Value {
                        location: lhs.location(),
                        kind: ValueKind::Integer(val),
                    })),
                    ty: Type::Custom(enum_id.to_string()),
                },
            }),
            SymbolData::FunctionDef { .. } => None,
            SymbolData::StructDef => {
                let mut value = Box::new(rhs.clone());

                let struct_name = s.id;
                let struct_comps = {
                    let mut struct_comps = HashMap::<Ident, Type>::new();
                    let mut ss = self.table.get_symbols();
                    ss.retain(|s| matches!(s.data, SymbolData::StructField { .. }));
                    for s in ss.iter() {
                        if let SymbolData::StructField { struct_id, ty } = &s.data {
                            if struct_name == *struct_id {
                                struct_comps.insert(s.id, ty.clone());
                            }
                        }
                    }
                    struct_comps
                };

                if let Ast::Value(Value {
                    kind:
                        ValueKind::Struct {
                            components: value_comps,
                            ..
                        },
                    ..
                }) = value.as_mut()
                {
                    if let Err(mut msg) =
                        self.check_struct_components(value_comps, s.id, &struct_comps)
                    {
                        msg.location = rhs.location();
                        return Err(msg);
                    }
                } else {
                    return Err(Message::unreachable(
                        rhs.location(),
                        "expected ValueKind::Struct, actually: {rhs:?}",
                    ));
                }

                let node = Expression {
                    location: lhs.location(),
                    kind: ExpressionKind::Term {
                        node: value,
                        ty: Type::Custom(s.id.to_string()),
                    },
                };

                Some(node)
            }
            SymbolData::UnionDef => {
                let mut value = Box::new(rhs.clone());

                let union_name = s.id;
                let union_comps = {
                    let mut union_comps = HashMap::<Ident, Type>::new();
                    let mut ss = self.table.get_symbols();
                    ss.retain(|s| matches!(s.data, SymbolData::UnionField { .. }));
                    for s in ss.iter() {
                        if let SymbolData::UnionField { union_id, ty } = &s.data {
                            if union_name == *union_id {
                                union_comps.insert(s.id, ty.clone());
                            }
                        }
                    }
                    union_comps
                };

                if let Ast::Value(value) = value.as_mut() {
                    let (union_id, value_comps) = if let ValueKind::Struct {
                        identifier,
                        components,
                    } = &mut value.kind
                    {
                        (*identifier, std::mem::take(components))
                    } else {
                        return Err(Message::unreachable(
                            value.location,
                            "expected ValueKind::Struct",
                        ));
                    };

                    if let Err(mut msg) =
                        self.check_union_components(&value_comps, s.id, &union_comps)
                    {
                        msg.location = rhs.location();
                        return Err(msg);
                    }

                    value.kind = ValueKind::Union {
                        identifier: union_id,
                        components: value_comps,
                    }
                } else {
                    return Err(Message::unreachable(
                        rhs.location(),
                        "expected ValueKind::Struct, actually: {rhs:?}",
                    ));
                }

                let node = Expression {
                    location: lhs.location(),
                    kind: ExpressionKind::Term {
                        node: value,
                        ty: Type::Custom(s.id.to_string()),
                    },
                };

                Some(node)
            }
            _ => todo!("Unaccessible: {:?}", s.data),
        };

        Ok(processed_node)
    }

    fn preprocess_access_tree(
        ids: &mut Vec<Ident>,
        lhs: &mut Ast,
        rhs: &mut Ast,
    ) -> Result<(), Message> {
        let mut loc = Location::new();

        let lhs_id = match lhs {
            Ast::Value(Value {
                location,
                kind: ValueKind::Identifier(id),
            }) => {
                loc = *location;
                *id
            }
            _ => Ident::default(),
        };

        ids.push(lhs_id);

        match rhs {
            Ast::Expression(Expression {
                kind: ExpressionKind::Access { lhs, rhs },
                ..
            }) => {
                Self::preprocess_access_tree(ids, lhs, rhs)?;
            }
            Ast::Value(Value {
                kind: ValueKind::Identifier(rhs_id),
                ..
            }) => {
                ids.push(*rhs_id);
            }
            Ast::Value(Value {
                kind: ValueKind::Call { identifier, .. },
                ..
            }) => {
                ids.push(*identifier);
            }
            Ast::Value(Value {
                kind: ValueKind::Struct { identifier, .. },
                ..
            }) => {
                ids.push(*identifier);
            }
            _ => {
                return Err(Message::unreachable(
                    loc,
                    &format!(
                        "expected ExpressionKind::Access or Value::Identifier, actually: {rhs:?}"
                    ),
                ))
            }
        }

        Ok(())
    }

    fn analyze_conversion_expr(&mut self) -> Result<Option<Expression>, Message> {
        todo!()
    }

    fn analyze_term_expr(&mut self, node: &mut Box<Ast>) -> Result<Option<Expression>, Message> {
        node.accept_mut(self)?;
        Ok(None)
    }
}

impl Analyzer {
    fn analyze_global_block(&mut self, block: &mut Block) -> Result<(), Message> {
        for n in block.statements.iter_mut() {
            if let Err(err) = n.accept_mut(self) {
                self.error(err);
            }
        }

        Ok(())
    }

    fn analyze_local_block(&mut self, block: &mut Block) -> Result<(), Message> {
        let cnt = self.counter();

        self.scope.push(ScopeUnit::Block(cnt));

        for n in block.statements.iter_mut() {
            if let Err(err) = n.accept_mut(self) {
                self.error(err);
            }
        }

        self.scope.pop();

        Ok(())
    }
}

impl Analyzer {
    fn analyze_struct_value(&mut self, value: &mut Value) -> Result<(), Message> {
        let (object_name, value_comps) = if let ValueKind::Struct {
            identifier,
            components,
        } = &mut value.kind
        {
            (identifier, components)
        } else {
            return Err(Message::unreachable(
                value.location,
                "expected ValueKind::Struct",
            ));
        };

        let first = if let Some(symbol) = self.get_first_symbol(*object_name) {
            symbol
        } else {
            return Err(Message::undefined_id(value.location, *object_name));
        };

        if matches!(first.data, SymbolData::StructDef) {
            let mut struct_comps = HashMap::<Ident, Type>::new();
            let mut ss = self.table.get_symbols();

            ss.retain(|s| matches!(s.data, SymbolData::StructField { .. }));
            for s in ss.iter() {
                if let SymbolData::StructField { struct_id, ty } = &s.data {
                    if *struct_id == first.id {
                        struct_comps.insert(s.id, ty.clone());
                    }
                }
            }

            if let Err(mut msg) =
                self.check_struct_components(value_comps, *object_name, &struct_comps)
            {
                msg.location = value.location;
                return Err(msg);
            }
        } else if matches!(first.data, SymbolData::UnionDef) {
            let mut union_comps = HashMap::<Ident, Type>::new();
            let mut ss = self.table.get_symbols();

            ss.retain(|s| matches!(s.data, SymbolData::UnionField { .. }));
            for s in ss.iter() {
                if let SymbolData::UnionField { union_id, ty } = &s.data {
                    if *union_id == first.id {
                        union_comps.insert(s.id, ty.clone());
                    }
                }
            }

            if let Err(mut msg) =
                self.check_union_components(value_comps, *object_name, &union_comps)
            {
                msg.location = value.location;
                return Err(msg);
            }

            value.kind = ValueKind::Union {
                identifier: *object_name,
                components: std::mem::take(value_comps),
            };
        } else {
            return Err(Message::new(
                value.location,
                &format!("Cannot find struct or union named \"{object_name}\" in this scope"),
            ));
        }

        Ok(())
    }

    fn check_struct_components(
        &mut self,
        value_comps: &[(Ident, Ast)],
        struct_name: Ident,
        struct_comps: &HashMap<Ident, Type>,
    ) -> Result<(), Message> {
        if value_comps.len() != struct_comps.len() {
            return Err(Message::new(
                Location::new(),
                &format!(
                    "Struct \"{}\" consists of {} fields, but {} were supplied",
                    struct_name,
                    struct_comps.len(),
                    value_comps.len()
                ),
            ));
        }

        for comp_id in 0..value_comps.len() {
            let value_comp = value_comps.get(comp_id).unwrap();
            let value_comp_type = self.get_type(&value_comp.1);
            let struct_comp_type = struct_comps.get(&value_comp.0).unwrap();

            if value_comp_type != *struct_comp_type {
                return Err(Message::new(
                    Location::new(),
                    &format!(
                        "Field named \"{}\" is {}, but initialized like {}",
                        value_comp.0, struct_comp_type, value_comp_type
                    ),
                ));
            }
        }

        Ok(())
    }

    fn check_union_components(
        &mut self,
        value_comps: &[(Ident, Ast)],
        union_name: Ident,
        union_comps: &HashMap<Ident, Type>,
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
            let union_comp_type = union_comps.get(&value_comp.0).unwrap();

            if value_comp_type != *union_comp_type {
                return Err(Message::new(
                    Location::new(),
                    &format!(
                        "Field named \"{value_comp_name}\" is {union_comp_type}, but initialized like {value_comp_type}"
                    ),
                ));
            }
        }

        Ok(())
    }
}
