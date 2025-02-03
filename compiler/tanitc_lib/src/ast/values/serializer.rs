use super::{CallParam, Value, ValueType};
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for CallParam {
    fn serialize(&self, _writer: &mut XmlWriter) -> std::io::Result<()> {
        todo!("serialize CallParam")
    }
}

impl Serialize for Value {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        match &self.value {
            ValueType::Call {
                identifier,
                arguments,
            } => {
                writer.begin_tag("call-statement")?;

                identifier.serialize(writer)?;

                writer.begin_tag("parameters")?;
                for arg in arguments.iter() {
                    writer.begin_tag("parameter")?;
                    match arg {
                        CallParam::Notified(id, expr) => {
                            id.serialize(writer)?;
                            expr.serialize(writer)?;
                        }
                        CallParam::Positional(index, expr) => {
                            writer.put_param("index", index)?;
                            expr.serialize(writer)?;
                        }
                    }
                    writer.end_tag()?; //parameter
                }
                writer.end_tag()?; // parameters
                writer.end_tag()?; // call-statement
            }
            ValueType::Struct {
                identifier,
                components,
            } => {
                writer.begin_tag("struct-initialization")?;

                identifier.serialize(writer)?;

                for (comp_id, comp_type) in components.iter() {
                    writer.begin_tag("field")?;

                    comp_id.serialize(writer)?;
                    comp_type.serialize(writer)?;

                    writer.end_tag()?;
                }

                writer.end_tag()?;
            }
            ValueType::Tuple { components } => {
                writer.begin_tag("tuple-initialization")?;

                for component in components.iter() {
                    component.serialize(writer)?;
                }

                writer.end_tag()?;
            }
            ValueType::Array { components } => {
                writer.begin_tag("array-initialization")?;

                for component in components.iter() {
                    component.serialize(writer)?;
                }

                writer.end_tag()?;
            }
            ValueType::Identifier(id) => {
                writer.begin_tag("variable")?;
                id.serialize(writer)?;
                writer.end_tag()?;
            }
            ValueType::Text(value) => {
                writer.begin_tag("literal")?;
                writer.put_param("style", "text")?;
                writer.put_param("value", value)?;
                writer.end_tag()?;
            }
            ValueType::Integer(value) => {
                writer.begin_tag("literal")?;
                writer.put_param("style", "integer-number")?;
                writer.put_param("value", value)?;
                writer.end_tag()?;
            }
            ValueType::Decimal(value) => {
                writer.begin_tag("literal")?;
                writer.put_param("style", "decimal-number")?;
                writer.put_param("value", value)?;
                writer.end_tag()?;
            }
        }

        Ok(())
    }
}
