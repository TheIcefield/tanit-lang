use super::EnumDef;
use tanitc_serializer::{Serialize, XmlWriter};

impl Serialize for EnumDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("enum-definition")?;
        writer.put_param("name", self.identifier)?;

        for field in self.fields.iter() {
            writer.begin_tag("field")?;
            writer.put_param("name", field.0)?;

            if let Some(value) = &field.1 {
                writer.put_param("value", value)?;
            }

            writer.end_tag()?;
        }

        writer.end_tag()?;

        Ok(())
    }
}
