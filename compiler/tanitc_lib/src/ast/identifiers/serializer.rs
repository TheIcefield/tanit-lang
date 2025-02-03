use super::Identifier;
use tanitc_serializer::{Serialize, XmlWriter};

impl Serialize for Identifier {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.put_param("name", self)
    }
}
