use super::AliasDef;
use tanitc_serializer::{Serialize, XmlWriter};

impl Serialize for AliasDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("alias-definition")?;
        writer.put_param("name", self.identifier)?;

        self.value.serialize(writer)?;

        writer.end_tag()?;

        Ok(())
    }
}
