use tanitc_ast::ast::{enums::EnumDef, Ast};
use tanitc_lexer::token::Lexem;
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_enum_def(&mut self) -> Result<Ast, Message> {
        let mut node = EnumDef::default();

        self.parse_enum_header(&mut node)?;
        self.parse_enum_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_enum_header(&mut self, enum_def: &mut EnumDef) -> Result<(), Message> {
        enum_def.location = self.consume_token(Lexem::KwEnum)?.location;
        enum_def.name.id = self.consume_identifier()?;

        Ok(())
    }

    fn parse_enum_body(&mut self, enum_def: &mut EnumDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;
        let old_opt = self.does_ignore_nl();

        self.set_ignore_nl_option(false);
        self.parse_enum_body_internal(enum_def)?;
        self.set_ignore_nl_option(old_opt);

        self.consume_token(Lexem::Rcb)?;

        Ok(())
    }

    fn parse_enum_body_internal(&mut self, enum_def: &mut EnumDef) -> Result<(), Message> {
        loop {
            let next = self.peek_token();

            match &next.lexem {
                Lexem::Rcb => break,
                Lexem::EndOfLine => {
                    self.get_token();
                    continue;
                }
                Lexem::Identifier(id) => {
                    let identifier = self.consume_identifier()?;

                    let value = if Lexem::Colon == self.peek_token().lexem {
                        self.consume_token(Lexem::Colon)?;

                        let token = self.consume_integer()?;
                        let value = if let Lexem::Integer(value) = token.lexem {
                            match value.parse::<usize>() {
                                Ok(value) => value,
                                Err(err) => {
                                    return Err(Message::parse_int_error(token.location, err))
                                }
                            }
                        } else {
                            unreachable!()
                        };

                        Some(value)
                    } else {
                        None
                    };

                    if enum_def.fields.contains_key(&identifier) {
                        self.error(Message::from_string(
                            next.location,
                            format!("Enum has already field with identifier \"{id}\""),
                        ));
                        continue;
                    }

                    enum_def.fields.insert(identifier, value);

                    self.consume_new_line()?;
                }

                Lexem::Lcb => {
                    return Err(Message::new(
                        next.location,
                        "Unexpected token: \"{{\" during parsing enum fields.\n\
                            Help: if you tried to declare struct-like field, place \"{{\" \
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

#[test]
fn parse_enum_def_test() {
    use tanitc_attributes::Publicity;
    use tanitc_ident::Ident;

    const SRC_TEXT: &str = "\nenum MyEnum {\
                            \n    One: 1\
                            \n    Two\
                            \n    Max\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let node = parser.parse_enum_def().unwrap();
    let Ast::EnumDef(enum_node) = &node else {
        panic!("Expected EnumDef, actually: {}", node.name());
    };

    assert_eq!(enum_node.name.id.to_string(), "MyEnum");
    assert_eq!(enum_node.attributes.publicity, Publicity::Private);
    assert_eq!(enum_node.fields.len(), 3);

    assert_eq!(
        enum_node.fields.get(&Ident::from("One".to_string())),
        Some(&Some(1))
    );
    assert_eq!(
        enum_node.fields.get(&Ident::from("Two".to_string())),
        Some(&None)
    );
    assert_eq!(
        enum_node.fields.get(&Ident::from("Max".to_string())),
        Some(&None)
    );
}

#[test]
fn parse_empty_enum_def_test() {
    use tanitc_attributes::Publicity;
    const SRC_TEXT: &str = "\nenum EmptyEnum { }";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let node = parser.parse_enum_def().unwrap();
    let Ast::EnumDef(enum_node) = &node else {
        panic!("Expected EnumDef, actually: {}", node.name());
    };

    assert_eq!(enum_node.name.id.to_string(), "EmptyEnum");
    assert_eq!(enum_node.attributes.publicity, Publicity::Private);
    assert!(enum_node.fields.is_empty());
}

#[test]
fn parse_enum_with_one_field_def_test() {
    use tanitc_attributes::Publicity;
    use tanitc_ident::Ident;

    const SRC_TEXT: &str = "\nenum MyEnum { MinsInHour: 60\n }";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Parser creation failed");

    let node = parser.parse_enum_def().unwrap();
    let Ast::EnumDef(enum_node) = &node else {
        panic!("Expected EnumDef, actually: {}", node.name());
    };

    assert_eq!(enum_node.name.id.to_string(), "MyEnum");
    assert_eq!(enum_node.attributes.publicity, Publicity::Private);
    assert_eq!(enum_node.fields.len(), 1);

    assert_eq!(
        enum_node.fields.get(&Ident::from("MinsInHour".to_string())),
        Some(&Some(60))
    );
}
