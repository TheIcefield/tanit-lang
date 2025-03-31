use tanitc_ast::Ast;

use tanitc_codegen::CodeGenStream;
use tanitc_ident::Ident;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;
use tanitc_serializer::XmlWriter;
use tanitc_ty::Type;

#[test]
fn alias_def_test() {
    const SRC_TEXT: &str = "alias MyAlias = f32";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let alias_node = parser.parse_alias_def().unwrap();

    {
        const EXPECTED: &str = "\n<alias-definition name=\"MyAlias\">\
                                \n    <type style=\"primitive\" name=\"f32\"/>\
                                \n</alias-definition>";

        let mut buffer = Vec::<u8>::new();
        let mut writer = XmlWriter::new(&mut buffer).unwrap();

        alias_node.accept(&mut writer).unwrap();
        let res = String::from_utf8(buffer).unwrap();

        assert_eq!(EXPECTED, res);
    }

    {
        const HEADER_EXPECTED: &str = "typedef float MyAlias;\n";

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        alias_node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        let source_res = String::from_utf8(source_buffer).unwrap();

        assert_eq!(HEADER_EXPECTED, header_res);
        assert!(source_res.is_empty());
    }
}

#[test]
fn alias_in_func_test() {
    const SRC_TEXT: &str = "func main() : ()\
                            {\
                                alias Items = Vec<Item>\
                            }";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).expect("Lexer creation failed"));

    let res = if let Ast::FuncDef(node) = parser.parse_func_def().unwrap() {
        assert!(node.identifier == Ident::from("main".to_string()));
        assert!(node.parameters.is_empty());

        if let Type::Tuple(components) = &node.return_type.get_type() {
            assert!(components.is_empty());
        } else {
            panic!("Type expected to be an empty tuple");
        }

        node.body.unwrap()
    } else {
        panic!("res has to be \'function definition\'");
    };

    let statements = if let Ast::Block(node) = res.as_ref() {
        &node.statements
    } else {
        panic!("node has to be \'local scope\'");
    };

    if let Ast::AliasDef(node) = &statements[0] {
        assert!(node.identifier == Ident::from("Items".to_string()));

        if let Type::Template {
            identifier,
            generics,
        } = &node.value.get_type()
        {
            assert!(*identifier == Ident::from("Vec".to_string()));
            assert_eq!(generics.len(), 1);
            if let Type::Custom(id) = &generics[0] {
                assert_eq!(id, "Item");
            } else {
                panic!("Type is expected to be \"Item\"")
            }
        } else {
            panic!("Alias type expected to be an template type");
        }
    } else {
        panic!("res has to be \'alias definition\'");
    };
}
