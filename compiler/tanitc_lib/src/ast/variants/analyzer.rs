use std::collections::BTreeMap;

use super::{VariantDef, VariantField};

use tanitc_analyzer::{
    symbol_table::{SymbolData, VariantFieldData},
    Analyze, Analyzer,
};
use tanitc_ident::Ident;
use tanitc_messages::Message;
use tanitc_ty::Type;

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

        analyzer.scope.push(format!("@v.{}", &self.identifier));
        for internal in self.internals.iter_mut() {
            internal.analyze(analyzer)?;
        }
        analyzer.scope.pop();

        let mut components = Vec::<VariantFieldData>::new();
        for field in self.fields.iter() {
            components.push(match field.1 {
                VariantField::Common => VariantFieldData::Common,
                VariantField::StructLike(subfields) => {
                    let mut processed_fields = BTreeMap::<Ident, Type>::new();
                    for field in subfields.iter() {
                        processed_fields.insert(*field.0, field.1.get_type());
                    }

                    VariantFieldData::StructLike(processed_fields)
                }
                VariantField::TupleLike(components) => {
                    let mut processed_components = Vec::<Type>::new();
                    for field in components.iter() {
                        processed_components.push(field.get_type());
                    }
                    VariantFieldData::TupleLike(processed_components)
                }
            });
        }

        analyzer.add_symbol(
            self.identifier,
            analyzer.create_symbol(SymbolData::VariantDef { components }),
        );

        Ok(())
    }
}
