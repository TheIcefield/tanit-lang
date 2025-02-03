use super::{VariantDef, VariantField};
use tanitc_serializer::{Serialize, XmlWriter};

impl Serialize for VariantField {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        match self {
            Self::StructLike(s) => {
                for (field_id, field_type) in s.iter() {
                    writer.begin_tag("field")?;

                    field_id.serialize(writer)?;
                    field_type.serialize(writer)?;

                    writer.end_tag()?;
                }
            }
            Self::TupleLike(tuple_field) => {
                for tuple_component in tuple_field.iter() {
                    tuple_component.serialize(writer)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl Serialize for VariantDef {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("variant-definition")?;

        self.identifier.serialize(writer)?;

        for internal in self.internals.iter() {
            internal.serialize(writer)?;
        }

        for (field_id, field) in self.fields.iter() {
            writer.begin_tag("field")?;

            field_id.serialize(writer)?;

            if VariantField::Common == *field {
                writer.end_tag()?;
                continue;
            }

            field.serialize(writer)?;

            writer.end_tag()?;
        }

        writer.end_tag()?;

        Ok(())
    }
}
