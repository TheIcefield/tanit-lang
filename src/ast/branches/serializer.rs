use super::{Branch, BranchType, Break, Continue, Return};
use crate::serializer::{Serialize, XmlWriter};

impl Serialize for Branch {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        match &self.branch {
            BranchType::Loop { body, condition } => {
                writer.begin_tag("loop")?;

                if let Some(cond) = condition {
                    writer.begin_tag("condition")?;

                    cond.serialize(writer)?;

                    writer.end_tag()?;
                }

                body.serialize(writer)?;

                writer.end_tag()?;
            }
            BranchType::IfElse {
                condition,
                main_body,
                else_body,
            } => {
                writer.begin_tag("branch")?;

                writer.begin_tag("condition")?;
                condition.serialize(writer)?;
                writer.end_tag()?;

                writer.begin_tag("true")?;
                main_body.serialize(writer)?;
                writer.end_tag()?;

                if let Some(else_body) = else_body {
                    writer.begin_tag("false")?;
                    else_body.serialize(writer)?;
                    writer.end_tag()?;
                }

                writer.end_tag()?;
            }
        }

        Ok(())
    }
}

impl Serialize for Break {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("break-statement")?;

        if let Some(expr) = &self.expr {
            expr.serialize(writer)?;
        }

        writer.end_tag()?;

        Ok(())
    }
}

impl Serialize for Continue {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("continue-statement")?;
        writer.end_tag()
    }
}

impl Serialize for Return {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("return-statement")?;

        if let Some(expr) = &self.expr {
            expr.serialize(writer)?;
        }

        writer.end_tag()?;

        Ok(())
    }
}
