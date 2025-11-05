use tanitc_hir::hir::definitions::functions::{FunctionDef, FunctionParam};
use tanitc_ident::Ident;

use crate::{CodeGenMode, CodeGenStream};

use std::io::{ErrorKind, Write};

impl CodeGenStream<'_> {
    pub fn generate_func_def(
        &mut self,
        func_def: &FunctionDef,
        struct_name: Option<Ident>,
    ) -> std::io::Result<()> {
        let old_mode = self.mode;
        self.mode = if func_def.body.is_some() {
            CodeGenMode::Both
        } else {
            CodeGenMode::HeaderOnly
        };

        let indentation = self.indentation();

        write!(self, "{indentation}")?;

        self.generate_type(&func_def.return_type)?;

        let full_name = if let Some(struct_name) = struct_name {
            format!("{struct_name}__{}", func_def.name)
        } else {
            format!("{}", func_def.name)
        };

        write!(self, " {full_name}")?;

        self.generate_func_def_params(func_def, struct_name)?;

        self.mode = CodeGenMode::HeaderOnly;
        writeln!(self, ";")?;

        if let Some(body) = &func_def.body {
            self.mode = CodeGenMode::SourceOnly;
            if body.statements.is_empty() {
                writeln!(self, " {{ }}")?;
            } else {
                writeln!(self)?;
                self.generate_block(body)?;
            }
        }

        self.mode = old_mode;
        Ok(())
    }

    fn generate_func_def_param(
        &mut self,
        param: &FunctionParam,
        struct_name: Option<Ident>,
    ) -> std::io::Result<()> {
        match param {
            FunctionParam::SelfVal(mutability) => {
                let Some(struct_name) = struct_name else {
                    return Err(std::io::Error::from(ErrorKind::InvalidData));
                };

                write!(
                    self,
                    "{struct_name} {}self",
                    if mutability.is_const() { "const " } else { "" }
                )
            }
            FunctionParam::SelfRef(mutability) | FunctionParam::SelfPtr(mutability) => {
                let Some(struct_name) = struct_name else {
                    return Err(std::io::Error::from(ErrorKind::InvalidData));
                };

                write!(
                    self,
                    "{struct_name} {}* const self",
                    if mutability.is_const() { "const " } else { "" }
                )
            }
            FunctionParam::Common(var_def) => self.generate_variable_def(var_def),
        }
    }

    fn generate_func_def_params(
        &mut self,
        func_def: &FunctionDef,
        struct_name: Option<Ident>,
    ) -> std::io::Result<()> {
        write!(self, "(")?;
        if !func_def.parameters.is_empty() {
            let param = func_def.parameters.first().unwrap();
            self.generate_func_def_param(param, struct_name)?;
        }

        for param in func_def.parameters.iter().skip(1) {
            write!(self, ", ")?;
            self.generate_func_def_param(param, struct_name)?;
        }
        write!(self, ")")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tanitc_attributes::Mutability;
    use tanitc_hir::hir::{
        blocks::Block,
        definitions::variables::VariableDef,
        types::{RefType, Type},
        Hir,
    };
    use tanitc_ident::{Ident, Name};

    use pretty_assertions::assert_str_eq;

    fn get_func_param(name: &str, var_type: Type) -> FunctionParam {
        FunctionParam::Common(VariableDef {
            var_type,
            identifier: Ident::from(name.to_string()),
            ..Default::default()
        })
    }

    fn get_func(name: &str, parameters: Vec<FunctionParam>, return_type: Type) -> FunctionDef {
        FunctionDef {
            parameters,
            return_type,
            name: Name::from(name.to_string()),
            body: Some(Box::new(Block::default())),
            ..Default::default()
        }
    }

    #[test]
    fn func_codegen_test() {
        const FUNC_NAME: &str = "hello";
        const PARAM_1_NAME: &str = "a";

        const HEADER_EXPECTED: &str = "unsigned char hello(signed long long const a);\n";
        const SOURCE_EXPECTED: &str = "unsigned char hello(signed long long const a) { }\n";

        let node = Hir::from(get_func(
            FUNC_NAME,
            vec![get_func_param(PARAM_1_NAME, Type::I128)],
            Type::Bool,
        ));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn empty_func_codegen_test() {
        const FUNC_NAME: &str = "empty_func";

        const HEADER_EXPECTED: &str = "signed long empty_func();\n";
        const SOURCE_EXPECTED: &str = "signed long empty_func() { }\n";

        let node = Hir::from(get_func(FUNC_NAME, vec![], Type::I64));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }

    #[test]
    fn full_func_codegen_test() {
        const FUNC_NAME: &str = "full_func";
        const PARAM_1_NAME: &str = "ref";
        const PARAM_2_NAME: &str = "mut_ref";
        const PARAM_3_NAME: &str = "integer";
        const PARAM_4_NAME: &str = "string";

        const HEADER_EXPECTED: &str =
            "void full_func(signed int const * const ref, signed int * const mut_ref, unsigned int const integer, char const * const string);\n";
        const SOURCE_EXPECTED: &str =
            "void full_func(signed int const * const ref, signed int * const mut_ref, unsigned int const integer, char const * const string) { }\n";

        let node = Hir::from(get_func(
            FUNC_NAME,
            vec![
                get_func_param(
                    PARAM_1_NAME,
                    Type::Ref(RefType {
                        ref_to: Box::new(Type::I32),
                        mutability: Mutability::Immutable,
                    }),
                ),
                get_func_param(
                    PARAM_2_NAME,
                    Type::Ref(RefType {
                        ref_to: Box::new(Type::I32),
                        mutability: Mutability::Mutable,
                    }),
                ),
                get_func_param(PARAM_3_NAME, Type::I8),
                get_func_param(
                    PARAM_4_NAME,
                    Type::Ref(RefType {
                        ref_to: Box::new(Type::Str),
                        mutability: Mutability::Immutable,
                    }),
                ),
            ],
            Type::unit(),
        ));

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer);

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
