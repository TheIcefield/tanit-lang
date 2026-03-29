use tanitc_lexer::location::Location;
use tanitc_name::NameSpec;

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub location: Location,
    pub name: NameSpec,
}
