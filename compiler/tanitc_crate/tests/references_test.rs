use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_parser::Parser;

use pretty_assertions::assert_str_eq;

#[test]
fn immutable_deref_test() {
    const SRC_TEXT: &str = "\nfunc main() {\
                            \n    var value = 50\
                            \n    var ref = &value\
                            \n    if 1 {\
                            \n        *ref = 10\
                            \n    }\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    {
        const EXPECTED_ERROR_TEXT: &str =
            "Semantic error: Reference \"ref\" is immutable in current scope";

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_str_eq!(
            errors.first().expect("Expected errors").text,
            EXPECTED_ERROR_TEXT
        );
    }
}

#[test]
fn mutable_deref_test() {
    const SRC_TEXT: &str = "\nfunc main() {\
                            \n    var mut value = 50\
                            \n    var ref = &mut value\
                            \n    if 1 {\
                            \n        *ref = 10\
                            \n    }\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    {
        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    {
        const HEADER_EXPECTED: &str = "void main();\n";
        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
                                     \n    signed int value = 50;\
                                     \n    signed int * const ref = &value;\
                                     \n    if (1)\
                                     \n    {\
                                     \n        *ref = 10;\
                                     \n    }\
                                     \n\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, header_res);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, source_res);
    }
}

#[test]
fn mutable_ref_to_immutable_var_test() {
    const SRC_TEXT: &str = "\nfunc main() {\
                            \n    var value = 50\
                            \n    var ref = &mut value\
                            \n    if 1 {\
                            \n        *ref = 10\
                            \n    }\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    {
        const EXPECTED_ERROR_TEXT: &str =
            "Semantic error: Mutable reference to immutable variable \"value\"";

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_str_eq!(
            errors.first().expect("Expected errors").text,
            EXPECTED_ERROR_TEXT
        );
    }
}

#[test]
fn immutable_deref_param_test() {
    const SRC_TEXT: &str = "\nfunc bar(p: &i32) {\
                            \n    *p = 10 # expected error\
                            \n}\
                            \nfunc main() {\
                            \n    var value = 50\
                            \n    bar(&value)\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    {
        const EXPECTED_ERROR_TEXT: &str =
            "Semantic error: Reference \"p\" is immutable in current scope";

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_str_eq!(
            errors.first().expect("Expected errors").text,
            EXPECTED_ERROR_TEXT
        );
    }
}

#[test]
fn mutable_deref_param_test() {
    const SRC_TEXT: &str = "\nfunc bar(p: &mut i32) {\
                            \n    *p = 10\
                            \n}\
                            \nfunc main() {\
                            \n    var mut value = 50\
                            \n    bar(&mut value)\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    {
        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    {
        const HEADER_EXPECTED: &str = "void bar(signed int * const p);\
                                     \nvoid main();\n";
        const SOURCE_EXPECTED: &str = "void bar(signed int * const p)\
                                     \n{\
                                     \n    *p = 10;\
                                     \n}\
                                     \nvoid main()\
                                     \n{\
                                     \n    signed int value = 50;\
                                     \n    bar(&value);\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, header_res);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(SOURCE_EXPECTED, source_res);
    }
}
