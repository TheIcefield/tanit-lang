use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::xml_writer::XmlWriter;

use pretty_assertions::assert_str_eq;

#[test]
fn struct_work_test() {
    const SRC_TEXT: &str = "\nstruct MyStruct\
                            \n{\
                            \n    pub f1: i32\
                            \n    f2: [f32: 2]\
                            \n}\
                            \nfunc main() {\
                            \n    var mut s = MyStruct { \
                            \n              f1: 1, f2: [2.0, 3.0]\
                            \n            }\
                            \n    s.f1 = 2\
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
                                \n    <field name=\"f1\" publicity=\"Public\">\
                                \n        <type style=\"primitive\" name=\"i32\"/>\
                                \n    </field>\
                                \n    <field name=\"f2\" publicity=\"Private\">\
                                \n        <type style=\"array\">\
                                \n            <size value=\"2\"/>\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </type>\
                                \n    </field>\
                                \n</struct-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"s\" is-global=\"false\" mutability=\"Mutable\">\
                                \n            <type style=\"named\" name=\"MyStruct\"/>\
                                \n        </variable-definition>\
                                \n        <struct-initialization name=\"MyStruct\">\
                                \n            <field name=\"f1\">\
                                \n                <literal style=\"integer-number\" value=\"1\"/>\
                                \n            </field>\
                                \n            <field name=\"f2\">\
                                \n                <array-initialization>\
                                \n                    <literal style=\"decimal-number\" value=\"2\"/>\
                                \n                    <literal style=\"decimal-number\" value=\"3\"/>\
                                \n                </array-initialization>\
                                \n            </field>\
                                \n        </struct-initialization>\
                                \n    </operation>\
                                \n    <operation style=\"get\">\
                                \n        <identifier name=\"s\"/>\
                                \n        <operation style=\"binary\" operation=\"=\">\
                                \n            <identifier name=\"f1\"/>\
                                \n            <literal style=\"integer-number\" value=\"2\"/>\
                                \n        </operation>\
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
                                     \n    signed int f1;\
                                     \n    float f2;\
                                     \n} MyStruct;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                     \n    MyStruct s = (MyStruct)\
                                     \n    {\
                                     \n        .f1=1,\
                                     \n        .f2={ 2, 3 },\
                                     \n    };\
                                     \n    s.f1 = 2;\
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

#[test]
fn struct_in_module_work_test() {
    const SRC_TEXT: &str = "\nmodule math {\
                            \n    struct Vector2 {\
                            \n        x: f32\
                            \n        y: f32\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var mut vec = math::Vector2 { \
                            \n              x: 0.0, y: 2.0\
                            \n            }\
                            \n    vec.x = 2.0\
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
                                \n        <field name=\"x\" publicity=\"Private\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </field>\
                                \n        <field name=\"y\" publicity=\"Private\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </field>\
                                \n    </struct-definition>\
                                \n</module-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"vec\" is-global=\"false\" mutability=\"Mutable\">\
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
                                \n    <operation style=\"get\">\
                                \n        <identifier name=\"vec\"/>\
                                \n        <operation style=\"binary\" operation=\"=\">\
                                \n            <identifier name=\"x\"/>\
                                \n            <literal style=\"decimal-number\" value=\"2\"/>\
                                \n        </operation>\
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
                                     \n    float x;\
                                     \n    float y;\
                                     \n} Vector2;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                     \n    Vector2 vec = (Vector2)\
                                     \n    {\
                                     \n        .x=0,\
                                     \n        .y=2,\
                                     \n    };\
                                     \n    vec.x = 2;\
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
fn incorrect_struct_work_test() {
    const SRC_TEXT: &str = "\nstruct MyStruct\
                            \n{\
                            \n    f1: i32\
                            \n    f2: f32\
                            \n}\
                            \nfunc main() {\
                            \n    var mut s = MyStruct { \
                            \n              f1: 1, f2: 2.0\
                            \n            }\
                            \n    s.f1 = 3.0\
                            \n    s.f2 = 2.0\
                            \n    s.f3 = 1.0\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Failed to create lexer"));

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        const EXPECTED_1: &str =
            "Semantic error: Cannot perform operation on objects with different types: i32 and f32";
        const EXPECTED_2: &str = "Semantic error: \"s\" doesn't have member named \"f3\"";

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 2);
        assert_str_eq!(errors[0].text, EXPECTED_1);
        assert_str_eq!(errors[1].text, EXPECTED_2);
    }
}

#[test]
fn internal_struct_work_test() {
    const SRC_TEXT: &str = "\nmodule math {\
                            \n    struct Unit {\
                            \n        value: f32\
                            \n    }\
                            \n    struct Point2 {\
                            \n        x: Unit\
                            \n        y: Unit\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var mut pnt = math::Point2 { \
                            \n                  x: Unit { value: 1.0 },\
                            \n                  y: Unit { value: 2.0 },\
                            \n            }\
                            \n    pnt.x.value = 2.0\
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
                                \n    <struct-definition name=\"Unit\">\
                                \n        <field name=\"value\" publicity=\"Private\">\
                                \n            <type style=\"primitive\" name=\"f32\"/>\
                                \n        </field>\
                                \n    </struct-definition>\
                                \n    <struct-definition name=\"Point2\">\
                                \n        <field name=\"x\" publicity=\"Private\">\
                                \n            <type style=\"named\" name=\"Unit\"/>\
                                \n        </field>\
                                \n        <field name=\"y\" publicity=\"Private\">\
                                \n            <type style=\"named\" name=\"Unit\"/>\
                                \n        </field>\
                                \n    </struct-definition>\
                                \n</module-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"pnt\" is-global=\"false\" mutability=\"Mutable\">\
                                \n            <type style=\"named\" name=\"Point2\"/>\
                                \n        </variable-definition>\
                                \n        <operation>\
                                \n            <struct-initialization name=\"Point2\">\
                                \n                <field name=\"x\">\
                                \n                    <struct-initialization name=\"Unit\">\
                                \n                        <field name=\"value\">\
                                \n                            <literal style=\"decimal-number\" value=\"1\"/>\
                                \n                        </field>\
                                \n                    </struct-initialization>\
                                \n                </field>\
                                \n                <field name=\"y\">\
                                \n                    <struct-initialization name=\"Unit\">\
                                \n                        <field name=\"value\">\
                                \n                            <literal style=\"decimal-number\" value=\"2\"/>\
                                \n                        </field>\
                                \n                    </struct-initialization>\
                                \n                </field>\
                                \n            </struct-initialization>\
                                \n        </operation>\
                                \n    </operation>\
                                \n    <operation style=\"get\">\
                                \n        <identifier name=\"pnt\"/>\
                                \n        <operation style=\"get\">\
                                \n            <identifier name=\"x\"/>\
                                \n            <operation style=\"binary\" operation=\"=\">\
                                \n                <identifier name=\"value\"/>\
                                \n                <literal style=\"decimal-number\" value=\"2\"/>\
                                \n            </operation>\
                                \n        </operation>\
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
                                     \n    float value;\
                                     \n} Unit;\
                                     \ntypedef struct {\
                                     \n    Unit x;\
                                     \n    Unit y;\
                                     \n} Point2;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main(){\
                                     \n    Point2 pnt = (Point2)\
                                     \n    {\
                                     \n        .x=(Unit)\
                                     \n        {\
                                     \n            .value=1,\
                                     \n        },\
                                     \n        .y=(Unit)\
                                     \n        {\
                                     \n            .value=2,\
                                     \n        },\
                                     \n    };\
                                     \n    pnt.x.value = 2;\
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
