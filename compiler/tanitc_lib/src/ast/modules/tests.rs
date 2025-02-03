use super::ModuleDef;
use crate::ast::{identifiers::Identifier, Ast};
use crate::parser::Parser;

use tanitc_lexer::Lexer;

use std::str::FromStr;

#[test]
fn module_test() {
    const SRC_TEXT: &str = "\nmodule M1\
                            \n{\
                            \n    module M2\
                            \n    {\
                            \n    }\
                            \n}";

    let mut parser = Parser::new(Lexer::from_text(SRC_TEXT).unwrap());

    let res = ModuleDef::parse(&mut parser).unwrap();

    let res = if let Ast::ModuleDef(node) = &res {
        assert!(node.identifier == Identifier::from_str("M1").unwrap());
        node.body.as_ref().unwrap()
    } else {
        panic!("res should be \'ModuleDef\'");
    };

    if let Ast::ModuleDef(node) = &res.statements[0] {
        assert!(node.identifier == Identifier::from_str("M2").unwrap());
    } else {
        panic!("res should be \'ModuleDef\'");
    };
}
