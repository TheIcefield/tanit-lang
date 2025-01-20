use super::ModuleDef;
use crate::ast::{identifiers::Identifier, Ast};
use crate::parser::{lexer::Lexer, Parser};

use std::str::FromStr;

#[test]
fn module_test() {
    const SRC_PATH: &str = "./examples/modules.tt";

    let lexer = Lexer::from_file(SRC_PATH).unwrap();

    let mut parser = Parser::new(lexer);

    let res = ModuleDef::parse(&mut parser).unwrap();

    let res = if let Ast::ModuleDef { node } = &res {
        assert!(node.identifier == Identifier::from_str("M1").unwrap());
        node.body.as_ref().unwrap()
    } else {
        panic!("res should be \'ModuleDef\'");
    };

    if let Ast::ModuleDef { node } = &res.statements[0] {
        assert!(node.identifier == Identifier::from_str("M2").unwrap());
    } else {
        panic!("res should be \'ModuleDef\'");
    };
}
