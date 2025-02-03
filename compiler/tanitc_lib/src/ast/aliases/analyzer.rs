use super::AliasDef;
use crate::{
    analyzer::{symbol_table::SymbolData, Analyze, Analyzer},
    ast::types::Type,
};

use tanitc_messages::Message;

impl Analyze for AliasDef {
    fn get_type(&self, _analyzer: &mut Analyzer) -> Type {
        unreachable!()
    }

    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if let Ok(_ss) = analyzer.check_identifier_existance(&self.identifier) {
            return Err(Message::multiple_ids(
                self.location,
                &self.identifier.get_string(),
            ));
        }

        analyzer.add_symbol(&self.identifier, analyzer.create_symbol(SymbolData::Type));

        Ok(())
    }
}
