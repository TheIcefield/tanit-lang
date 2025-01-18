use super::ModuleDef;
use crate::ast::{identifiers::Identifier, Ast};
use crate::parser::{lexer::Lexer, Parser};

use std::str::FromStr;

#[test]
fn module_test() {
    static SRC_PATH: &str = "./examples/modules.tt";

    let lexer = Lexer::from_file(SRC_PATH, false).unwrap();

    let mut parser = Parser::new(lexer);

    let res = ModuleDef::parse(&mut parser).unwrap();

    let res = if let Ast::ModuleDef { node } = &res {
        assert!(node.identifier == Identifier::from_str("M1").unwrap());
        &node.body.as_ref().unwrap()
    } else {
        panic!("res should be \'ModuleDef\'");
    };

    let res = if let Ast::Scope { node } = res.as_ref() {
        &node.statements[0]
    } else {
        panic!("res should be \'global scope\'");
    };

    if let Ast::ModuleDef { node } = res {
        assert!(node.identifier == Identifier::from_str("M2").unwrap());
    } else {
        panic!("res should be \'ModuleDef\'");
    };
}
