use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub location: Location,
    pub id: Ident,
}
