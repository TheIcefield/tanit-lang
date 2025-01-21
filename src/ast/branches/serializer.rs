use super::{Branch, BranchType, Interupter, InterupterType};
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for Branch {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        match &self.branch {
            BranchType::While { body, condition } => {
                writer.begin_tag("while")?;

                writer.begin_tag("condition")?;

                condition.serialize(writer)?;

                writer.end_tag()?;

                body.serialize(writer)?;

                writer.end_tag()?;
            }
            BranchType::Loop { body } => {
                writer.begin_tag("loop")?;

                body.serialize(writer)?;

                writer.end_tag()?;
            }
            BranchType::If { condition, body } => {
                writer.begin_tag("if")?;

                writer.begin_tag("condition")?;
                condition.serialize(writer)?;
                writer.end_tag()?;

                writer.begin_tag("than")?;
                body.serialize(writer)?;
                writer.end_tag()?;

                writer.end_tag()?;
            }
            BranchType::Else { body } => {
                writer.begin_tag("else")?;

                body.serialize(writer)?;

                writer.end_tag()?;
            }
        }

        Ok(())
    }
}

impl Serialize for Interupter {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag(&format!("{}-statement", self.interupter.to_str()))?;

        match &self.interupter {
            InterupterType::Break { ret } | InterupterType::Return { ret } => {
                if let Some(expr) = ret {
                    expr.serialize(writer)?;
                }
            }
            _ => {}
        }

        writer.end_tag()?;

        Ok(())
    }
}
