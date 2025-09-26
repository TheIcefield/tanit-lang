use tanitc_ast::ast::{
    unions::{UnionDef, UnionFieldAttributes, UnionFieldInfo},
    Ast,
};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_union_def(&mut self) -> Result<Ast, Message> {
        let mut node = UnionDef::default();

        self.parse_union_header(&mut node)?;
        self.parse_union_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_union_header(&mut self, union_def: &mut UnionDef) -> Result<(), Message> {
        union_def.location = self.consume_token(Lexem::KwUnion)?.location;
        union_def.name.id = self.consume_identifier()?;
        Ok(())
    }

    fn parse_union_body(&mut self, union_def: &mut UnionDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;

        self.parse_union_body_internal(union_def)?;

        self.consume_token(Lexem::Rcb)?;

        Ok(())
    }

    fn parse_union_body_internal(&mut self, union_def: &mut UnionDef) -> Result<(), Message> {
        loop {
            let attrs = self.parse_attributes()?;

            let next = self.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    self.get_token();
                    continue;
                }

                Lexem::KwStruct => union_def.internals.push(self.parse_struct_def()?),

                Lexem::KwUnion => union_def.internals.push(self.parse_union_def()?),

                Lexem::KwVariant => union_def.internals.push(self.parse_variant_def()?),

                Lexem::Identifier(id) => {
                    let identifier = self.consume_identifier()?;

                    if union_def.fields.contains_key(&identifier) {
                        self.error(Message::from_string(
                            &next.location,
                            format!("Struct has already field with identifier {id}"),
                        ));
                        continue;
                    }

                    self.consume_token(Lexem::Colon)?;

                    union_def.fields.insert(
                        identifier,
                        UnionFieldInfo {
                            ty: self.parse_type_spec()?,
                            attributes: UnionFieldAttributes {
                                publicity: attrs.publicity.unwrap_or_default(),
                            },
                        },
                    );
                }

                _ => {
                    return Err(Message::from_string(
                        &next.location,
                        format!("Unexpected token when parsing union fields: {}", next.lexem),
                    ));
                }
            }
        }

        Ok(())
    }
}
