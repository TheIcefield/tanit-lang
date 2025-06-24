use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::xml_writer::XmlWriter;

use pretty_assertions::assert_str_eq;

#[test]
fn array_parse_test() {
    const SRC_TEXT: &str = "\nfunc main() {\
                            \n    var arr_1: [f32: 6]\
                            \n    var arr_2: [i32: 3] = [4, 5, 6]\
                            \n    var arr_3 = [1.0, 2.0, 3.0]\
                            \n    arr_1[1 + 1] = 7.0\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Failed to create lexer"));

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
        const EXPECTED: &str = "\n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <variable-definition name=\"arr_1\" is-global=\"false\" is-mutable=\"false\">\
                                \n        <type style=\"array\" size=\"6\" style=\"primitive\" name=\"f32\"/>\
                                \n    </variable-definition>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"arr_2\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"array\" size=\"3\" style=\"primitive\" name=\"i32\"/>\
                                \n        </variable-definition>\
                                \n        <array-initialization>\
                                \n            <literal style=\"integer-number\" value=\"4\"/>\
                                \n            <literal style=\"integer-number\" value=\"5\"/>\
                                \n            <literal style=\"integer-number\" value=\"6\"/>\
                                \n        </array-initialization>\
                                \n    </operation>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"arr_3\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"array\" size=\"3\" style=\"primitive\" name=\"f32\"/>\
                                \n        </variable-definition>\
                                \n        <array-initialization>\
                                \n            <literal style=\"decimal-number\" value=\"1\"/>\
                                \n            <literal style=\"decimal-number\" value=\"2\"/>\
                                \n            <literal style=\"decimal-number\" value=\"3\"/>\
                                \n        </array-initialization>\
                                \n    </operation>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <operation style=\"indexing\">\
                                \n            <identifier name=\"arr_1\"/>\
                                \n            <operation style=\"binary\" operation=\"+\">\
                                \n                <literal style=\"integer-number\" value=\"1\"/>\
                                \n                <literal style=\"integer-number\" value=\"1\"/>\
                                \n            </operation>\
                                \n        </operation>\
                                \n        <literal style=\"decimal-number\" value=\"7\"/>\
                                \n    </operation>\
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
                                        \nfloat const arr_1[6];\
                                        \nsigned int const arr_2[3] = { 4, 5, 6 };\
                                        \nfloat const arr_3[3] = { 1, 2, 3 };\
                                        \narr_1[1 + 1] = 7;\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_str_eq!(HEADER_EXPECTED, header_res);
        assert_str_eq!(SOURCE_EXPECTED, source_res);
    }
}
