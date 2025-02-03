use super::EnumDef;
use crate::{
    analyzer::{symbol_table::SymbolData, Analyze, Analyzer},
    ast::{identifiers::Identifier, types::Type},
};

use tanitc_messages::Message;

impl Analyze for EnumDef {
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

        let mut counter = 0usize;
        let mut components = Vec::<(Identifier, usize)>::new();
        for field in self.fields.iter_mut() {
            if let Some(value) = field.1 {
                counter = *value;
            }

            // mark unmarked enum fields
            *field.1 = Some(counter);

            components.push((field.0.clone(), counter));

            counter += 1;
        }

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::EnumDef { components }),
        );

        Ok(())
    }
}
