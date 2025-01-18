use super::ModuleDef;
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for ModuleDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("module-definition")?;

        if let Some(body) = &self.body {
            body.serialize(writer)?;
        }

        writer.end_tag()?;

        Ok(())
    }
}
