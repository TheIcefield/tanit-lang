use crate::parser::put_intent;
use crate::ast::{IAst, Stream};

use std::io::Write;

#[derive(Clone)]
pub enum ValueType {
    Identifier(String),
    Text(String),
    Integer(usize),
    Decimal(f64),
}

impl IAst for ValueType {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {       
        match self {
            ValueType::Identifier(id) => {
                writeln!(stream, "{}<variable name=\"{}\"/>", put_intent(intent), id)?
            },
            ValueType::Text(text) => {
                writeln!(stream, "{}<text content=\"{}\"/>", put_intent(intent), text)?
            },
            ValueType::Integer(val) => {
                writeln!(stream, "{}<value type=\"int\" value=\"{}\"/>", put_intent(intent), val)?
            },
            ValueType::Decimal(val) => {
                writeln!(stream, "{}<value type=\"float\" value=\"{}\"/>", put_intent(intent), val)?
            }
        }
        
        Ok(())
    }
}

