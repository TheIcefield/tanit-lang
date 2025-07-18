use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::xml_writer::XmlWriter;

use pretty_assertions::assert_str_eq;

#[test]
fn extern_test() {
    const SRC_TEXT: &str = "\nextern \"C\" {\
                            \n    func hello(): i32
                            \n}\
                            \nfunc main() {\
                            \n    var res = hello()\
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
            panic!("{:#?}", analyzer.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<extern-definition abi-name=\"C\">\
                                \n    <function-definition name=\"hello\">\
                                \n        <return-type>\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </return-type>\
                                \n    </function-definition>\
                                \n</extern-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"res\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </variable-definition>\
                                \n        <call-statement name=\"hello\"/>\
                                \n    </operation>\
                                \n</function-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        program.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "signed int hello();\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                         \nsigned int const res = hello();\
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
