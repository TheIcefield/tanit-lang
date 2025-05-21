use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::xml_writer::XmlWriter;

use pretty_assertions::assert_str_eq;

#[test]
fn struct_with_methods_test() {
    const SRC_TEXT: &str = "\nstruct MyStruct\
                            \n{\
                            \n    f1: i32\
                            \n    f2: f32\
                            \n}\
                            \nimpl MyStruct\
                            \n{\
                            \n    func new(): MyStruct {\
                            \n        return MyStruct {\
                            \n                 f1: 0, f2: 0.0\
                            \n               }\
                            \n    }
                            \n}\
                            \nfunc main() {\
                            \n    var s = MyStruct { \
                            \n              f1: 1, f2: 2.0\
                            \n            }\
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
        const EXPECTED: &str = "\n<struct-definition name=\"MyStruct\">\
                                \n    <field name=\"f1\" publicity=\"Private\">\
                                \n        <type style=\"primitive\" name=\"i32\"/>\
                                \n    </field>\
                                \n    <field name=\"f2\" publicity=\"Private\">\
                                \n        <type style=\"primitive\" name=\"f32\"/>\
                                \n    </field>\
                                \n</struct-definition>\
                                \n<impl-definition name=\"MyStruct\">\
                                \n    <function-definition name=\"MyStruct__new\">\
                                \n        <return-type>\
                                \n            <type style=\"named\" name=\"MyStruct\"/>\
                                \n        </return-type>\
                                \n        <return-statement>\
                                \n            <struct-initialization name=\"MyStruct\">\
                                \n                <field name=\"f1\">\
                                \n                    <literal style=\"integer-number\" value=\"0\"/>\
                                \n                </field>\
                                \n                <field name=\"f2\">\
                                \n                    <literal style=\"decimal-number\" value=\"0\"/>\
                                \n                </field>\
                                \n            </struct-initialization>\
                                \n        </return-statement>\
                                \n    </function-definition>\
                                \n</impl-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"s\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"named\" name=\"MyStruct\"/>\
                                \n        </variable-definition>\
                                \n        <struct-initialization name=\"MyStruct\">\
                                \n            <field name=\"f1\">\
                                \n                <literal style=\"integer-number\" value=\"1\"/>\
                                \n            </field>\
                                \n            <field name=\"f2\">\
                                \n                <literal style=\"decimal-number\" value=\"2\"/>\
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
        const HEADER_EXPECTED: &str = "typedef struct {\
                                        \nsigned int f1;\
                                        \nfloat f2;\
                                     \n} MyStruct;\
                                     \nMyStruct MyStruct__new();\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "MyStruct MyStruct__new(){\
                                        \nreturn (MyStruct){\
                                            \n.f1=0,\
                                            \n.f2=0,\
                                        \n};\
                                    \n}\
                                    \nvoid main(){\
                                     \nMyStruct const s = (MyStruct){\
                                     \n.f1=1,\
                                     \n.f2=2,\
                                     \n};\
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
