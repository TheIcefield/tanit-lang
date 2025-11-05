use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

use crate::hir::Hir;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum UseIdentifier {
    #[default]
    UseSelf,
    UseCrate,
    UseSuper,
    UseAll,
    UseId(Ident),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Use {
    pub location: Location,
    pub identifiers: Vec<UseIdentifier>,
}

impl From<Use> for Hir {
    fn from(value: Use) -> Self {
        Self::Use(value)
    }
}
