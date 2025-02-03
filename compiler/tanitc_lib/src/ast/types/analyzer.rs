use super::Type;
use crate::analyzer::{Analyze, Analyzer};

use tanitc_messages::Message;

impl Analyze for Type {
    fn analyze(&mut self, _analyzer: &mut Analyzer) -> Result<(), Message> {
        unreachable!();
    }
}
