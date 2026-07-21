//! Identifier and scoped-name representations.
//!
//! A single identifier is wrapped in [`NameCtx`]. A potentially-qualified
//! (scoped) name such as `foo::bar::baz` is represented by [`NameSpecCtx`],
//! which stores a sequence of [`NameSpecSegmentCtx`] entries separated by `::`
//! tokens.

use std::fmt::Display;

use tanitc_ident::Ident;
use tanitc_lexer::token::Token;

/// A single identifier (e.g. `point`, `main`, `Inner`).
///
/// Wraps the token that holds the identifier lexeme and provides convenience
/// methods to extract the interned [`Ident`].
#[derive(Debug, Clone)]
pub struct NameCtx {
    pub name_tkn: Token, // identifier
}

/// One segment of a qualified name, together with its optional trailing `::`.
///
/// For `foo::bar::baz` this would be three entries:
/// `[(foo, Some(::)), (bar, Some(::)), (baz, None)]`.
pub type NameSpecSegmentCtx = (
    Token,         // identifier
    Option<Token>, // '::'?
);

/// A potentially-qualified (scoped) name specification.
///
/// Represents names like `MyType`, `math::Vector2`, or `std::io::stdout`.
/// Each segment stores the identifier token and an optional `::` separator
/// token that follows it.
#[derive(Debug, Clone)]
pub struct NameSpecCtx {
    pub names: Vec<NameSpecSegmentCtx>,
}

impl Display for NameCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name_tkn.identifier())
    }
}

impl NameCtx {
    /// Returns the interned identifier for this name.
    pub fn identifier(&self) -> Ident {
        self.name_tkn.identifier()
    }
}
