use tanitc_ast::{name::Name, Fields, TypeSpec, VariantDef, VariantField, VariantFields};
use tanitc_ident::Ident;

use crate::c_generator::{CodeGenMode, CodeGenStream};

use std::{collections::BTreeMap, io::Write};

impl CodeGenStream<'_> {
    pub fn generate_variant_def(&mut self, variant_def: &VariantDef) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        self.generate_variant_kind(&variant_def.name, &variant_def.fields)?;
        self.generate_variant_data(&variant_def.name, &variant_def.fields)?;

        writeln!(self, "typedef struct {{")?;

        self.generate_variant_kind_field(&variant_def.name)?;
        self.generate_variant_data_field(&variant_def.name)?;

        writeln!(self, "}} {};\n", variant_def.name.full_name())?;

        self.mode = old_mode;
        Ok(())
    }

    fn generate_variant_kind(
        &mut self,
        variant_name: &Name,
        fields: &VariantFields,
    ) -> Result<(), std::io::Error> {
        let enum_name = tanitc_ast::variant_utils::get_variant_data_kind_id(variant_name);

        // Enum definition
        writeln!(self, "typedef enum {{")?;
        for (field_id, _) in fields.iter() {
            writeln!(self, "    {enum_name}{field_id}__,")?;
        }
        writeln!(self, "}} {enum_name};\n")?;

        Ok(())
    }

    fn generate_variant_kind_field(&mut self, variant_name: &Name) -> Result<(), std::io::Error> {
        let enum_name = tanitc_ast::variant_utils::get_variant_data_kind_id(variant_name);
        let field_name = Ident::from("__kind__".to_string());

        writeln!(self, "    {enum_name} {field_name};")?;

        Ok(())
    }

    fn generate_variant_common_field(
        &mut self,
        union_id: Ident,
        field_id: Ident,
    ) -> Result<(), std::io::Error> {
        let struct_name = format!("{union_id}{field_id}__");

        writeln!(self, "typedef struct {{ }} {struct_name};")?;

        Ok(())
    }

    fn generate_variant_struct_field(
        &mut self,
        union_id: Ident,
        field_id: Ident,
        subfields: &Fields,
    ) -> Result<(), std::io::Error> {
        let struct_name = format!("{union_id}{field_id}__");

        writeln!(self, "typedef struct {{")?;

        for (subfield_id, subfield_type) in subfields.iter() {
            let subfield_type = subfield_type.ty.get_type().get_c_type();
            writeln!(self, "    {subfield_type} {subfield_id};")?;
        }

        writeln!(self, "}} {struct_name};")?;

        Ok(())
    }

    fn generate_variant_tuple_field(
        &mut self,
        union_id: Ident,
        field_id: Ident,
        components: &[TypeSpec],
    ) -> Result<(), std::io::Error> {
        let struct_name = format!("{union_id}{field_id}__");

        writeln!(self, "typedef struct {{")?;

        for (field_num, field_type) in components.iter().enumerate() {
            let field_type = field_type.get_type().get_c_type();
            writeln!(self, "    {field_type} _{field_num};")?;
        }

        writeln!(self, "}} {struct_name};")?;

        Ok(())
    }

    fn generate_variant_data_types(
        &mut self,
        variant_name: &Name,
        fields: &BTreeMap<Ident, VariantField>,
    ) -> Result<(), std::io::Error> {
        let union_name = tanitc_ast::variant_utils::get_variant_data_type_id(variant_name);

        for (field_id, field_data) in fields.iter() {
            match field_data {
                VariantField::Common => {
                    self.generate_variant_common_field(union_name, *field_id)?
                }
                VariantField::StructLike(subfields) => {
                    self.generate_variant_struct_field(union_name, *field_id, subfields)?
                }
                VariantField::TupleLike(components) => {
                    self.generate_variant_tuple_field(union_name, *field_id, components)?
                }
            }
            writeln!(self)?;
        }

        Ok(())
    }

    fn generate_variant_data_fields(
        &mut self,
        variant_name: &Name,
        fields: &BTreeMap<Ident, VariantField>,
    ) -> Result<(), std::io::Error> {
        let union_name = tanitc_ast::variant_utils::get_variant_data_type_id(variant_name);

        writeln!(self, "typedef union {union_name} {{")?;

        for (field_id, _) in fields.iter() {
            writeln!(self, "    {union_name}{field_id}__ {field_id};")?;
        }

        writeln!(self, "}} {union_name};\n")?;

        Ok(())
    }

    fn generate_variant_data(
        &mut self,
        variant_name: &Name,
        fields: &BTreeMap<Ident, VariantField>,
    ) -> Result<(), std::io::Error> {
        self.generate_variant_data_types(variant_name, fields)?;
        self.generate_variant_data_fields(variant_name, fields)?;

        Ok(())
    }

    fn generate_variant_data_field(&mut self, variant_name: &Name) -> Result<(), std::io::Error> {
        let union_name = tanitc_ast::variant_utils::get_variant_data_type_id(variant_name);
        let field_name = Ident::from("__data__".to_string());

        writeln!(self, "    {union_name} {field_name};")?;

        Ok(())
    }
}
