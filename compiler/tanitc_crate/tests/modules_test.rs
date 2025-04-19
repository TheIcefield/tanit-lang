use tanitc_ast::Ast;
use tanitc_ident::Ident;
use tanitc_lexer::Lexer;
use tanitc_parser::Parser;

#[test]
fn module_test() {
    const SRC_TEXT: &str = "\nmodule M1\
                            \n{\
                            \n    module M2\
                            \n    {\
                            \n    }\
                            \n}";

    let m1_id = Ident::from("M1".to_string());
    let m2_id = Ident::from("M2".to_string());

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).unwrap());

    let res = parser.parse_module_def().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    let res = if let Ast::ModuleDef(node) = &res {
        assert!(node.identifier == m1_id);
        node.body.as_ref().unwrap()
    } else {
        panic!("res should be \'ModuleDef\'");
    };

    if let Ast::ModuleDef(node) = &res.statements[0] {
        assert!(node.identifier == m2_id);
    } else {
        panic!("res should be \'ModuleDef\'");
    };
}
