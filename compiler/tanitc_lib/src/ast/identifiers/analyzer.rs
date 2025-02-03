use super::Identifier;
use crate::analyzer::{Analyze, Analyzer};
use crate::messages::Message;

impl Analyze for Identifier {
    fn analyze(&mut self, _analyzer: &mut Analyzer) -> Result<(), Message> {
        Ok(())
    }
}
