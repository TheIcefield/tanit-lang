use super::Scope;
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for Scope {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        for stmt in self.statements.iter() {
            stmt.serialize(writer)?;
        }

        Ok(())
    }
}
