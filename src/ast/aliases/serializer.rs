use super::AliasDef;
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for AliasDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("alias-defintion")?;

        self.identifier.serialize(writer)?;
        self.value.serialize(writer)?;

        writer.end_tag()?;

        Ok(())
    }
}
