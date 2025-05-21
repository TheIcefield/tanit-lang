use tanitc_ast::{FunctionDef, ImplDef, VisitorMut};
use tanitc_ident::Ident;
use tanitc_messages::Message;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_impl_def(&mut self, impl_def: &mut ImplDef) -> Result<(), Message> {
        let Some(_) = self.table.lookup_mut(impl_def.identifier) else {
            return Err(Message::undefined_id(
                impl_def.location,
                impl_def.identifier,
            ));
        };

        self.analyze_impl_methods(&mut impl_def.methods)?;
        self.rename_impl_methods(&mut impl_def.methods, impl_def.identifier)?;

        Ok(())
    }

    fn analyze_impl_methods(&mut self, methods: &mut [FunctionDef]) -> Result<(), Message> {
        for method in methods.iter_mut() {
            match self.visit_func_def(method) {
                Ok(_) => {}
                Err(err) => self.error(err),
            }
        }

        Ok(())
    }

    fn rename_impl_methods(
        &mut self,
        methods: &mut [FunctionDef],
        prefix: Ident,
    ) -> Result<(), Message> {
        for method in methods.iter_mut() {
            method.identifier = Ident::from(format!("{prefix}__{}", method.identifier));
        }

        Ok(())
    }
}
