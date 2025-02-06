use super::StructDef;
use tanitc_serializer::{Serialize, XmlWriter};

impl Serialize for StructDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("struct-definition")?;
        writer.put_param("name", self.identifier)?;

        for internal in self.internals.iter() {
            internal.serialize(writer)?;
        }

        for (field_id, field_type) in self.fields.iter() {
            writer.begin_tag("field")?;
            writer.put_param("name", field_id)?;

            field_type.serialize(writer)?;

            writer.end_tag()?;
        }

        writer.end_tag()?;

        Ok(())
    }
}
