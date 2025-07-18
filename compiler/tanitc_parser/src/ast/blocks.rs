use tanitc_ast::{Ast, Block};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_global_block(&mut self) -> Result<Ast, Message> {
        let mut node = Block::default();

        self.parse_block_internal(&mut node)?;
        node.is_global = true;

        Ok(Ast::from(node))
    }

    pub fn parse_local_block(&mut self) -> Result<Ast, Message> {
        let mut node = Block::default();

        self.consume_token(Lexem::Lcb)?;

        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(false);

        self.parse_block_internal(&mut node)?;
        node.is_global = false;

        self.consume_token(Lexem::Rcb)?;

        self.set_ignore_nl_option(old_opt);

        Ok(Ast::from(node))
    }

    fn parse_block_internal(&mut self, block: &mut Block) -> Result<(), Message> {
        block.location = self.get_location();

        loop {
            let next = self.peek_token();

            if matches!(next.lexem, Lexem::Rcb | Lexem::EndOfFile) {
                break;
            }

            if next.lexem == Lexem::EndOfLine {
                self.get_token();
                continue;
            }

            let attrs = self.parse_attributes()?;

            let next = self.peek_token();

            let statement = match next.lexem {
                Lexem::KwDef | Lexem::KwModule => self.parse_module_def(),

                Lexem::KwFunc => self.parse_func_def(),

                Lexem::KwEnum => self.parse_enum_def(),

                Lexem::KwStruct => self.parse_struct_def(),

                Lexem::KwUnion => self.parse_union_def(),

                Lexem::KwVariant => self.parse_variant_def(),

                Lexem::KwVar | Lexem::KwStatic => self.parse_variable_def(),

                Lexem::KwAlias => self.parse_alias_def(),

                Lexem::Identifier(_)
                | Lexem::Integer(_)
                | Lexem::Decimal(_)
                | Lexem::Ampersand
                | Lexem::Plus
                | Lexem::Minus
                | Lexem::Star
                | Lexem::Not
                | Lexem::LParen => self.parse_expression(),

                Lexem::KwLoop | Lexem::KwWhile | Lexem::KwIf | Lexem::KwElse => self.parse_branch(),

                Lexem::KwReturn | Lexem::KwBreak | Lexem::KwContinue => self.parse_control_flow(),

                Lexem::KwUse => self.parse_use(),

                Lexem::Lcb => self.parse_local_block(),

                Lexem::KwExtern => self.parse_extern_def(),

                Lexem::KwImpl => self.parse_impl_def(),
                _ => {
                    self.skip_until(&[Lexem::EndOfLine]);
                    self.get_token();

                    self.error(Message::unexpected_token(next, &[]));
                    continue;
                }
            };

            match statement {
                Ok(mut child) => {
                    child.apply_attributes(attrs)?;
                    block.statements.push(child);
                }
                Err(err) => self.error(err),
            }
        }

        Ok(())
    }
}

#[test]
fn parse_local_block_test() {
    use tanitc_attributes::Safety;

    const SRC_TEXT: &str = "{\
                          \n    unsafe {\
                          \n    }\
                          \n{}\
                          \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");
    let ast = parser.parse_local_block().unwrap();

    let Ast::Block(block_node) = &ast else {
        panic!("Expected Block, actually: {}", ast.name());
    };

    assert_eq!(block_node.is_global, false);
    assert_eq!(block_node.attributes.safety, Safety::Inherited);

    let statements = &block_node.statements;
    assert_eq!(statements.len(), 2);

    {
        let node = &statements[0];
        let Ast::Block(block_node) = node else {
            panic!("Expected Block, actually: {}", node.name());
        };

        assert_eq!(block_node.attributes.safety, Safety::Unsafe);
        assert!(block_node.statements.is_empty());
    }

    {
        let node = &statements[1];
        let Ast::Block(block_node) = node else {
            panic!("Expected Block, actually: {}", node.name());
        };

        assert_eq!(block_node.attributes.safety, Safety::Inherited);
        assert!(block_node.statements.is_empty());
    }
}
