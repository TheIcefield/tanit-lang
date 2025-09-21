use tanitc_ast::ast::{aliases::AliasDef, Ast};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_alias_def(&mut self) -> Result<Ast, Message> {
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

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::Ast;
    use tanitc_ident::{Ident, Name};
    use tanitc_ty::Type;

    use crate::Parser;

    #[test]
    fn alias_in_func_test() {
        const SRC_TEXT: &str = "func main() : ()\
                                {\
                                    alias Items = Vec<Item>\
                                }";

        let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

        let program = parser.parse_func_def().unwrap();

        let Ast::FuncDef(func_def) = &program else {
            panic!("Expected Ast::FuncDef, actually: {}", program.name());
        };

        assert!(func_def.parameters.is_empty());
        assert_eq!(func_def.name.id.to_string(), "main");
        assert_eq!(func_def.return_type.ty, Type::unit());

        let Some(func_body) = &func_def.body else {
            panic!("Function expected to have a body")
        };

        assert_eq!(func_body.statements.len(), 1);

        let stmt = &func_body.statements[0];
        let Ast::AliasDef(alias_node) = stmt else {
            panic!("Expected Ast::AliasDef, actually: {}", stmt.name());
        };

        assert_eq!(alias_node.identifier.to_string(), "Items");
        assert_eq!(
            alias_node.value.ty,
            Type::Template {
                identifier: Ident::from("Vec".to_string()),
                generics: vec![Type::Custom(Name::from("Item".to_string()))]
            }
        );
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
}
