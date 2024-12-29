use super::{VariantDef, VariantField};
use crate::analyzer::{Analyze, Analyzer, SymbolData};
use crate::messages::Message;

impl Analyze for VariantField {
    fn analyze(&mut self, _analyzer: &mut Analyzer) -> Result<(), Message> {
        todo!("EnumField analyzer")
    }
}

impl Analyze for VariantDef {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if let Ok(_ss) = analyzer.check_identifier_existance(&self.identifier) {
            return Err(Message::multiple_ids(
                self.location,
                &self.identifier.get_string(),
            ));
        }

        analyzer.scope.push(&format!("@e.{}", &self.identifier));
        for internal in self.internals.iter_mut() {
            internal.analyze(analyzer)?;
        }
        analyzer.scope.pop();

        let mut components = Vec::<VariantField>::new();
        for field in self.fields.iter() {
            components.push(field.1.clone());
        }

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::VariantDef { components }),
        );

        Ok(())
    }
}
