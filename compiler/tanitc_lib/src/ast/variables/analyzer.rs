use super::VariableDef;
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};

use tanitc_messages::Message;
use tanitc_ty::Type;

impl Analyze for VariableDef {
    fn get_type(&self, _analyzer: &mut Analyzer) -> Type {
        self.var_type.get_type()
    }

    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if analyzer.has_symbol(self.identifier) {
            return Err(Message::multiple_ids(self.location, self.identifier));
        }

        analyzer.add_symbol(
            self.identifier,
            analyzer.create_symbol(SymbolData::VariableDef {
                var_type: self.var_type.get_type(),
                is_mutable: self.is_mutable,
                is_initialization: false,
            }),
        );

        Ok(())
    }
}
