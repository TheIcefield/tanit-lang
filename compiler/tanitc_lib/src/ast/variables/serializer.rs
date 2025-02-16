use super::VariableDef;
use tanitc_serializer::{Serialize, XmlWriter};

impl Serialize for VariableDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("variable-definition")?;
        writer.put_param("name", self.identifier)?;
        writer.put_param("is-global", self.is_global)?;
        writer.put_param("is-mutable", self.is_mutable)?;

        self.var_type.serialize(writer)?;

        writer.end_tag()?;

        Ok(())
    }
}
