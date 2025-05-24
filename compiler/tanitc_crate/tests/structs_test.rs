use tanitc_analyzer::Analyzer;
use tanitc_codegen::CodeGenStream;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::xml_writer::XmlWriter;

#[test]
fn struct_work_test() {
    const SRC_TEXT: &str = "\nstruct MyStruct\
                            \n{\
                            \n    f1: i32\
                            \n    f2: f32\
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
                                \n    <field name=\"f1\">\
                                \n        <type style=\"primitive\" name=\"i32\"/>\
                                \n    </field>\
                                \n    <field name=\"f2\">\
                                \n        <type style=\"primitive\" name=\"f32\"/>\
                                \n    </field>\
                                \n</struct-definition>\
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

        assert_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \nsigned int f1;\
                                     \nfloat f2;\
                                     \n} MyStruct;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
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
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert_eq!(SOURCE_EXPECTED, source_res);
    }
}

#[test]
fn struct_in_module_work_test() {
    const SRC_TEXT: &str = "\nmodule math {\
                            \n    struct Vector2 {\
                            \n        x: f32\
                            \n        y: f32\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var vec = math::Vector2 { \
                            \n              x: 0.0, y: 2.0\
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
        const EXPECTED: &str = "\n<module-definition name=\"math\">\
                                \n    <struct-definition name=\"Vector2\">\
                                \n        <field name=\"x\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </field>\
                                \n        <field name=\"y\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </field>\
                                \n    </struct-definition>\
                                \n</module-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"vec\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"named\" name=\"Vector2\"/>\
                                \n        </variable-definition>\
                                \n        <operation>\
                                \n            <struct-initialization name=\"Vector2\">\
                                \n                <field name=\"x\">\
                                \n                    <literal style=\"decimal-number\" value=\"0\"/>\
                                \n                </field>\
                                \n                <field name=\"y\">\
                                \n                    <literal style=\"decimal-number\" value=\"2\"/>\
                                \n                </field>\
                                \n            </struct-initialization>\
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
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \nfloat x;\
                                     \nfloat y;\
                                     \n} Vector2;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                     \nVector2 const vec = (Vector2){\
                                     \n.x=0,\
                                     \n.y=2,\
                                     \n};\
                                     \n}\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        program.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert_eq!(SOURCE_EXPECTED, source_res);
    }
}
