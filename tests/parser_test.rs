
#[test]
fn module_test() {
    use tanit::ast;
    use tanit::{lexer, lexer::{Token, TokenType},
        error_listener, parser
    };

    static SRC_PATH: &str = "./examples/modules.tt";

    let lexer = lexer::Lexer::from_file(SRC_PATH, false);

    assert_eq!(lexer.is_ok(), true);

    let lexer = lexer.unwrap();

    let error_listener = error_listener::ErrorListener::new();

    let mut parser = parser::Parser::new(lexer, error_listener);

    let res = tanit::ast::modules::parse(&mut parser);

    assert_eq!(res.is_some(), true);

    let res = res.unwrap();

    let res = if let ast::Ast::ModuleDef { node } = res {
        assert_eq!(node.identifier, String::from("M1"));

        node.body.statements[0].clone()
    } else {
        panic!("res should be \'ModuleDef\'");
    };

    if let ast::Ast::ModuleDef { node } = res {
        assert_eq!(node.identifier, String::from("M2"));
    } else {
        panic!("res should be \'ModuleDef\'");
    };

}
