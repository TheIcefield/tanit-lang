use super::TypeSpec;
use crate::analyzer::{Analyze, Analyzer};

use tanitc_messages::Message;

impl Analyze for TypeSpec {
    fn analyze(&mut self, _analyzer: &mut Analyzer) -> Result<(), Message> {
        Err(Message::unreachable(self.location))
    }
}
