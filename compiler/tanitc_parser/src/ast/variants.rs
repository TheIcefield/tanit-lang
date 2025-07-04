use tanitc_ast::{Ast, Fields, StructDef, TypeSpec, VariantDef, VariantField};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;
use tanitc_ty::Type;

use crate::Parser;

impl Parser {
    pub fn parse_variant_def(&mut self) -> Result<Ast, Message> {
        let mut node = VariantDef::default();

        self.parse_variant_header(&mut node)?;
        self.parse_variant_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_variant_header(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        variant_def.location = self.consume_token(Lexem::KwVariant)?.location;
        variant_def.identifier = self.consume_identifier()?;

        Ok(())
    }

    fn parse_variant_body(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;
        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);
        self.parse_variant_body_internal(variant_def)?;
        self.set_ignore_nl_option(old_opt);

        self.consume_token(Lexem::Rcb)?;

        Ok(())
    }

    fn parse_variant_body_internal(&mut self, variant_def: &mut VariantDef) -> Result<(), Message> {
        loop {
            let next = self.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,

                Lexem::EndOfLine => {
                    self.get_token();
                    continue;
                }

                Lexem::KwStruct => variant_def.internals.push(self.parse_struct_def()?),

                Lexem::KwUnion => variant_def.internals.push(self.parse_union_def()?),

                Lexem::KwVariant => variant_def.internals.push(self.parse_variant_def()?),

                Lexem::Identifier(id) => {
                    let identifier = self.consume_identifier()?;

                    if variant_def.fields.contains_key(&identifier) {
                        self.error(Message::from_string(
                            next.location,
                            format!("Enum has already field with identifier \"{id}\""),
                        ));
                        continue;
                    }

                    variant_def
                        .fields
                        .insert(identifier, self.parse_variant_field()?);

                    self.consume_new_line()?;
                }

                Lexem::Lcb => {
                    return Err(Message::new(
                        next.location,
                        "Unexpected token: \"{\" during parsing enum fields.\n\
                            Help: If you tried to declare struct-like field, place \"{\" \
                            in the same line with name of the field.",
                    ));
                }

                _ => {
                    return Err(Message::unexpected_token(next, &[]));
                }
            }
        }

        Ok(())
    }
}

// Variant field
impl Parser {
    fn parse_variant_field(&mut self) -> Result<VariantField, Message> {
        let mut node = VariantField::default();

        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(false);
        self.parse_variant_field_internal(&mut node)?;
        self.set_ignore_nl_option(old_opt);

        Ok(node)
    }

    fn parse_variant_field_internal(
        &mut self,
        variant_field: &mut VariantField,
    ) -> Result<(), Message> {
        let next = self.peek_token();
        match next.lexem {
            Lexem::EndOfLine => {
                *variant_field = VariantField::Common;

                Ok(())
            }

            Lexem::LParen => {
                let location = self.peek_token().location;
                let (ty, info) = self.parse_tuple_def()?;

                if let Type::Tuple(components) = &ty {
                    let mut processed_components: Vec<TypeSpec> = vec![];
                    for ty in components.iter() {
                        processed_components.push(TypeSpec {
                            location,
                            info,
                            ty: ty.clone(),
                        });
                    }
                    *variant_field = VariantField::TupleLike(processed_components);

                    Ok(())
                } else {
                    Err(Message::unexpected_token(next, &[]))
                }
            }

            Lexem::Lcb => {
                let mut node = StructDef::default();
                self.parse_struct_body(&mut node)?;

                let mut fields = Fields::new();
                for (field_name, field_info) in node.fields.iter() {
                    fields.insert(*field_name, field_info.clone());
                }

                *variant_field = VariantField::StructLike(fields);

                Ok(())
            }

            _ => Err(Message::from_string(
                next.location,
                format!("Unexpected token during parsing enum: {next}"),
            )),
        }
    }
}
