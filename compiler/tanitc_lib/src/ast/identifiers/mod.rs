use crate::ast::{expressions::Expression, values::ValueType, Ast};

use tanitc_lexer::{
    location::Location,
    token::{Lexem, Token},
};
use tanitc_messages::Message;

pub mod analyzer;
pub mod codegen;
pub mod serializer;

use super::expressions::ExpressionType;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IdentifierType {
    Common(String),
    Complex(Vec<String>),
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier {
    pub location: Location,
    pub identifier: IdentifierType,
}

impl std::str::FromStr for Identifier {
    type Err = Message;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            location: Location::new(),
            identifier: IdentifierType::Common(s.to_string()),
        })
    }
}

impl Identifier {
    pub fn new() -> Self {
        Self {
            location: Location::new(),
            identifier: IdentifierType::Common(String::new()),
        }
    }

    pub fn from_token(tkn: &Token) -> Result<Self, Message> {
        use std::str::FromStr;
        if let Lexem::Identifier(id) = &tkn.lexem {
            return Self::from_str(id);
        }

        Err(Message::new(tkn.location, "token must be an identifier"))
    }

    // Converts expression chained by '::' to Identifier::Complex
    pub fn from_expr(expr: &Expression) -> Result<Self, Message> {
        let mut res = Self {
            location: expr.location,
            identifier: IdentifierType::Complex(Vec::new()),
        };

        if let ExpressionType::Unary { .. } = expr.expr {
            return Err(Message::new(expr.location, "Expected binary expression"));
        }

        if let ExpressionType::Binary {
            operation,
            lhs,
            rhs,
        } = &expr.expr
        {
            if Lexem::Dcolon != *operation {
                return Err(Message::new(
                    expr.location,
                    "Expected expression chained by \"::\"",
                ));
            }

            if let Ast::Expression(node) = rhs.as_ref() {
                // recursively parse expression tail
                res.append(Self::from_expr(node)?);
            }

            let rhs_id = if let Ast::Value(node) = rhs.as_ref() {
                if let ValueType::Identifier(id) = &node.value {
                    Some(id.clone())
                } else {
                    return Err(Message::new(expr.location, "Rhs is not an identifier"));
                }
            } else {
                None
            };

            let lhs_id = if let Ast::Value(node) = lhs.as_ref() {
                if let ValueType::Identifier(id) = &node.value {
                    id.clone()
                } else {
                    return Err(Message::new(expr.location, "Lhs is not an identifier"));
                }
            } else {
                return Err(Message::new(expr.location, "Rhs is not a value"));
            };

            if let IdentifierType::Complex(..) = res.identifier {
                let current = res;
                res = lhs_id;
                res.append(current);

                if let Some(id) = rhs_id {
                    res.append(id);
                }
            }
        }

        Ok(res)
    }

    pub fn get_string(&self) -> String {
        if let IdentifierType::Common(id) = &self.identifier {
            return id.clone();
        }

        if let IdentifierType::Complex(ids) = &self.identifier {
            let mut res = ids[0].clone();
            for id in ids.iter().skip(1) {
                res.push_str("::");
                res.push_str(id);
            }
            return res;
        }

        unreachable!()
    }
}

// Private methods
impl Identifier {
    fn append(&mut self, other: Self) {
        // if self is common, convert to complex
        if let IdentifierType::Common(id) = &self.identifier {
            self.identifier = IdentifierType::Complex(vec![id.to_string()]);
        }

        // if self is complex, merge them
        if let IdentifierType::Complex(ref mut pieces) = &mut self.identifier {
            // if other is common, just add
            if let IdentifierType::Common(id) = &other.identifier {
                pieces.push(id.clone());
            }

            // if other is complex, concat them
            if let IdentifierType::Complex(ids) = &other.identifier {
                pieces.append(&mut ids.clone());
            }
        }
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let IdentifierType::Common(id) = &self.identifier {
            write!(f, "{}", id)?;
        }

        if let IdentifierType::Complex(ids) = &self.identifier {
            write!(f, "{}", ids[0])?;
            for id in ids.iter().skip(1) {
                write!(f, "::{}", id)?;
            }
        }

        Ok(())
    }
}

impl Default for Identifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
