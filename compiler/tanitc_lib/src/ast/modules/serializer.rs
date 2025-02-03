use super::ModuleDef;
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for ModuleDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        if self.body.is_some() {
            self.serialize_internal(writer)
        } else {
            self.serialize_external(writer)
        }
    }
}

impl ModuleDef {
    fn serialize_internal(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("module-definition")?;

        self.identifier.serialize(writer)?;

        if let Some(body) = &self.body {
            body.serialize(writer)?;
        }

        writer.end_tag()?;

        Ok(())
    }

    fn serialize_external(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("module-import")?;

        self.identifier.serialize(writer)?;

        writer.end_tag()?;

        Ok(())
    }
}
