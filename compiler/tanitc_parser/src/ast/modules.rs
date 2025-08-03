use tanitc_ast::ast::{modules::ModuleDef, Ast};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_module_def(&mut self) -> Result<Ast, Message> {
        let mut node = ModuleDef::default();

        self.parse_module_header(&mut node)?;
        self.parse_module_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_module_header(&mut self, mod_def: &mut ModuleDef) -> Result<(), Message> {
        let next = self.peek_token();
        mod_def.location = next.location;

        if Lexem::KwDef == next.lexem {
            self.consume_token(Lexem::KwDef)?;
            mod_def.is_external = true;
        }

        self.consume_token(Lexem::KwModule)?;

        mod_def.identifier = self.consume_identifier()?;

        Ok(())
    }

    fn parse_module_body(&mut self, mod_def: &mut ModuleDef) -> Result<(), Message> {
        if !mod_def.is_external {
            self.parse_module_body_internal(mod_def)
        } else {
            Ok(())
        }
    }

    fn parse_module_body_internal(&mut self, mod_def: &mut ModuleDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;

        let block = self.parse_global_block()?;

        self.consume_token(Lexem::Rcb)?;

        if let Ast::Block(node) = block {
            mod_def.body = Some(node);
        } else {
            return Err(Message::unreachable(
                mod_def.location,
                "expected Block".to_string(),
            ));
        }

        Ok(())
    }
}

#[test]
fn module_test() {
    const SRC_TEXT: &str = "\nmodule M1\
                            \n{\
                            \n    unsafe module M2\
                            \n    {\
                            \n    }\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).unwrap();

    let res = parser.parse_module_def().unwrap();

    let Ast::ModuleDef(node) = &res else {
        panic!("Expected ModuleDef, actually: {}", res.name());
    };

    assert_eq!(node.identifier.to_string(), "M1");
    assert!(node.body.is_some());

    let body = node.body.as_ref().unwrap();
    let Ast::ModuleDef(node) = &body.statements[0] else {
        panic!(
            "Expected ModuleDef, actually: {}",
            &body.statements[0].name()
        );
    };

    assert_eq!(node.identifier.to_string(), "M2");
}
