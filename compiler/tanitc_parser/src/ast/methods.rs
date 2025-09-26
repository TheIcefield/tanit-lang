use tanitc_ast::ast::{methods::ImplDef, Ast};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

// Impl definition
impl Parser {
    pub fn parse_impl_def(&mut self) -> Result<Ast, Message> {
        let mut node = ImplDef::default();

        self.parse_impl_header(&mut node)?;
        self.parse_impl_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_impl_header(&mut self, impl_def: &mut ImplDef) -> Result<(), Message> {
        impl_def.location = self.consume_token(Lexem::KwImpl)?.location;
        impl_def.identifier = self.consume_identifier()?;

        Ok(())
    }

    fn parse_impl_body(&mut self, impl_def: &mut ImplDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;

        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(true);

        let methods = self.parse_global_block()?;

        let Ast::Block(mut block) = methods else {
            return Err(Message {
                location: methods.location(),
                text: format!("Unexpected node {} within impl block", methods.name()),
            });
        };

        for method in block.statements.iter_mut() {
            if let Ast::FuncDef(method) = method {
                impl_def.methods.push(std::mem::take(method));
            }
        }

        self.set_ignore_nl_option(old_opt);

        self.consume_token(Lexem::Rcb)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;
    use tanitc_ast::ast::{
        functions::FunctionParam,
        types::{ParsedTypeInfo, TypeSpec},
        variables::{VariableAttributes, VariableDef},
        Ast,
    };
    use tanitc_attributes::{Mutability, Visibility};
    use tanitc_ident::Ident;
    use tanitc_lexer::location::Location;
    use tanitc_ty::Type;

    fn get_location(row: usize, col: usize) -> Location {
        Location {
            row,
            col,
            ..Default::default()
        }
    }

    #[test]
    fn parse_impl_def_good_test() {
        const SRC_TEXT: &str = "\nstruct MyStruct\
                                \n{\
                                \n}\
                                \nimpl MyStruct\
                                \n{\
                                \n    func empty() {\
                                \n    }\
                                \n    func with_self(self) {\
                                \n    }\
                                \n    func with_self_p(self, p: i32) {\
                                \n    }\
                                \n    func with_mut_self(mut self) {\
                                \n    }\
                                \n    func with_self_ref(& self) {\
                                \n    }\
                                \n    func with_mut_self_ref(& mut self) {\
                                \n    }\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT).expect("Failed to create parser");

        let program = parser.parse_global_block().unwrap();
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }

        let Ast::Block(block) = program else {
            panic!("Expected Ast::Block, actually: {}", program.name());
        };

        assert_eq!(block.statements.len(), 2);

        {
            const STRUCT_DEF_INDEX: usize = 0;

            let Ast::StructDef(struct_def) = &block.statements[STRUCT_DEF_INDEX] else {
                panic!(
                    "Expected Ast::StructDef, actually: {}",
                    block.statements[STRUCT_DEF_INDEX].name()
                );
            };

            assert_eq!(struct_def.name.id.to_string(), "MyStruct");
            assert!(struct_def.fields.is_empty());
        }

        {
            const IMPL_DEF_INDEX: usize = 1;
            const METHODS_COUNT: usize = 6;

            let Ast::ImplDef(impl_def) = &block.statements[IMPL_DEF_INDEX] else {
                panic!(
                    "Expected Ast::ImplDef, actually: {}",
                    block.statements[IMPL_DEF_INDEX].name()
                );
            };

            assert_eq!(impl_def.identifier.to_string(), "MyStruct");
            assert_eq!(impl_def.methods.len(), METHODS_COUNT);

            {
                const METHOD_INDEX: usize = 0;
                const METHOD_NAME: &str = "empty";

                let method = &impl_def.methods[METHOD_INDEX];

                assert_eq!(method.name.to_string(), METHOD_NAME);
                assert!(method.parameters.is_empty());
            }

            {
                const METHOD_INDEX: usize = 1;
                const METHOD_NAME: &str = "with_self";
                const PARAMS_COUNT: usize = 1;

                let method = &impl_def.methods[METHOD_INDEX];

                assert_eq!(method.name.to_string(), METHOD_NAME);
                assert_eq!(method.parameters.len(), PARAMS_COUNT);
                assert_eq!(
                    method.parameters[0],
                    FunctionParam::SelfVal(Mutability::Immutable)
                );
            }

            {
                const METHOD_INDEX: usize = 2;
                const METHOD_NAME: &str = "with_self_p";
                const PARAMS_COUNT: usize = 2;

                let method = &impl_def.methods[METHOD_INDEX];

                assert_eq!(method.name.to_string(), METHOD_NAME);
                assert_eq!(method.parameters.len(), PARAMS_COUNT);
                assert_eq!(
                    method.parameters[0],
                    FunctionParam::SelfVal(Mutability::Immutable)
                );
                assert_eq!(
                    method.parameters[1],
                    FunctionParam::Common(VariableDef {
                        location: get_location(11, 29),
                        attributes: VariableAttributes::default(),
                        identifier: Ident::from("p".to_string()),
                        var_type: TypeSpec {
                            location: get_location(11, 32),
                            info: ParsedTypeInfo {
                                mutability: Mutability::Immutable
                            },
                            ty: Type::I32
                        },
                        visibility: Visibility::Local,
                        mutability: Mutability::Immutable
                    })
                );
            }

            {
                const METHOD_INDEX: usize = 3;
                const METHOD_NAME: &str = "with_mut_self";
                const PARAMS_COUNT: usize = 1;

                let method = &impl_def.methods[METHOD_INDEX];

                assert_eq!(method.name.to_string(), METHOD_NAME);
                assert_eq!(method.parameters.len(), PARAMS_COUNT);
                assert_eq!(
                    method.parameters[0],
                    FunctionParam::SelfVal(Mutability::Mutable)
                );
            }

            {
                const METHOD_INDEX: usize = 4;
                const METHOD_NAME: &str = "with_self_ref";
                const PARAMS_COUNT: usize = 1;

                let method = &impl_def.methods[METHOD_INDEX];

                assert_eq!(method.name.to_string(), METHOD_NAME);
                assert_eq!(method.parameters.len(), PARAMS_COUNT);
                assert_eq!(
                    method.parameters[0],
                    FunctionParam::SelfRef(Mutability::Immutable)
                );
            }

            {
                const METHOD_INDEX: usize = 5;
                const METHOD_NAME: &str = "with_mut_self_ref";
                const PARAMS_COUNT: usize = 1;

                let method = &impl_def.methods[METHOD_INDEX];

                assert_eq!(method.name.to_string(), METHOD_NAME);
                assert_eq!(method.parameters.len(), PARAMS_COUNT);
                assert_eq!(
                    method.parameters[0],
                    FunctionParam::SelfRef(Mutability::Mutable)
                );
            }
        }
    }

    #[test]
    fn parse_impl_def_bad_test() {
        const SRC_TEXT: &str = "\nstruct MyStruct\
                                \n{\
                                \n}\
                                \nimpl MyStruct\
                                \n{\
                                \n    func with_mut_self(self mut) {\
                                \n    }\
                                \n    func with_mut_self_ref(mut & self) {\
                                \n    }\
                                \n}";

        const ERR_1: &str =
            "Syntax error: In definition of function \"with_mut_self\": Unexpected token: ). ";
        const ERR_2: &str =
            "Syntax error: In definition of function \"with_mut_self_ref\": \"Mut\" must be followed named binding";

        let mut parser = Parser::from_text(SRC_TEXT).expect("Failed to create parser");

        let _ = parser.parse_global_block().unwrap();

        let errors = parser.get_errors();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].text, ERR_1);
        assert_eq!(errors[1].text, ERR_2);
    }
}
