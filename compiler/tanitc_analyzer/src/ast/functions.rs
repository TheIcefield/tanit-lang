use tanitc_ast::{
    ast::{
        expressions::Expression,
        functions::{FunctionDef, FunctionParam},
        types::TypeSpec,
        values::{Value, ValueKind},
        Ast,
    },
    visitor::VisitorMut,
};
use tanitc_ident::Ident;
use tanitc_messages::Message;
use tanitc_symbol_table::entry::{Entry, FuncDefData, SymbolKind};
use tanitc_ty::Type;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_func_def(
        &mut self,
        func_def: &mut FunctionDef,
        is_method: bool,
    ) -> Result<(), Message> {
        if self.has_symbol(func_def.name.id) {
            return Err(Message::multiple_ids(&func_def.location, func_def.name.id));
        }

        func_def.name.prefix = self.table.get_id();

        let mut scope_info = self.table.get_scope_info();
        scope_info.safety = func_def.attributes.safety;
        scope_info.is_in_func = true;

        self.table.enter_scope(scope_info);

        self.analyze_func_def_params(func_def, is_method)?;
        let parameters = self.get_func_def_params(func_def)?;

        if let Some(body) = &mut func_def.body {
            self.visit_block(body)?;
        }

        self.table.exit_scope();

        self.analyze_return_type(&mut func_def.return_type)?;

        self.add_symbol(Entry {
            name: func_def.name.id,
            is_static: false,
            kind: SymbolKind::from(FuncDefData {
                parameters,
                name: func_def.name,
                return_type: func_def.return_type.get_type(),
                is_virtual: false,
                is_inline: false,
                no_return: func_def.return_type.get_type() == Type::unit(),
                safety: func_def.attributes.safety,
            }),
        });

        Ok(())
    }

    pub fn access_func_def(
        &mut self,
        func_data: &FuncDefData,
        rhs: &mut Ast,
    ) -> Result<Option<Expression>, Message> {
        let Ast::Value(Value {
            location,
            kind:
                ValueKind::Call {
                    arguments: call_args,
                    ..
                },
        }) = rhs
        else {
            todo!("Unexpected rhs: {rhs:#?}");
        };

        self.analyze_call(func_data.name.id, func_data, call_args, location)?;

        Ok(None)
    }
}
impl Analyzer {
    fn analyze_func_def_params(
        &mut self,
        func_def: &mut FunctionDef,
        is_method: bool,
    ) -> Result<(), Message> {
        for (index, param) in func_def.parameters.iter_mut().enumerate() {
            match param {
                FunctionParam::Common(var_def) => {
                    if let Err(err) = self.visit_variable_def(var_def) {
                        self.error(err);
                    }
                }
                FunctionParam::SelfPtr(_)
                | FunctionParam::SelfRef(_)
                | FunctionParam::SelfVal(_) => {
                    if !is_method {
                        self.error(Message::from_string(
                            &func_def.location,
                            format!(
                                "In definition of function \"{}\": \"self\" parameter is allowed only in associated functions",
                                func_def.name.id),
                        ));
                    }

                    if index > 0 {
                        self.error(Message::from_string(
                            &func_def.location,
                            format!(
                                "In definition of function \"{}\": Unexpected \"self\" parameter. Must be the first parameter of the associated function",
                                func_def.name.id
                            )));
                    }
                }
            }
        }

        Ok(())
    }

    fn get_func_def_params(&self, func_def: &FunctionDef) -> Result<Vec<(Ident, Type)>, Message> {
        let mut parameters = Vec::<(Ident, Type)>::with_capacity(func_def.parameters.len());

        for param in func_def.parameters.iter() {
            if let FunctionParam::Common(var_def) = param {
                parameters.push((var_def.identifier, var_def.var_type.get_type()));
            }
        }

        Ok(parameters)
    }

    fn analyze_return_type(&mut self, return_type: &mut TypeSpec) -> Result<(), Message> {
        let Some(type_info) = self.table.lookup_type(&return_type.ty) else {
            return Err(Message::undefined_type(
                &return_type.location,
                &return_type.ty.as_str(),
            ));
        };

        return_type.ty = type_info.ty;

        Ok(())
    }
}
