use super::Type;
use tanitc_serializer::{Serialize, XmlWriter};

impl Serialize for Type {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("type")?;
        match self {
            Self::Ref { is_mut, ref_to } => {
                writer.put_param("style", "reference")?;
                writer.put_param("is-mutable", is_mut)?;

                ref_to.serialize(writer)?;
            }

            Self::Ptr { is_mut, ptr_to } => {
                writer.put_param("style", "pointer")?;
                writer.put_param("is-mutable", is_mut)?;

                ptr_to.serialize(writer)?;
            }

            Self::Tuple { components } => {
                writer.put_param("style", "tuple")?;

                for comp in components.iter() {
                    comp.serialize(writer)?;
                }
            }

            Self::Array { size, value_type } => {
                writer.put_param("style", "array")?;

                if let Some(size) = size {
                    writer.begin_tag("size")?;
                    size.serialize(writer)?;
                    writer.end_tag()?;
                }

                value_type.serialize(writer)?;
            }

            Self::Template {
                identifier,
                arguments,
            } => {
                writer.put_param("style", "generic")?;
                writer.put_param("name", identifier)?;

                for arg in arguments.iter() {
                    arg.serialize(writer)?;
                }
            }

            Self::Custom(id) => {
                writer.put_param("style", "named")?;
                writer.put_param("name", id)?
            }

            Self::Auto => writer.put_param("style", "automatic")?,

            Self::Bool => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "bool")?;
            }
            Self::I8 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "i8")?;
            }
            Self::I16 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "i16")?;
            }
            Self::I32 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "i32")?;
            }
            Self::I64 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "i64")?;
            }
            Self::I128 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "i128")?;
            }
            Self::U8 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "u8")?;
            }
            Self::U16 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "u16")?;
            }
            Self::U32 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "u32")?;
            }
            Self::U64 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "u64")?;
            }
            Self::U128 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "u128")?;
            }
            Self::F32 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "f32")?;
            }
            Self::F64 => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "f64")?;
            }
            Self::Str => {
                writer.put_param("style", "primitive")?;
                writer.put_param("name", "str")?;
            }
        }

        writer.end_tag()?;

        Ok(())
    }
}
