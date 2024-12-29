use super::VariableDef;
use crate::analyzer::{Analyze, Analyzer, SymbolData};
use crate::ast::types::Type;
use crate::messages::Message;

impl Analyze for VariableDef {
    fn get_type(&self, _analyzer: &mut Analyzer) -> Type {
        unreachable!()
    }

    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if analyzer
            .check_identifier_existance(&self.identifier)
            .is_ok()
        {
            return Err(Message::multiple_ids(
                self.location,
                &self.identifier.get_string(),
            ));
        }

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::VariableDef {
                var_type: self.var_type.clone(),
                is_mutable: self.is_mutable,
                is_initialization: false,
            }),
        );

        Ok(())
    }
}
