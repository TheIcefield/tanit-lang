use super::Identifier;
use crate::analyzer::{Analyze, Analyzer};

use tanitc_messages::Message;

impl Analyze for Identifier {
    fn analyze(&mut self, _analyzer: &mut Analyzer) -> Result<(), Message> {
        Ok(())
    }
}
