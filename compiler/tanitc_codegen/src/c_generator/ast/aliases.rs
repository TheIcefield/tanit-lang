use tanitc_ast::ast::aliases::AliasDef;

use crate::c_generator::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_alias_def(&mut self, alias_def: &AliasDef) -> Result<(), std::io::Error> {
        use std::io::Write;

        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        write!(
            self,
            "typedef {} {}",
            alias_def.value.get_c_type(),
            alias_def.identifier
        )?;

        writeln!(self, ";")?;

        self.mode = old_mode;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        aliases::AliasDef, blocks::Block, structs::StructDef, types::TypeSpec, Ast,
    };
    use tanitc_ident::Ident;
    use tanitc_ty::Type;

    use pretty_assertions::assert_str_eq;

    use crate::c_generator::CodeGenStream;

    fn get_struct(name: &str) -> StructDef {
        StructDef {
            identifier: Ident::from(name.to_string()),
            ..Default::default()
        }
    }

    fn get_alias(name: &str, ty: Type) -> AliasDef {
        AliasDef {
            identifier: Ident::from(name.to_string()),
            value: TypeSpec {
                ty,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn codegen_alias_test() {
        const STRUCT_NAME: &str = "EmptyStruct";
        const ALIAS_1_NAME: &str = "FirstAlias";
        const ALIAS_2_NAME: &str = "SecondAlias";
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} EmptyStruct;\
                                     \ntypedef EmptyStruct FirstAlias;\
                                     \ntypedef FirstAlias SecondAlias;\n";

        let node = Ast::from(Block {
            is_global: true,
            statements: vec![
                get_struct(STRUCT_NAME).into(),
                get_alias(ALIAS_1_NAME, Type::Custom(STRUCT_NAME.to_string())).into(),
                get_alias(ALIAS_2_NAME, Type::Custom(ALIAS_1_NAME.to_string())).into(),
            ],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }
}
