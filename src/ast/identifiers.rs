use crate::ast::{expressions::Expression, values::Value, Ast, IAst, Stream};
use crate::codegen::CodeGenStream;
use crate::lexer::Lexem;

use std::io::Write;
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Identifier {
    Common(String),
    Complex(Vec<String>),
}

impl FromStr for Identifier {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Common(s.to_string()))
    }
}

impl Identifier {
    pub fn new() -> Self {
        Self::Common(String::new())
    }

    pub fn from_token(tkn: &Lexem) -> Result<Self, &'static str> {
        if let Lexem::Identifier(id) = tkn {
            return Self::from_str(id);
        }

        Err("token must be an identifier")
    }

    // Converts expression chained by '::' to Identifier::Complex
    pub fn from_expr(expr: &Expression) -> Result<Self, &'static str> {
        let mut res = Self::Complex(Vec::new());

        if let Expression::Unary { .. } = expr {
            return Err("Expected binary expression");
        }

        if let Expression::Binary {
            operation,
            lhs,
            rhs,
        } = expr
        {
            if Lexem::Dcolon != *operation {
                return Err("Expected expression chained by \"::\"");
            }

            if let Ast::Expression { node } = rhs.as_ref() {
                // recursively parse expression tail
                res.append(Self::from_expr(node)?)?;
            }

            let rhs_id = if let Ast::Value { node } = rhs.as_ref() {
                if let Value::Identifier(id) = node {
                    Some(id.clone())
                } else {
                    return Err("Rhs is not an identifier");
                }
            } else {
                None
            };

            let lhs_id = if let Ast::Value { node } = lhs.as_ref() {
                if let Value::Identifier(id) = node {
                    id.clone()
                } else {
                    return Err("Lhs is not an identifier");
                }
            } else {
                return Err("Lhs in not a value");
            };

            if let Self::Complex(..) = res {
                let current = res;
                res = lhs_id;
                res.append(current)?;

                if let Some(id) = rhs_id {
                    res.append(id)?;
                }
            }
        }

        Ok(res)
    }
}

// Private methods
impl Identifier {
    fn append(&mut self, other: Self) -> Result<(), &'static str> {
        // if self is common, convert to complex
        if let Self::Common(id) = self {
            *self = Self::Complex(vec![id.to_string()]);
        }

        // if self is complex, merge them
        if let Self::Complex(pieces) = self {
            // if other is common, just add
            if let Self::Common(id) = &other {
                pieces.push(id.clone());
            }

            // if other is complex, concat them
            if let Self::Complex(ids) = &other {
                pieces.append(&mut ids.clone());
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::Common(id) = self {
            write!(f, "{}", id)?;
        }

        if let Self::Complex(ids) = self {
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

impl IAst for Identifier {
    fn analyze(&mut self, _analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        Ok(())
    }

    fn traverse(&self, stream: &mut Stream, _intent: usize) -> std::io::Result<()> {
        write!(stream, "{}", self)
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        if let Self::Common(id) = self {
            write!(stream, "{}", id)?;
        }

        if let Self::Complex(ids) = self {
            write!(stream, "{}", ids[0])?;
            for id in ids.iter().skip(1) {
                write!(stream, "__{}", id)?;
            }
        }

        Ok(())
    }
}
