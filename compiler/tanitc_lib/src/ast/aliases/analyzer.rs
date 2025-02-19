use super::AliasDef;

use tanitc_analyzer::{symbol_table::SymbolData, Analyze, Analyzer};
use tanitc_messages::Message;
use tanitc_ty::Type;

impl Analyze for AliasDef {
    fn get_type(&self, _analyzer: &mut Analyzer) -> Type {
        unreachable!()
    }

    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if analyzer.has_symbol(self.identifier) {
            return Err(Message::multiple_ids(self.location, self.identifier));
        }

        analyzer.add_symbol(self.identifier, analyzer.create_symbol(SymbolData::Type));

        Ok(())
    }
}
