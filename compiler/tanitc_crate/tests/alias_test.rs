use tanitc_analyzer::Analyzer;
use tanitc_ast::Ast;

use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_ident::Ident;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::xml_writer::XmlWriter;
use tanitc_ty::Type;

use pretty_assertions::assert_str_eq;

#[test]
fn alias_def_test() {
    const SRC_TEXT: &str = "alias MyAlias = f32";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let alias_node = parser.parse_alias_def().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<alias-definition name=\"MyAlias\">\
                                \n    <type style=\"primitive\" name=\"f32\"/>\
                                \n</alias-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        alias_node.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef float MyAlias;\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        alias_node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert!(source_res.is_empty());
    }
}

#[test]
fn alias_in_func_test() {
    const SRC_TEXT: &str = "func main() : ()\
                            {\
                                alias Items = Vec<Item>\
                            }";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let res = if let Ast::FuncDef(node) = parser.parse_func_def().unwrap() {
        assert!(node.identifier == Ident::from("main".to_string()));
        assert!(node.parameters.is_empty());

        if let Type::Tuple(components) = &node.return_type.get_type() {
            assert!(components.is_empty());
        } else {
            panic!("Type expected to be an empty tuple");
        }

        node.body.unwrap()
    } else {
        panic!("res has to be \'function definition\'");
    };

    let statements = if let Ast::Block(node) = res.as_ref() {
        &node.statements
    } else {
        panic!("node has to be \'local scope\'");
    };

    if let Ast::AliasDef(node) = &statements[0] {
        assert!(node.identifier == Ident::from("Items".to_string()));

        if let Type::Template {
            identifier,
            generics,
        } = &node.value.get_type()
        {
            assert!(*identifier == Ident::from("Vec".to_string()));
            assert_eq!(generics.len(), 1);
            if let Type::Custom(id) = &generics[0] {
                assert_eq!(id, "Item");
            } else {
                panic!("Type is expected to be \"Item\"")
            }
        } else {
            panic!("Alias type expected to be an template type");
        }
    } else {
        panic!("res has to be \'alias definition\'");
    };
}

#[test]
fn alias_test() {
    const SRC_TEXT: &str = "\npub alias VecUnit = f32\
                            \npub struct Vec2 {\
                            \n    x: VecUnit\
                            \n    y: VecUnit\
                            \n}\
                            \nalias Vec = Vec2\
                            \nfunc main() {\
                            \n    var v = Vec { x: 10.0, y: 10.0 }\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<alias-definition name=\"VecUnit\">\
                                \n    <attributes publicity=\"Public\"/>\
                                \n    <type style=\"primitive\" name=\"f32\"/>\
                                \n</alias-definition>\
                                \n<struct-definition name=\"Vec2\">\
                                \n    <attributes publicity=\"Public\"/>\
                                \n    <field name=\"x\" publicity=\"Private\">\
                                \n        <type style=\"named\" name=\"VecUnit\"/>\
                                \n    </field>\
                                \n    <field name=\"y\" publicity=\"Private\">\
                                \n        <type style=\"named\" name=\"VecUnit\"/>\
                                \n    </field>\
                                \n</struct-definition>\
                                \n<alias-definition name=\"Vec\">\
                                \n    <type style=\"named\" name=\"Vec2\"/>\
                                \n</alias-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"v\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"named\" name=\"Vec\"/>\
                                \n        </variable-definition>\
                                \n        <struct-initialization name=\"Vec\">\
                                \n            <field name=\"x\">\
                                \n                <literal style=\"decimal-number\" value=\"10\"/>\
                                \n            </field>\
                                \n            <field name=\"y\">\
                                \n                <literal style=\"decimal-number\" value=\"10\"/>\
                                \n            </field>\
                                \n        </struct-initialization>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        program.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef float VecUnit;\
                                     \ntypedef struct {\
                                     \n    VecUnit x;\
                                     \n    VecUnit y;\
                                     \n} Vec2;\
                                     \ntypedef Vec2 Vec;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                     \n    Vec const v = (Vec)\
                                     \n    {\
                                     \n        .x=10,\
                                     \n        .y=10,\
                                     \n    };\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let mut res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, res);

        res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, res);
    }
}

#[test]
fn incorrect_alias_object_test() {
    const SRC_TEXT: &str = "\nalias Vec = i32\
                            \nfunc main() {\
                            \n    var v = Vec { x: 10.0, y: 10.0 }\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        const EXPECTED: &str = "Semantic error: Common type \"i32\" does not have any fields";

        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_str_eq!(errors.first().expect("Expected error").text, EXPECTED);
    }
}

#[test]
fn alias_common_type_test() {
    const SRC_TEXT: &str = "\nalias A = i32\
                            \nfunc main() {\
                            \n    var a: A = 100\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<alias-definition name=\"A\">\
                                \n    <type style=\"primitive\" name=\"i32\"/>\
                                \n</alias-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"a\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"named\" name=\"A\"/>\
                                \n        </variable-definition>\
                                \n        <literal style=\"integer-number\" value=\"100\"/>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        program.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef signed int A;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                     \n    A const a = 100;\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let mut res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, res);

        res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, res);
    }
}

#[test]
fn incorrect_alias_common_type_test() {
    const SRC_TEXT: &str = "\nalias A = i32\
                            \nfunc main() {\
                            \n    var a: A = 3.14\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        const EXPECTED: &str = "Semantic error: Cannot perform operation on objects with different types: A (aka: i32) and f32";

        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_str_eq!(errors.first().expect("Expected error").text, EXPECTED);
    }
}

#[test]
fn alias_custom_type_test() {
    const SRC_TEXT: &str = "\nstruct S {}\
                            \nalias A = S\
                            \nfunc main() {\
                            \n    var a: A = S {}\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<struct-definition name=\"S\"/>\
                                \n<alias-definition name=\"A\">\
                                \n    <type style=\"named\" name=\"S\"/>\
                                \n</alias-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"a\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"named\" name=\"A\"/>\
                                \n        </variable-definition>\
                                \n        <struct-initialization name=\"S\"/>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        program.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} S;\
                                     \ntypedef S A;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                     \n    A const a = (S) { };\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let mut res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, res);

        res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, res);
    }
}

#[test]
fn incorrect_alias_custom_type_test() {
    const SRC_TEXT: &str = "\nstruct S {}\
                            \nalias A = S\
                            \nfunc main() {\
                            \n    var a: A = 100\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        const EXPECTED: &str = "Semantic error: Cannot perform operation on objects with different types: A (aka: S) and i32";

        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_str_eq!(errors.first().expect("Expected error").text, EXPECTED);
    }
}

#[test]
fn alias_to_alias_type_test() {
    const SRC_TEXT: &str = "\nstruct S {}\
                            \nalias A = S\
                            \nalias B = A\
                            \nfunc main() {\
                            \n    var b: B = S {}\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<struct-definition name=\"S\"/>\
                                \n<alias-definition name=\"A\">\
                                \n    <type style=\"named\" name=\"S\"/>\
                                \n</alias-definition>\
                                \n<alias-definition name=\"B\">\
                                \n    <type style=\"named\" name=\"A\"/>\
                                \n</alias-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"b\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"named\" name=\"B\"/>\
                                \n        </variable-definition>\
                                \n        <struct-initialization name=\"S\"/>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        program.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} S;\
                                     \ntypedef S A;\
                                     \ntypedef A B;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                     \n    B const b = (S) { };\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let mut res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, res);

        res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, res);
    }
}

#[test]
fn incorrect_alias_to_alias_type_test() {
    const SRC_TEXT: &str = "\nstruct S {}\
                            \nalias A = S\
                            \nalias B = A\
                            \nfunc main() {\
                            \n    var b: B = 50\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    let mut analyzer = Analyzer::new();
    {
        const EXPECTED: &str = "Semantic error: Cannot perform operation on objects with different types: B (aka: S) and i32";

        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_str_eq!(errors.first().expect("Expected error").text, EXPECTED);
    }
}
