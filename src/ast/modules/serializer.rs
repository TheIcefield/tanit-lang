use super::ModuleDef;
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for ModuleDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        self.serialize_external(writer)
    }
}

impl ModuleDef {
    fn serialize_internal(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("module-definition")?;

        self.identifier.serialize(writer)?;
        self.body.serialize(writer)?;

        writer.end_tag()?;

        Ok(())
    }

    fn serialize_external(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("module-import")?;

        self.identifier.serialize(writer)?;

        let mut stream = std::fs::File::create(format!("{}_ast.xml", self.identifier))
            .expect("Error: can't create file for external AST serialization");

        let mut external_writer =
            XmlWriter::new(&mut stream).expect("Error: can't create serializer for external AST");

        self.serialize_internal(&mut external_writer)?;

        writer.end_tag()?;

        Ok(())
    }
}
