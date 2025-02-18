use super::EnumDef;
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};

use tanitc_ident::Ident;
use tanitc_messages::Message;
use tanitc_ty::Type;

impl Analyze for EnumDef {
    fn get_type(&self, _analyzer: &mut Analyzer) -> Type {
        unreachable!()
    }

    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if analyzer.has_symbol(self.identifier) {
            return Err(Message::multiple_ids(self.location, self.identifier));
        }

        let mut counter = 0usize;
        let mut components = Vec::<(Ident, usize)>::new();
        for field in self.fields.iter_mut() {
            if let Some(value) = field.1 {
                counter = *value;
            }

            // mark unmarked enum fields
            *field.1 = Some(counter);

            components.push((*field.0, counter));

            counter += 1;
        }

        analyzer.add_symbol(
            self.identifier,
            analyzer.create_symbol(SymbolData::EnumDef { components }),
        );

        Ok(())
    }
}
