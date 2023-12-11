use crate::ast::{calls, Ast, IAst, Stream};
use crate::parser::put_intent;

use std::io::Write;

#[derive(Clone)]
pub struct Tuple {
    pub components: Vec<Box<Ast>>,
}

impl IAst for Tuple {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        if self.components.is_empty() {
            return writeln!(stream, "{}<tuple/>", put_intent(intent));
        }

        writeln!(stream, "{}<tuple>", put_intent(intent))?;

        for comp in self.components.iter() {
            comp.traverse(stream, intent + 1)?;
        }

        writeln!(stream, "{}</tuple>", put_intent(intent))?;

        Ok(())
    }
}

#[derive(Clone)]
pub enum Value {
    Call(calls::Node),
    Tuple(Tuple),
    Identifier(String),
    Text(String),
    Integer(usize),
    Decimal(f64),
}

impl IAst for Value {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match self {
            Self::Call(node) => node.traverse(stream, intent)?,
            Self::Tuple(node) => node.traverse(stream, intent)?,
            Self::Identifier(id) => {
                writeln!(stream, "{}<variable name=\"{}\"/>", put_intent(intent), id)?
            }
            Self::Text(text) => {
                writeln!(stream, "{}<text content=\"{}\"/>", put_intent(intent), text)?
            }
            Self::Integer(val) => writeln!(
                stream,
                "{}<value type=\"int\" value=\"{}\"/>",
                put_intent(intent),
                val
            )?,
            Self::Decimal(val) => writeln!(
                stream,
                "{}<value type=\"float\" value=\"{}\"/>",
                put_intent(intent),
                val
            )?,
        }

        Ok(())
    }
}
