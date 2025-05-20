use tanitc_ast::{Ast, ImplDef};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

// Impl definition
impl Parser {
    pub fn parse_impl_def(&mut self) -> Result<Ast, Message> {
        let mut node = ImplDef::default();

        self.parse_impl_header(&mut node)?;
        self.parse_impl_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_impl_header(&mut self, impl_def: &mut ImplDef) -> Result<(), Message> {
        impl_def.location = self.consume_token(Lexem::KwImpl)?.location;
        impl_def.identifier = self.consume_identifier()?;

        Ok(())
    }

    fn parse_impl_body(&mut self, impl_def: &mut ImplDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;

        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(true);

        let methods = self.parse_global_block()?;

        let Ast::Block(mut block) = methods else {
            return Err(Message {
                location: methods.location(),
                text: format!("Unexpected node {} within impl block", methods.name()),
            });
        };

        for method in block.statements.iter_mut() {
            if let Ast::FuncDef(method) = method {
                impl_def.methods.push(std::mem::take(method));
            }
        }

        self.set_ignore_nl_option(old_opt);

        self.consume_token(Lexem::Rcb)?;

        Ok(())
    }
}
