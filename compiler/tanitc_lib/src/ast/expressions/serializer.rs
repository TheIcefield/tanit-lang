use super::{Expression, ExpressionType};
use tanitc_serializer::{Serialize, XmlWriter};

impl Serialize for Expression {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("operation")?;

        match &self.expr {
            ExpressionType::Unary { operation, node } => {
                writer.put_param("style", "unary")?;
                writer.put_param("operation", operation)?;
                node.serialize(writer)?;
            }
            ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } => {
                writer.put_param("style", "binary")?;
                writer.put_param("operation", operation)?;

                lhs.serialize(writer)?;
                rhs.serialize(writer)?;
            }
            ExpressionType::Conversion { lhs, ty } => {
                writer.put_param("style", "conversion")?;

                ty.serialize(writer)?;
                lhs.serialize(writer)?;
            }
        }

        writer.end_tag()?;

        Ok(())
    }
}
