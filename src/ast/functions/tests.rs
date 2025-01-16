use super::FunctionDef;
use crate::ast::{expressions::ExpressionType, Ast};
use crate::parser::{lexer::Lexer, token::Lexem, Parser};
use crate::serializer::XmlWriter;

#[test]
fn function_def_test() {
    const SRC_TEXT: &str = "\nfunc foo(a: i32, b: i32) -> f32 {\
                             \n    return a + b\
                             \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT, true).expect("Lexer creation failed"));

    let node = FunctionDef::parse(&mut parser).unwrap();

    {
        const EXPECTED: &str = "\n<function-definition name=\"foo\">\
                                \n    <return-type>\
                                \n        <type style=\"primitive\" name=\"f32\"/>\
                                \n    </return-type>\
                                \n    <parameters>\
                                \n        <variable-definition name=\"a\" is-global=\"false\" is-mutable=\"true\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </variable-definition>\
                                \n        <variable-definition name=\"b\" is-global=\"false\" is-mutable=\"true\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </variable-definition>\
                                \n    </parameters>\
                                \n    <return-statement>\
                                \n        <operation style=\"binary\" operation=\"+\">\
                                \n            <variable name=\"a\"/>\
                                \n            <variable name=\"b\"/>\
                                \n        </operation>\
                                \n    </return-statement>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        node.serialize(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }
}

#[test]
fn functions_test() {
    static SRC_TEXT: &str = "\nfunc f(a: i32, b: i32) -> f32 {\
                             \n    return a + b\
                             \n}\
                             \n\
                             \nfunc main() {\
                             \n   let res = f(a: 1, 2, c: 1 + 2)\
                             \n}\
                             \n\
                             \nfunc bar () {\
                             \n   let PI = 3.14\
                             \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT, true).expect("Lexer creation failed"));

    {
        let func = FunctionDef::parse(&mut parser).unwrap();

        let scope = if let Ast::FuncDef { node } = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let Ast::Scope { node } = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        assert!(matches!(node, Ast::ReturnStmt { .. }));
    }

    {
        let func = FunctionDef::parse(&mut parser).unwrap();

        let scope = if let Ast::FuncDef { node } = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let Ast::Scope { node } = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        let (lhs, rhs) = if let Ast::Expression { node } = node {
            if let ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } = &node.as_ref().expr
            {
                assert_eq!(*operation, Lexem::Assign);
                (lhs.as_ref(), rhs.as_ref())
            } else {
                panic!("Expression expected to be binary");
            }
        } else {
            panic!("Expected expression");
        };

        assert!(matches!(lhs, Ast::VariableDef { .. }));
        assert!(matches!(rhs, Ast::Value { .. }));
    }
}
