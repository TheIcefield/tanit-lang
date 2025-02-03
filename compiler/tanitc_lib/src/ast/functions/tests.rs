use super::FunctionDef;
use crate::ast::{expressions::ExpressionType, scopes::Scope, Ast};
use crate::codegen::CodeGenStream;
use crate::parser::Parser;
use crate::serializer::XmlWriter;

use tanitc_lexer::{token::Lexem, Lexer};

#[test]
fn function_def_test() {
    const SRC_TEXT: &str = "\nfunc sum(a: f32, b: f32) -> f32 {\
                            \n    return a + b\
                            \n}\
                            \nfunc main() {\
                            \n    let ret: f32 = sum(a, b)\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let node = Scope::parse_global(&mut parser).unwrap();

    {
        const EXPECTED: &str = "\n<function-definition name=\"sum\">\
                                \n    <return-type>\
                                \n        <type style=\"primitive\" name=\"f32\"/>\
                                \n    </return-type>\
                                \n    <parameters>\
                                \n        <variable-definition name=\"a\" is-global=\"false\" is-mutable=\"true\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </variable-definition>\
                                \n        <variable-definition name=\"b\" is-global=\"false\" is-mutable=\"true\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </variable-definition>\
                                \n    </parameters>\
                                \n    <return-statement>\
                                \n        <operation style=\"binary\" operation=\"+\">\
                                \n            <variable name=\"a\"/>\
                                \n            <variable name=\"b\"/>\
                                \n        </operation>\
                                \n    </return-statement>\
                                \n</function-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"ret\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </variable-definition>\
                                \n        <call-statement name=\"sum\">\
                                \n            <parameters>\
                                \n                <parameter index=\"0\">\
                                \n                    <variable name=\"a\"/>\
                                \n                </parameter>\
                                \n                <parameter index=\"1\">\
                                \n                    <variable name=\"b\"/>\
                                \n                </parameter>\
                                \n            </parameters>\
                                \n        </call-statement>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        node.serialize(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "float sum(float a, float b);\
                                     \nvoid main();\n";
        const SOURCE_EXPECTED: &str = "float sum(float a, float b){\
                                     \nreturn a + b;\
                                     \n}\
                                     \nvoid main(){\
                                     \nfloat const ret = sum(a, b);\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.codegen(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert_eq!(SOURCE_EXPECTED, source_res);
    }
}

#[test]
fn functions_test() {
    const SRC_TEXT: &str = "\nfunc f(a: i32, b: i32) -> f32 {\
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

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    {
        let func = FunctionDef::parse(&mut parser).unwrap();

        let scope = if let Ast::FuncDef(node) = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let Ast::Scope(node) = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        assert!(matches!(node, Ast::ReturnStmt { .. }));
    }

    {
        let func = FunctionDef::parse(&mut parser).unwrap();

        let scope = if let Ast::FuncDef(node) = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let Ast::Scope(node) = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        let (lhs, rhs) = if let Ast::Expression(node) = node {
            if let ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } = &node.expr
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
