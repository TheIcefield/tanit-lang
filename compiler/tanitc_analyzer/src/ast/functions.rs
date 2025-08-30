use tanitc_ast::{
    ast::{
        expressions::Expression,
        functions::{FunctionDef, FunctionParam},
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
        if self.has_symbol(func_def.identifier) {
            return Err(Message::multiple_ids(
                func_def.location,
                func_def.identifier,
            ));
        }

        let mut scope_info = self.table.get_scope_info();
        scope_info.safety = func_def.attributes.safety;
        scope_info.is_in_func = true;

        self.table.enter_scope(scope_info);

        let parameters = self.analyze_func_def_params(func_def, is_method)?;

        if let Some(body) = &mut func_def.body {
            self.visit_block(body)?;
        }

        self.table.exit_scope();

        self.add_symbol(Entry {
            name: func_def.identifier,
            is_static: false,
            kind: SymbolKind::from(FuncDefData {
                parameters,
                return_type: func_def.return_type.get_type(),
                is_virtual: false,
                is_inline: false,
                no_return: func_def.return_type.get_type() == Type::unit(),
                safety: func_def.attributes.safety,
            }),
        });

        Ok(())
    }

    fn analyze_func_def_params(
        &mut self,
        func_def: &mut FunctionDef,
        is_method: bool,
    ) -> Result<Vec<(Ident, Type)>, Message> {
        let mut parameters = Vec::<(Ident, Type)>::with_capacity(func_def.parameters.len());

        for (index, param) in func_def.parameters.iter_mut().enumerate() {
            match param {
                FunctionParam::Common(var_def) => {
                    if let Err(err) = self.visit_variable_def(var_def) {
                        self.error(err);
                    } else {
                        parameters.push((var_def.identifier, var_def.var_type.get_type()));
                    }
                }
                FunctionParam::SelfPtr(_)
                | FunctionParam::SelfRef(_)
                | FunctionParam::SelfVal(_) => {
                    if !is_method {
                        self.error(Message::from_string(
                            func_def.location,
                            format!(
                                "In definition of function \"{}\": \"self\" parameter is allowed only in associated functions",
                                func_def.identifier),
                        ));
                    }

                    if index > 0 {
                        self.error(Message::from_string(
                            func_def.location,
                            format!(
                                "In definition of function \"{}\": Unexpected \"self\" parameter. Must be the first parameter of the associated function",
                                func_def.identifier
                            )));
                    }
                }
            }
        }
        Ok(parameters)
    }

    pub fn access_func_def(
        &mut self,
        func_name: Ident,
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

        self.analyze_call(func_name, func_data, call_args, *location)?;

        Ok(None)
    }
}
