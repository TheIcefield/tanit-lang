use super::StructDef;
use tanitc_serializer::{Serialize, XmlWriter};

impl Serialize for StructDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("struct-definition")?;

        self.identifier.serialize(writer)?;

        for internal in self.internals.iter() {
            internal.serialize(writer)?;
        }

        for (field_id, field_type) in self.fields.iter() {
            writer.begin_tag("field")?;

            field_id.serialize(writer)?;
            field_type.serialize(writer)?;

            writer.end_tag()?;
        }

        writer.end_tag()?;

        Ok(())
    }
}
