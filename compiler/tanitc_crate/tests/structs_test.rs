use tanitc_ast::Ast;
use tanitc_codegen::CodeGenStream;
use tanitc_ident::Ident;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::XmlWriter;
use tanitc_ty::Type;

#[test]
fn struct_def_test() {
    const SRC_TEXT: &str = "\nstruct MyStruct\
                            \n{\
                            \n    f1: i32\
                            \n    f2: f32\
                            \n}";

    let struct_id = Ident::from("MyStruct".to_string());
    let f1_id = Ident::from("f1".to_string());
    let f2_id = Ident::from("f2".to_string());

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Failed to create lexer"));

    let struct_node = parser.parse_struct_def().unwrap();

    if let Ast::StructDef(node) = &struct_node {
        assert!(node.identifier == struct_id);

        let field_type = node.fields.get(&f1_id).unwrap().get_type();
        assert!(matches!(field_type, Type::I32));

        let field_type = node.fields.get(&f2_id).unwrap().get_type();
        assert!(matches!(field_type, Type::F32));
    } else {
        panic!("res should be \'StructDef\'");
    };

    {
        const EXPECTED: &str = "\n<struct-definition name=\"MyStruct\">\
                                \n    <field name=\"f1\">\
                                \n        <type style=\"primitive\" name=\"i32\"/>\
                                \n    </field>\
                                \n    <field name=\"f2\">\
                                \n        <type style=\"primitive\" name=\"f32\"/>\
                                \n    </field>\
                                \n</struct-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        struct_node.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef struct {\
                                     \nsigned int f1;\
                                     \nfloat f2;\
                                     \n} MyStruct;\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        struct_node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert!(source_res.is_empty());
    }
}
