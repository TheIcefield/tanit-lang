use tanitc_ast::ImplDef;

use crate::c_generator::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_impl_def(&mut self, impl_def: &ImplDef) -> Result<(), std::io::Error> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        for method in impl_def.methods.iter() {
            self.generate_func_def(method, Some(impl_def.identifier))?;
        }

        self.mode = old_mode;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::{
        Ast, Block, FunctionDef, FunctionParam, ImplDef, StructDef, TypeSpec, VariableDef,
    };
    use tanitc_attributes::Mutability;
    use tanitc_ident::Ident;
    use tanitc_ty::Type;

    use crate::c_generator::CodeGenStream;

    use pretty_assertions::assert_str_eq;

    fn get_struct_def(name: &str) -> StructDef {
        StructDef {
            identifier: Ident::from(name.to_string()),
            ..Default::default()
        }
    }

    fn get_impl_def(name: &str, methods: Vec<FunctionDef>) -> ImplDef {
        ImplDef {
            identifier: Ident::from(name.to_string()),
            methods,
            ..Default::default()
        }
    }

    fn get_common_param(name: &str) -> FunctionParam {
        FunctionParam::Common(VariableDef {
            identifier: Ident::from(name.to_string()),
            var_type: TypeSpec {
                ty: Type::I32,
                ..Default::default()
            },
            is_global: false,
            mutability: Mutability::Immutable,
            ..Default::default()
        })
    }

    fn get_func(name: &str, parameters: Vec<FunctionParam>) -> FunctionDef {
        FunctionDef {
            identifier: Ident::from(name.to_string()),
            return_type: TypeSpec::default(),
            parameters,
            body: Some(Box::new(Ast::from(Block::default()))),
            ..Default::default()
        }
    }

    #[test]
    fn self_in_beginning_good_test() {
        const STRUCT_NAME: &str = "MyStruct";
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} MyStruct;\
                                     \nvoid MyStruct__by_self(MyStruct const self, signed int const hello);\n";

        const SOURCE_EXPECTED: &str =
            "void MyStruct__by_self(MyStruct const self, signed int const hello){\
           \n}\n";

        let impl_def_node = get_impl_def(
            STRUCT_NAME,
            vec![get_func(
                "by_self",
                vec![
                    FunctionParam::SelfVal(Mutability::Immutable),
                    get_common_param("hello"),
                ],
            )],
        );

        let program = Ast::from(Block {
            is_global: true,
            statements: vec![get_struct_def(STRUCT_NAME).into(), impl_def_node.into()],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
