use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::xml_writer::XmlWriter;

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

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

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

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

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
        const EXPECTED: &str = "\n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"value\" is-global=\"false\" is-mutable=\"true\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </variable-definition>\
                                \n        <literal style=\"integer-number\" value=\"50\"/>\
                                \n    </operation>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"ref\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"reference\" is-mutable=\"true\" style=\"primitive\" name=\"i32\"/>\
                                \n        </variable-definition>\
                                \n        <operation style=\"unary\" operation=\"&mut\">\
                                \n            <identifier name=\"value\"/>\
                                \n        </operation>\
                                \n    </operation>\
                                \n    <if>\
                                \n        <condition>\
                                \n            <literal style=\"integer-number\" value=\"1\"/>\
                                \n        </condition>\
                                \n        <than>\
                                \n            <operation style=\"unary\" operation=\"*\">\
                                \n                <operation style=\"binary\" operation=\"=\">\
                                \n                    <identifier name=\"ref\"/>\
                                \n                    <literal style=\"integer-number\" value=\"10\"/>\
                                \n                </operation>\
                                \n            </operation>\
                                \n        </than>\
                                \n    </if>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        program.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "void main();\n";
        const SOURCE_EXPECTED: &str = "void main(){\
                                        \nsigned int value = 50;\
                                        \nsigned int const * const ref = &mut value;\
                                        \nif (1)\
                                        \n{\
                                            \n*ref = 10;\
                                        \n}\
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

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

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
