use super::FunctionDef;
use tanitc_serializer::{Serialize, XmlWriter};

impl Serialize for FunctionDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("function-definition")?;

        self.identifier.serialize(writer)?;

        writer.begin_tag("return-type")?;
        self.return_type.serialize(writer)?;
        writer.end_tag()?;

        if !self.parameters.is_empty() {
            writer.begin_tag("parameters")?;
            for param in self.parameters.iter() {
                param.serialize(writer)?;
            }
            writer.end_tag()?;
        }

        if let Some(body) = &self.body {
            body.serialize(writer)?;
        }

        writer.end_tag()?;

        Ok(())
    }
}
