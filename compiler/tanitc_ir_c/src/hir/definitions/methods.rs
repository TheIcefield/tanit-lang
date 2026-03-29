use tanitc_hir::hir::definitions::methods::ImplDef;

use crate::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_impl_def(&mut self, impl_def: &ImplDef) -> std::io::Result<()> {
        let old_mode = self.mode;
        self.mode = CodeGenMode::HeaderOnly;

        for method in impl_def.methods.iter() {
            self.generate_func_def(method, Some(&impl_def.name))?;
        }

        self.mode = old_mode;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tanitc_attributes::Mutability;
    use tanitc_hir::hir::{
        blocks::Block, definitions::functions::FunctionParam, type_spec::Type, Hir,
    };
    use tanitc_hir_test::{
        create_common_func_param, create_func_def, create_impl_def, create_program,
        create_struct_def,
    };

    use pretty_assertions::assert_str_eq;

    #[test]
    fn self_in_beginning_good_test() {
        // Given

        const STRUCT_NAME: &str = "MyStruct";
        let struct_def = create_struct_def(STRUCT_NAME, vec![]);

        let impl_def = create_impl_def(
            STRUCT_NAME,
            vec![create_func_def(
                "by_self",
                vec![
                    FunctionParam::SelfVal(Mutability::Immutable),
                    create_common_func_param("hello", Mutability::Immutable, Type::I32),
                ],
                Type::unit(),
                vec![],
            )],
        );

        let program = Hir::from(Block {
            is_global: true,
            statements: vec![struct_def.into(), impl_def.into()],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        // When
        program.accept(&mut writer).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} MyStruct;\
                                     \nvoid MyStruct__by_self(MyStruct const self, signed int const hello);\n";

        const SOURCE_EXPECTED: &str =
            "void MyStruct__by_self(MyStruct const self, signed int const hello) { }\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn mut_self_in_beginning_good_test() {
        // Given
        const STRUCT_NAME: &str = "MyStruct";
        let struct_def = create_struct_def(STRUCT_NAME, vec![]);

        let func_def = create_func_def(
            "by_mut_self",
            vec![FunctionParam::SelfVal(Mutability::Mutable)],
            Type::unit(),
            vec![],
        );
        let impl_def = create_impl_def(STRUCT_NAME, vec![func_def]);

        let program = create_program(vec![struct_def.into(), impl_def.into()]);

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        // When
        program.accept(&mut writer).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} MyStruct;\
                                     \nvoid MyStruct__by_mut_self(MyStruct self);\n";

        const SOURCE_EXPECTED: &str = "void MyStruct__by_mut_self(MyStruct self) { }\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn self_ref_in_beginning_good_test() {
        // Given
        const STRUCT_NAME: &str = "MyStruct";
        let struct_def = create_struct_def(STRUCT_NAME, vec![]);

        let func_def = create_func_def(
            "by_self_ref",
            vec![FunctionParam::SelfRef(Mutability::Immutable)],
            Type::unit(),
            vec![],
        );
        let impl_def = create_impl_def(STRUCT_NAME, vec![func_def]);

        let program = create_program(vec![struct_def.into(), impl_def.into()]);

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        // When
        program.accept(&mut writer).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} MyStruct;\
                                     \nvoid MyStruct__by_self_ref(MyStruct const * const self);\n";

        const SOURCE_EXPECTED: &str =
            "void MyStruct__by_self_ref(MyStruct const * const self) { }\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn mut_self_ref_in_beginning_good_test() {
        // Given
        const STRUCT_NAME: &str = "MyStruct";
        let struct_def = create_struct_def(STRUCT_NAME, vec![]);

        let func_def = create_func_def(
            "by_mut_self_ref",
            vec![
                FunctionParam::SelfRef(Mutability::Mutable),
                create_common_func_param("hello", Mutability::Immutable, Type::I32),
            ],
            Type::unit(),
            vec![],
        );
        let impl_def = create_impl_def(STRUCT_NAME, vec![func_def]);

        let program = create_program(vec![struct_def.into(), impl_def.into()]);

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        // When
        program.accept(&mut writer).unwrap();

        // Then
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} MyStruct;\
                                     \nvoid MyStruct__by_mut_self_ref(MyStruct * const self, signed int const hello);\n";

        const SOURCE_EXPECTED: &str =
            "void MyStruct__by_mut_self_ref(MyStruct * const self, signed int const hello) { }\n";

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
