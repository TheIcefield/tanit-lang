use tanitc_analyzer::Analyzer;
use tanitc_codegen::c_generator::CodeGenStream;
use tanitc_parser::Parser;
use tanitc_serializer::xml_writer::XmlWriter;

use pretty_assertions::assert_str_eq;

#[test]
fn enum_def_test() {
    const SRC_TEXT: &str = "\nenum MyEnum {\
                            \n    One: 1\
                            \n    Second\
                            \n    Max\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let enum_node = parser.parse_enum_def().unwrap();

    {
        const EXPECTED: &str = "\n<enum-definition name=\"MyEnum\">\
                                \n    <field name=\"One\" value=\"1\"/>\
                                \n    <field name=\"Second\"/>\
                                \n    <field name=\"Max\"/>\
                                \n</enum-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        enum_node.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    One = 1,\
                                     \n    Second = 0,\
                                     \n    Max = 0,\
                                     \n} MyEnum;\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        enum_node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_str_eq!(HEADER_EXPECTED, header_res);
        assert!(source_res.is_empty());
    }
}

#[test]
fn empty_enum_def_test() {
    const SRC_TEXT: &str = "\nenum EmptyEnum { }";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let enum_node = parser.parse_enum_def().unwrap();

    {
        const EXPECTED: &str = "\n<enum-definition name=\"EmptyEnum\"/>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        enum_node.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef enum {\n} EmptyEnum;\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        enum_node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(HEADER_EXPECTED, header_res);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert!(source_res.is_empty());
    }
}

#[test]
fn enum_with_one_field_def_test() {
    const SRC_TEXT: &str = "\nenum MyEnum { MinsInHour: 60\n }";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let enum_node = parser.parse_enum_def().unwrap();

    {
        const EXPECTED: &str = "\n<enum-definition name=\"MyEnum\">\
                                \n    <field name=\"MinsInHour\" value=\"60\"/>\
                                \n</enum-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        enum_node.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_str_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    MinsInHour = 60,\
                                     \n} MyEnum;\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        enum_node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert!(source_res.is_empty());
    }
}

#[test]
fn enum_work_test() {
    const SRC_TEXT: &str = "\npub enum MyEnum {\
                            \n    One: 1\
                            \n    Second\
                            \n    Max\
                            \n}\
                            \nfunc main() {\
                            \n    var a = MyEnum::Second\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        const EXPECTED: &str = "\n<enum-definition name=\"MyEnum\">\
                                \n    <attributes publicity=\"Public\"/>\
                                \n    <field name=\"One\" value=\"1\"/>\
                                \n    <field name=\"Second\"/>\
                                \n    <field name=\"Max\"/>\
                                \n</enum-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"a\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"automatic\"/>\
                                \n        </variable-definition>\
                                \n        <operation style=\"access\">\
                                \n            <identifier name=\"MyEnum\"/>\
                                \n            <identifier name=\"Second\"/>\
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
        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        if analyzer.has_errors() {
            panic!("{:?}", analyzer.get_errors());
        }
    }

    {
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    One = 1,\
                                     \n    Second = 2,\
                                     \n    Max = 3,\
                                     \n} MyEnum;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
                                     \n    MyEnum const a = 2;\
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
fn enum_in_module_work_test() {
    const SRC_TEXT: &str = "\nmodule color {\
                            \n    enum Color {\
                            \n        Red\
                            \n        Green\
                            \n        Blue\
                            \n    }\
                            \n}\
                            \nfunc main() {\
                            \n    var a = color::Color::Red\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

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
        const EXPECTED: &str = "\n<module-definition name=\"color\">\
                                \n    <enum-definition name=\"Color\">\
                                \n        <field name=\"Red\" value=\"0\"/>\
                                \n        <field name=\"Green\" value=\"1\"/>\
                                \n        <field name=\"Blue\" value=\"2\"/>\
                                \n    </enum-definition>\
                                \n</module-definition>\
                                \n<function-definition name=\"main\">\
                                \n    <return-type>\
                                \n        <type style=\"tuple\"/>\
                                \n    </return-type>\
                                \n    <operation style=\"binary\" operation=\"=\">\
                                \n        <variable-definition name=\"a\" is-global=\"false\" mutability=\"Immutable\">\
                                \n            <type style=\"named\" name=\"Color\"/>\
                                \n        </variable-definition>\
                                \n        <operation>\
                                \n            <literal style=\"integer-number\" value=\"0\"/>\
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
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    Red = 0,\
                                     \n    Green = 1,\
                                     \n    Blue = 2,\
                                     \n} Color;\
                                     \nvoid main();\n";

        const SOURCE_EXPECTED: &str = "void main()\
                                     \n{\
                                     \n    Color const a = 0;\
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
