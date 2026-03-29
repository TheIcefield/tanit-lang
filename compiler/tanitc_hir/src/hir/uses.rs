use tanitc_lexer::location::Location;
use tanitc_name::NameSpec;

use crate::hir::Hir;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Use {
    pub location: Location,
    pub name: NameSpec,
}

impl From<Use> for Hir {
    fn from(value: Use) -> Self {
        Self::Use(value)
    }
}
