use super::{VariantDef, VariantField};
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};

use tanitc_messages::Message;

impl Analyze for VariantField {
    fn analyze(&mut self, _analyzer: &mut Analyzer) -> Result<(), Message> {
        todo!("EnumField analyzer")
    }
}

impl Analyze for VariantDef {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if analyzer.has_symbol(self.identifier) {
            return Err(Message::multiple_ids(self.location, self.identifier));
        }

        analyzer.scope.push(&format!("@v.{}", &self.identifier));
        for internal in self.internals.iter_mut() {
            internal.analyze(analyzer)?;
        }
        analyzer.scope.pop();

        let mut components = Vec::<VariantField>::new();
        for field in self.fields.iter() {
            components.push(field.1.clone());
        }

        analyzer.add_symbol(
            self.identifier,
            analyzer.create_symbol(SymbolData::VariantDef { components }),
        );

        Ok(())
    }
}
