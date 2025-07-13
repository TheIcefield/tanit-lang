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
                            format!("Variant already has field with name \"{id}\""),
                        ));
                        continue;
                    }

                    let field = self.parse_variant_field()?;
                    variant_def.fields.insert(identifier, field);
                }

                Lexem::Lcb => {
                    return Err(Message::new(
                        next.location,
                        "Unexpected token: \"{\" during parsing variant fields.\n\
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

    fn parse_variant_field(&mut self) -> Result<VariantField, Message> {
        let old_opt = self.does_ignore_nl();
        self.set_ignore_nl_option(false);
        let field = self.parse_variant_field_internal()?;
        self.set_ignore_nl_option(old_opt);

        Ok(field)
    }

    fn parse_variant_field_internal(&mut self) -> Result<VariantField, Message> {
        let next = self.peek_token();
        match next.lexem {
            Lexem::EndOfLine | Lexem::Rcb | Lexem::Identifier(_) => Ok(VariantField::Common),

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

                    Ok(VariantField::TupleLike(processed_components))
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

                Ok(VariantField::StructLike(fields))
            }

            _ => Err(Message::from_string(
                next.location,
                format!("Unexpected token during parsing variant: {next}"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;

    use tanitc_ast::{
        attributes::{FieldAttributes, Publicity},
        Ast, FieldInfo, Fields, ParsedTypeInfo, TypeSpec, VariantField,
    };
    use tanitc_ident::Ident;
    use tanitc_lexer::location::Location;
    use tanitc_ty::Type;

    #[test]
    fn empty_variant_test() {
        const SRC_TEXT: &str = "variant EmptyVariant { }";

        let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

        let node = parser.parse_variant_def().unwrap();
        let Ast::VariantDef(variant_node) = &node else {
            panic!("Expected VariantDef, actually: {}", node.name());
        };

        assert_eq!(variant_node.identifier.to_string(), "EmptyVariant");
        assert_eq!(variant_node.attributes.publicity, Publicity::Private);
        assert!(variant_node.fields.is_empty());
    }

    #[test]
    fn variant_with_one_unit_test() {
        const SRC_TEXT: &str = "variant MyVariant { F1 }";
        let f1_id = Ident::from("F1".to_string());

        let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

        let node = parser.parse_variant_def().unwrap();
        let Ast::VariantDef(variant_node) = &node else {
            panic!("Expected VariantDef, actually: {}", node.name());
        };

        assert_eq!(variant_node.identifier.to_string(), "MyVariant");
        assert_eq!(variant_node.fields.len(), 1);
        assert_eq!(variant_node.fields.get(&f1_id), Some(&VariantField::Common));
    }

    #[test]
    fn variant_with_one_tuple_test() {
        const SRC_TEXT: &str = "variant MyVariant { F2(u32) }";

        let f2_id = Ident::from("F2".to_string());

        let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

        let node = parser.parse_variant_def().unwrap();
        let Ast::VariantDef(variant_node) = &node else {
            panic!("Expected VariantDef, actually: {}", node.name());
        };

        assert_eq!(variant_node.identifier.to_string(), "MyVariant");
        assert_eq!(variant_node.fields.len(), 1);
        assert_eq!(
            variant_node.fields.get(&f2_id),
            Some(&VariantField::TupleLike(vec![TypeSpec {
                ty: Type::U32,
                location: Location { row: 1, col: 24 },
                info: ParsedTypeInfo { is_mut: false },
            }]))
        );
    }

    #[test]
    fn variant_with_one_struct_test() {
        const SRC_TEXT: &str = "variant MyVariant { F3 { value: u32 } }";

        let f3_id = Ident::from("F3".to_string());
        let value_id = Ident::from("value".to_string());

        let mut fields = Fields::new();
        fields.insert(
            value_id,
            FieldInfo {
                ty: TypeSpec {
                    location: Location { row: 1, col: 34 },
                    info: ParsedTypeInfo { is_mut: false },
                    ty: Type::U32,
                },
                attributes: FieldAttributes::default(),
            },
        );

        let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

        let node = parser.parse_variant_def().unwrap();
        let Ast::VariantDef(variant_node) = &node else {
            panic!("Expected VariantDef, actually: {}", node.name());
        };

        assert_eq!(variant_node.identifier.to_string(), "MyVariant");
        assert_eq!(variant_node.fields.len(), 1);
        assert_eq!(
            variant_node.fields.get(&f3_id),
            Some(&VariantField::StructLike(fields))
        );
    }

    #[test]
    fn variant_with_many_fields_test() {
        const SRC_TEXT: &str = "variant MyVariant { F1 F2(u32) F3 F4(i32) F5 { value: f32} }";

        let f1_id = Ident::from("F1".to_string());
        let f2_id = Ident::from("F2".to_string());
        let f3_id = Ident::from("F3".to_string());
        let f4_id = Ident::from("F4".to_string());
        let f5_id = Ident::from("F5".to_string());
        let value_id = Ident::from("value".to_string());

        let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

        let node = parser.parse_variant_def().unwrap();
        let Ast::VariantDef(variant_node) = &node else {
            panic!("Expected VariantDef, actually: {}", node.name());
        };

        assert_eq!(variant_node.identifier.to_string(), "MyVariant");
        assert_eq!(variant_node.fields.len(), 5);
        assert_eq!(variant_node.fields.get(&f1_id), Some(&VariantField::Common));
        assert_eq!(
            variant_node.fields.get(&f2_id),
            Some(&VariantField::TupleLike(vec![TypeSpec {
                ty: Type::U32,
                location: Location { row: 1, col: 27 },
                info: ParsedTypeInfo { is_mut: false },
            }]))
        );
        assert_eq!(variant_node.fields.get(&f3_id), Some(&VariantField::Common));
        assert_eq!(
            variant_node.fields.get(&f4_id),
            Some(&VariantField::TupleLike(vec![TypeSpec {
                ty: Type::I32,
                location: Location { row: 1, col: 38 },
                info: ParsedTypeInfo { is_mut: false },
            }]))
        );

        let mut fields = Fields::new();
        fields.insert(
            value_id,
            FieldInfo {
                ty: TypeSpec {
                    location: Location { row: 1, col: 56 },
                    info: ParsedTypeInfo { is_mut: false },
                    ty: Type::F32,
                },
                attributes: FieldAttributes::default(),
            },
        );
        assert_eq!(
            variant_node.fields.get(&f5_id),
            Some(&VariantField::StructLike(fields))
        );
    }
}
