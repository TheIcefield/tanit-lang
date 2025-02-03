use super::Identifier;
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for Identifier {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.put_param("name", self)
    }
}
