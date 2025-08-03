use tanitc_analyzer::Analyzer;

use tanitc_ast::ast::Ast;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_ident::Ident;
use tanitc_parser::Parser;
use tanitc_ty::Type;

use pretty_assertions::assert_str_eq;

#[test]
fn alias_def_test() {
    const SRC_TEXT: &str = "alias MyAlias = f32";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let alias_node = parser.parse_alias_def().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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

    let statements = &res.as_ref().statements;

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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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
        const HEADER_EXPECTED: &str = "typedef float VecUnit;\
                                     \ntypedef struct {\
                                     \n    VecUnit x;\
                                     \n    VecUnit y;\
                                     \n} Vec2;\
                                     \ntypedef Vec2 Vec;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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
        const HEADER_EXPECTED: &str = "typedef signed int A;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} S;\
                                     \ntypedef S A;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \n} S;\
                                     \ntypedef S A;\
                                     \ntypedef A B;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
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

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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
