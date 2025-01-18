use super::ModuleDef;
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};
use crate::ast::identifiers::IdentifierType;
use crate::messages::Message;

impl Analyze for ModuleDef {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        let identifier = match &self.identifier.identifier {
            IdentifierType::Common(id) => id.clone(),
            IdentifierType::Complex(..) => {
                return Err(Message::new(
                    self.location,
                    &format!(
                        "Expected common identifier, actually complex: {}",
                        self.identifier
                    ),
                ));
            }
        };

        if analyzer
            .check_identifier_existance(&self.identifier)
            .is_ok()
        {
            return Err(Message::new(
                self.location,
                &format!("Identifier \"{}\" defined multiple times", &self.identifier),
            ));
        }

        analyzer.scope.push(&identifier);

        let mut internal_analyzer = Analyzer::new();
        self.body.analyze(&mut internal_analyzer)?;

        analyzer.scope.pop();

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::ModuleDef {
                full_name: vec![identifier],
            }),
        );

        Ok(())
    }
}
