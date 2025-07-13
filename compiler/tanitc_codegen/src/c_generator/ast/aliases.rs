use tanitc_ast::AliasDef;

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

#[test]
fn alias_def_codegen_test() {
    use tanitc_ast::{Ast, Block, TypeSpec};
    use tanitc_ident::Ident;
    use tanitc_ty::Type;

    let expected_name = Ident::from("MyAlias".to_string());
    let expected_header = format!("typedef float {expected_name};\n");

    let program = Ast::from(Block {
        is_global: true,
        statements: vec![Ast::from(AliasDef {
            identifier: expected_name,
            value: TypeSpec {
                ty: Type::F32,
                ..Default::default()
            },
            ..Default::default()
        })],
        ..Default::default()
    });

    let mut header_buffer = Vec::<u8>::new();
    let mut source_buffer = Vec::<u8>::new();
    let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

    program.accept(&mut writer).unwrap();

    let header_res = String::from_utf8(header_buffer).unwrap();
    assert_eq!(expected_header, header_res);

    let source_res = String::from_utf8(source_buffer).unwrap();
    assert!(source_res.is_empty());
}
