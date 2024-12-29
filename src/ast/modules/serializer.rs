use super::ModuleDef;
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for ModuleDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("module-definition")?;

        self.body.serialize(writer)?;

        writer.end_tag()?;

        Ok(())
    }
}
