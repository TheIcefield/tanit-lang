use super::VariableDef;
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for VariableDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("variable-definition")?;

        self.identifier.serialize(writer)?;
        writer.put_param("is-global", self.is_global)?;
        writer.put_param("is-mutable", self.is_mutable)?;

        self.var_type.serialize(writer)?;

        writer.end_tag()?;

        Ok(())
    }
}
