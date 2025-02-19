use super::ModuleDef;

use tanitc_analyzer::{symbol_table::SymbolData, Analyze, Analyzer};
use tanitc_messages::Message;

impl Analyze for ModuleDef {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if analyzer.has_symbol(self.identifier) {
            return Err(Message::multiple_ids(self.location, self.identifier));
        }

        analyzer.scope.push(self.identifier.to_string());

        if let Some(body) = &mut self.body {
            body.analyze(analyzer)?;
        }

        analyzer.scope.pop();

        analyzer.add_symbol(
            self.identifier,
            analyzer.create_symbol(SymbolData::ModuleDef {
                full_name: vec![self.identifier],
            }),
        );

        Ok(())
    }
}
