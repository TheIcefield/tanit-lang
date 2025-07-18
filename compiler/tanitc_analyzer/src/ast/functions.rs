use tanitc_ast::{FunctionDef, FunctionParam, VisitorMut};
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
            body.accept_mut(self)?;
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
                        self.error(Message::new(
                            func_def.location,
                            "\"self\" parameter is allowed only in associated functions",
                        ));
                    }

                    if index > 0 {
                        self.error(Message::new(
                            func_def.location,
                            "Unexpected \"self\" parameter. Must be the first parameter of the associated function"));
                    }
                }
            }
        }
        Ok(parameters)
    }
}
