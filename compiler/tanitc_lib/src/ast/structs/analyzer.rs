use super::StructDef;
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};
use crate::ast::types::Type;

use tanitc_messages::Message;

impl Analyze for StructDef {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if analyzer.has_symbol(self.identifier) {
            return Err(Message::multiple_ids(self.location, self.identifier));
        }

        analyzer.scope.push(&format!("@s.{}", &self.identifier));
        for internal in self.internals.iter_mut() {
            internal.analyze(analyzer)?;
        }

        let mut components = Vec::<Type>::new();
        for field in self.fields.iter() {
            components.push(field.1.clone());
        }

        analyzer.scope.pop();

        analyzer.add_symbol(
            self.identifier,
            analyzer.create_symbol(SymbolData::StructDef { components }),
        );

        Ok(())
    }
}
