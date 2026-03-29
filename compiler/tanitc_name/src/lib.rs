use std::fmt::Display;

use tanitc_ident::Ident;
use tanitc_lexer::location::Location;

#[derive(Debug, Clone, PartialEq)]
pub enum NamePathSegment {
    SelfNameSpace,
    SuperNameSpace,
    CrateNameSpace,
    Id(Ident),
    AllIdents,
}

impl From<Ident> for NamePathSegment {
    fn from(value: Ident) -> Self {
        Self::Id(value)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct NameSpec {
    pub location: Location,
    pub path: Vec<NamePathSegment>,
}

impl NameSpec {
    pub fn get_id(&self) -> Option<Ident> {
        let seg = self.path.last()?;

        let NamePathSegment::Id(id) = seg else {
            return None;
        };

        Some(*id)
    }
}

impl Display for NamePathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelfNameSpace => write!(f, "Self"),
            Self::SuperNameSpace => write!(f, "Super"),
            Self::CrateNameSpace => write!(f, "crate"),
            Self::Id(id) => write!(f, "{id}"),
            Self::AllIdents => write!(f, "*"),
        }
    }
}

impl Display for NameSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            return Err(std::fmt::Error);
        }

        write!(f, "{}", self.path[0])?;
        for segment in self.path.iter().skip(1) {
            write!(f, "::{segment}")?;
        }

        Ok(())
    }
}
