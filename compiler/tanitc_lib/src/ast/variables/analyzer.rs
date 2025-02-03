use super::VariableDef;
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};
use crate::ast::types::Type;

use tanitc_messages::Message;

impl Analyze for VariableDef {
    fn get_type(&self, _analyzer: &mut Analyzer) -> Type {
        self.var_type.clone()
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
