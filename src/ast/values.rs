use crate::parser::put_intent;
use crate::ast::{IAst, Stream};

use std::io::Write;

#[derive(Clone)]
pub enum ValueType {
    Identifier(String),
    Text(String),
    Integer(usize),
    Decimal(f64),
    Struct,
    Alias,
}

impl IAst for ValueType {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        write!(stream, "{}<value val=\"", put_intent(intent))?;
        
        match self {
            ValueType::Identifier(id) => write!(stream, "{}", id)?,
            ValueType::Text(text) => write!(stream, "{}", text)?,
            ValueType::Integer(val) => write!(stream, "{}", val)?,
            ValueType::Decimal(val) => write!(stream, "{}", val)?,
            ValueType::Struct => write!(stream, "struct")?,
            ValueType::Alias => write!(stream, "alias")?,
        }
        
        writeln!(stream, "\">\n{}</value>", put_intent(intent))?;
        
        Ok(())
    }
}

