use tanitc_ast::ast::{
    structs::{StructDef, StructFieldAttributes, StructFieldInfo},
    Ast,
};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_struct_def(&mut self) -> Result<Ast, Message> {
        let mut node = StructDef::default();

        self.parse_struct_header(&mut node)?;
        self.parse_struct_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_struct_header(&mut self, struct_def: &mut StructDef) -> Result<(), Message> {
        struct_def.location = self.consume_token(Lexem::KwStruct)?.location;
        struct_def.name.id = self.consume_identifier()?;
        Ok(())
    }

    pub fn parse_struct_body(&mut self, struct_def: &mut StructDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;
        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(true);
        self.parse_struct_body_internal(struct_def)?;
        self.set_ignore_nl_option(old_opt);

        self.consume_token(Lexem::Rcb)?;

        Ok(())
    }

    fn parse_struct_body_internal(&mut self, struct_def: &mut StructDef) -> Result<(), Message> {
        loop {
            let attrs = self.parse_attributes()?;

            let next = self.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    self.get_token();
                    continue;
                }

                Lexem::KwStruct => struct_def.internals.push(self.parse_struct_def()?),

                Lexem::KwUnion => struct_def.internals.push(self.parse_union_def()?),

                Lexem::KwVariant => struct_def.internals.push(self.parse_variant_def()?),

                Lexem::Identifier(id) => {
                    let identifier = self.consume_identifier()?;

                    if struct_def.fields.contains_key(&identifier) {
                        self.error(Message::from_string(
                            next.location,
                            format!("Struct has already field with identifier {id}"),
                        ));
                        continue;
                    }

                    self.consume_token(Lexem::Colon)?;

                    struct_def.fields.insert(
                        identifier,
                        StructFieldInfo {
                            ty: self.parse_type_spec()?,
                            attributes: StructFieldAttributes {
                                publicity: attrs.publicity.unwrap_or_default(),
                            },
                        },
                    );
                }

                _ => {
                    return Err(Message::from_string(
                        next.location,
                        format!(
                            "Unexpected token when parsing struct fields: {}",
                            next.lexem
                        ),
                    ));
                }
            }
        }

        Ok(())
    }
}
