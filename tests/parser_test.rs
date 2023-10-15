
#[test]
fn module_test() {
    use tanit::ast;
    use tanit::{lexer, error_listener, parser};

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

#[test]
fn struct_test() {
    use tanit::ast;
    use tanit::{lexer, error_listener, parser};

    static SRC_PATH: &str = "./examples/structs.tt";

    let lexer = lexer::Lexer::from_file(SRC_PATH, false);

    assert_eq!(lexer.is_ok(), true);

    let lexer = lexer.unwrap();

    let error_listener = error_listener::ErrorListener::new();

    let mut parser = parser::Parser::new(lexer, error_listener);

    let res = tanit::ast::structs::parse_struct_def(&mut parser);

    assert_eq!(res.is_some(), true);

    let res = res.unwrap();

    if let ast::Ast::StructDef { node } = res {
        assert_eq!(node.identifier, String::from("S1"));

        assert_eq!(node.fields[0].identifier, String::from("f1"));
        assert_eq!(node.fields[0].is_field, true);
        assert_eq!(node.fields[0].is_global, false);
        assert_eq!(node.fields[0].is_mutable, true);

        let mut field_type = node.fields[0].var_type.clone();
        assert_eq!(field_type.identifier, String::from("i32"));

        assert_eq!(node.fields[1].identifier, String::from("f2"));

        field_type = node.fields[1].var_type.clone();
        assert_eq!(field_type.identifier, String::from("Vec"));
        assert_eq!(field_type.children[0].identifier, String::from("i32"));
    } else {
        panic!("res should be \'ModuleDef\'");
    };
}
