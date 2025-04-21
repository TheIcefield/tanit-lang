use tanitc_analyzer::Analyzer;
use tanitc_codegen::CodeGenStream;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::XmlWriter;

#[test]
fn union_work_test() {
    const SRC_TEXT: &str = "\nunion MyUnion\
                            \n{\
                            \n    f1: i32\
                            \n    f2: f32\
                            \n}\
                            \nfunc main() {\
                            \n    var s = MyUnion { \
                            \n              f2: 2.0\
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
        const EXPECTED: &str = "\n<union-definition name=\"MyUnion\">\
                                \n    <field name=\"f1\">\
                                \n        <type style=\"primitive\" name=\"i32\"/>\
                                \n    </field>\
                                \n    <field name=\"f2\">\
                                \n        <type style=\"primitive\" name=\"f32\"/>\
                                \n    </field>\
                                \n</union-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"s\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"named\" name=\"MyUnion\"/>\
                                \n        </variable-definition>\
                                \n        <struct-initialization name=\"MyUnion\">\
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
        const HEADER_EXPECTED: &str = "typedef union {\
                                     \nsigned int f1;\
                                     \nfloat f2;\
                                     \n} MyUnion;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                     \nMyUnion const s = (MyUnion){\
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
fn union_in_module_work_test() {
    const SRC_TEXT: &str = "\nmodule mod {\
                            \n    union MyUnion {\
                            \n        x: i32\
                            \n        y: f32\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var u = mod::MyUnion { \
                            \n                  y: 2.0\
                            \n              }\
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
        const EXPECTED: &str = "\n<module-definition name=\"mod\">\
                                \n    <union-definition name=\"MyUnion\">\
                                \n        <field name=\"x\">\
                                \n            <type style=\"primitive\" name=\"i32\"/>\
                                \n        </field>\
                                \n        <field name=\"y\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </field>\
                                \n    </union-definition>\
                                \n</module-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"u\" is-global=\"false\" is-mutable=\"false\">\
                                \n            <type style=\"named\" name=\"MyUnion\"/>\
                                \n        </variable-definition>\
                                \n        <operation>\
                                \n            <struct-initialization name=\"MyUnion\">\
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
        const HEADER_EXPECTED: &str = "typedef union {\
                                     \nsigned int x;\
                                     \nfloat y;\
                                     \n} MyUnion;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                     \nMyUnion const u = (MyUnion){\
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
