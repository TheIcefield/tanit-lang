use super::{MetaInfo, TypeSpec};

use tanitc_serializer::{Serialize, XmlWriter};
use tanitc_ty::Type;

fn serialize(ty: &Type, info: MetaInfo, writer: &mut XmlWriter) -> std::io::Result<()> {
    match ty {
        Type::Ref(ref_to) => {
            writer.put_param("style", "reference")?;
            writer.put_param("is-mutable", info.is_mut)?;

            serialize(ref_to, info, writer)?;
        }
        Type::Ptr(ptr_to) => {
            writer.put_param("style", "pointer")?;
            writer.put_param("is-mutable", info.is_mut)?;

            serialize(ptr_to, info, writer)?;
        }
        Type::Tuple(components) => {
            writer.put_param("style", "tuple")?;

            for comp in components.iter() {
                serialize(comp, info, writer)?;
            }
        }
        Type::Array { size, value_type } => {
            writer.put_param("style", "array")?;

            if let Some(size) = size {
                writer.put_param("size", size)?;
            }

            serialize(value_type, info, writer)?;
        }
        Type::Template {
            identifier,
            generics,
        } => {
            writer.put_param("style", "generic")?;
            writer.put_param("name", identifier)?;

            for generic in generics.iter() {
                serialize(generic, info, writer)?;
            }
        }
        Type::Custom(id) => {
            writer.put_param("style", "named")?;
            writer.put_param("name", id)?
        }
        Type::Auto => writer.put_param("style", "automatic")?,
        Type::Never => {
            writer.put_param("style", "never")?;
        }
        Type::Bool => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "bool")?;
        }
        Type::I8 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "i8")?;
        }
        Type::I16 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "i16")?;
        }
        Type::I32 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "i32")?;
        }
        Type::I64 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "i64")?;
        }
        Type::I128 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "i128")?;
        }
        Type::U8 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "u8")?;
        }
        Type::U16 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "u16")?;
        }
        Type::U32 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "u32")?;
        }
        Type::U64 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "u64")?;
        }
        Type::U128 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "u128")?;
        }
        Type::F32 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "f32")?;
        }
        Type::F64 => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "f64")?;
        }
        Type::Str => {
            writer.put_param("style", "primitive")?;
            writer.put_param("name", "str")?;
        }
    }

    Ok(())
}

impl Serialize for TypeSpec {
    fn serialize(&self, writer: &mut XmlWriter) -> std::io::Result<()> {
        writer.begin_tag("type")?;

        serialize(&self.ty, self.info, writer)?;

        writer.end_tag()?;

        Ok(())
    }
}
