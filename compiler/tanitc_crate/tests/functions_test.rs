use tanitc_analyzer::Analyzer;
use tanitc_ast::{Ast, ControlFlow, ControlFlowKind, ExpressionKind};
use tanitc_codegen::CodeGenStream;
use tanitc_lexer::{token::Lexem, Lexer};
use tanitc_parser::Parser;
use tanitc_serializer::XmlWriter;

#[test]
fn function_def_test() {
    const SRC_TEXT: &str = "\nfunc sum(a: f32, b: f32): f32 {\
                            \n    return a + b\
                            \n}\
                            \nfunc main() {\
                            \n    var ret: f32 = sum(a, b)\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let node = parser.parse_global_block().unwrap();

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
                                \n            <identifier name=\"a\"/>\
                                \n            <identifier name=\"b\"/>\
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
                                \n                    <identifier name=\"a\"/>\
                                \n                </parameter>\
                                \n                <parameter index=\"1\">\
                                \n                    <identifier name=\"b\"/>\
                                \n                </parameter>\
                                \n            </parameters>\
                                \n        </call-statement>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        node.accept(&mut writer).unwrap();
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

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert_eq!(SOURCE_EXPECTED, source_res);
    }
}

#[test]
fn functions_test() {
    const SRC_TEXT: &str = "\nfunc f(a: i32, b: i32): f32 {\
                            \n    return a + b\
                            \n}\
                            \n\
                            \nfunc main() {\
                            \n   var res = f(a: 1, 2, c: 1 + 2)\
                            \n}\
                            \n\
                            \nfunc bar () {\
                            \n   var PI = 3.14\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    {
        let func = parser.parse_func_def().unwrap();

        let scope = if let Ast::FuncDef(node) = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let Ast::Block(node) = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        assert!(matches!(
            node,
            Ast::ControlFlow(ControlFlow {
                kind: ControlFlowKind::Return { .. },
                ..
            })
        ));
    }

    {
        let func = parser.parse_func_def().unwrap();

        let scope = if let Ast::FuncDef(node) = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let Ast::Block(node) = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        let (lhs, rhs) = if let Ast::Expression(node) = node {
            if let ExpressionKind::Binary {
                operation,
                lhs,
                rhs,
            } = &node.kind
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

    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }
}

#[test]
fn function_in_module_work_test() {
    const SRC_TEXT: &str = "\nmodule color {\
                            \n    enum Color {\
                            \n        Red\
                            \n        Green\
                            \n        Blue\
                            \n    }\
                            \n    func get_green(): Color {\
                            \n        var ret = Color::Green\
                            \n        return ret\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var green = color::get_green()\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:?}", analyzer.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<module-definition name=\"color\">\
                                \n    <enum-definition name=\"Color\">\
                                \n        <field name=\"Red\" value=\"0\"/>\
                                \n        <field name=\"Green\" value=\"1\"/>\
                                \n        <field name=\"Blue\" value=\"2\"/>\
                                \n    </enum-definition>\
                                \n    <function-definition name=\"get_green\">\
                                \n        <return-type>\
                                \n            <type style=\"named\" name=\"Color\"/>\
                                \n        </return-type>\
                                \n        <operation style=\"binary\" operation=\"=\">\
                                \n            <variable-definition name=\"ret\" is-global=\"false\" is-mutable=\"false\">\
                                \n                <type style=\"named\" name=\"Color\"/>\
                                \n            </variable-definition>\
                                \n            <operation>\
                                \n                <literal style=\"integer-number\" value=\"1\"/>\
                                \n            </operation>\
                                \n        </operation>\
                                \n        <return-statement>\
                                \n            <identifier name=\"ret\"/>\
                                \n        </return-statement>\
                                \n    </function-definition>\
                                \n</module-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"green\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"tuple\"/>\
                                \n        </variable-definition>\
                                \n        <operation style=\"access\">\
                                \n            <identifier name=\"color\"/>\
                                \n            <call-statement name=\"get_green\">\
                                \n                <parameters/>\
                                \n            </call-statement>\
                                \n        </operation>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        program.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    Red = 0,\
                                     \n    Green = 1,\
                                     \n    Blue = 2,\
                                     \n} Color;\
                                     \nColor get_green();\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "Color get_green(){\
                                     \nColor const ret = 1;\
                                     \nreturn ret;\
                                     \n}\
                                     \nvoid main(){\
                                     \nvoid const green = color.get_green();\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let mut res = String::from_utf8(header_buffer).unwrap();
        assert_eq!(HEADER_EXPECTED, res);

        res = String::from_utf8(source_buffer).unwrap();
        assert_eq!(SOURCE_EXPECTED, res);
    }
}
