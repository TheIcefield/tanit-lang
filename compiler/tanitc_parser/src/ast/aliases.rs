use tanitc_ast::Ast;
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_alias_def(&mut self) -> Result<Ast, Message> {
        use tanitc_ast::AliasDef;
        let mut node = AliasDef {
            location: self.consume_token(Lexem::KwAlias)?.location,
            identifier: self.consume_identifier()?,
            ..Default::default()
        };

        self.consume_token(Lexem::Assign)?;

        node.value = self.parse_type_spec()?;

        Ok(Ast::from(node))
    }
}

#[test]
fn parse_alias_def_test() {
    const SRC_TEXT: &str = "alias MyAlias = f32";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");
    let ast = parser.parse_alias_def().unwrap();

    let Ast::AliasDef(alias_node) = &ast else {
        panic!("Expected AliasDef, actually: {}", ast.name());
    };

    assert_eq!(alias_node.identifier.to_string(), "MyAlias");
}
