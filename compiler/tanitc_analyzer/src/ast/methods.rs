use tanitc_ast::{FunctionDef, ImplDef};
use tanitc_messages::Message;

use crate::Analyzer;

impl Analyzer {
    pub fn analyze_impl_def(&mut self, impl_def: &mut ImplDef) -> Result<(), Message> {
        if self.table.lookup_mut(impl_def.identifier).is_none() {
            return Err(Message::undefined_id(
                impl_def.location,
                impl_def.identifier,
            ));
        };

        self.analyze_impl_methods(&mut impl_def.methods)?;

        Ok(())
    }

    fn analyze_impl_methods(&mut self, methods: &mut [FunctionDef]) -> Result<(), Message> {
        for method in methods.iter_mut() {
            const IS_METHOD: bool = true;

            match self.analyze_func_def(method, IS_METHOD) {
                Ok(_) => {}
                Err(err) => self.error(err),
            }
        }

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

    use crate::Analyzer;

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
            body: Some(Box::new(Block::default())),
            ..Default::default()
        }
    }

    #[test]
    fn self_in_beginning_good_test() {
        const STRUCT_NAME: &str = "MyStruct";

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

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![get_struct_def(STRUCT_NAME).into(), impl_def_node.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert!(errors.is_empty());
    }

    #[test]
    fn self_in_middle_test() {
        const STRUCT_NAME: &str = "MyStruct";
        const EXPECTED_ERR: &str = "Semantic error: In definition of function \"by_self\": Unexpected \"self\" parameter. Must be the first parameter of the associated function";

        let impl_def_node = get_impl_def(
            STRUCT_NAME,
            vec![get_func(
                "by_self",
                vec![
                    get_common_param("hello"),
                    FunctionParam::SelfVal(Mutability::Immutable),
                ],
            )],
        );

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![get_struct_def(STRUCT_NAME).into(), impl_def_node.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }

    #[test]
    fn self_in_func_test() {
        const EXPECTED_ERR: &str = "Semantic error: In definition of function \"by_self\": \"self\" parameter is allowed only in associated functions";

        let mut program = Ast::from(Block {
            is_global: true,
            statements: vec![get_func(
                "by_self",
                vec![
                    FunctionParam::SelfVal(Mutability::Immutable),
                    get_common_param("hello"),
                ],
            )
            .into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED_ERR);
    }
}
