use super::EnumDef;

use tanitc_codegen::CodeGenStream;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::XmlWriter;

#[test]
fn enum_def_test() {
    const SRC_TEXT: &str = "\nenum MyEnum {\
                            \n    One: 1\
                            \n    Second\
                            \n    Max\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let enum_node = EnumDef::parse(&mut parser).unwrap();

    {
        const EXPECTED: &str = "\n<enum-definition name=\"MyEnum\">\
                                \n    <field name=\"One\" value=\"1\"/>\
                                \n    <field name=\"Second\"/>\
                                \n    <field name=\"Max\"/>\
                                \n</enum-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        enum_node.serialize(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
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

        enum_node.codegen(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert!(source_res.is_empty());
    }
}

#[test]
fn empty_enum_def_test() {
    const SRC_TEXT: &str = "\nenum EmptyEnum { }";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let enum_node = EnumDef::parse(&mut parser).unwrap();

    {
        const EXPECTED: &str = "\n<enum-definition name=\"EmptyEnum\"/>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        enum_node.serialize(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef enum {\n} EmptyEnum;\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        enum_node.codegen(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert!(source_res.is_empty());
    }
}

#[test]
fn enum_with_one_field_def_test() {
    const SRC_TEXT: &str = "\nenum MyEnum { MinsInHour: 60\n }";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let enum_node = EnumDef::parse(&mut parser).unwrap();

    {
        const EXPECTED: &str = "\n<enum-definition name=\"MyEnum\">\
                                \n    <field name=\"MinsInHour\" value=\"60\"/>\
                                \n</enum-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        enum_node.serialize(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef enum {\
                                     \n    MinsInHour = 60,\
                                     \n} MyEnum;\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        enum_node.codegen(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert!(source_res.is_empty());
    }
}
