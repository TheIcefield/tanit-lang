use super::EnumDef;
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for EnumDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("enum-definition")?;

        self.identifier.serialize(writer)?;

        for field in self.fields.iter() {
            writer.begin_tag("field")?;

            field.0.serialize(writer)?;

            if let Some(value) = &field.1 {
                writer.put_param("value", value)?;
            }

            writer.end_tag()?;
        }

        writer.end_tag()?;

        Ok(())
    }
}
