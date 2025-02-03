use super::Scope;
use crate::analyzer::{Analyze, Analyzer};

use tanitc_messages::Message;

impl Analyze for Scope {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        let cnt = analyzer.counter();

        analyzer.scope.push(&format!("@s.{}", cnt));
        for n in self.statements.iter_mut() {
            if let Err(err) = n.analyze(analyzer) {
                analyzer.error(err);
            }
        }
        analyzer.scope.pop();

        Ok(())
    }
}
