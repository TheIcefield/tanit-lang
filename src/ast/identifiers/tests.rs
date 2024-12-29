use super::{Identifier, IdentifierType};
use crate::ast::{
    expressions::{Expression, ExpressionType},
    values::{Value, ValueType},
    Ast,
};
use crate::parser::{location::Location, token::Lexem};

use std::str::FromStr;

#[test]
fn str_conversion_test() {
    let id = Identifier::from_str("hello").unwrap();

    if let IdentifierType::Common(id) = &id.identifier {
        assert_eq!(id, "hello");
    } else {
        panic!("expected common identifier");
    }
}

#[test]
fn expr_conversion_test() {
    let expression = Expression {
        location: Location::new(),
        expr: ExpressionType::Binary {
            operation: Lexem::Dcolon,
            lhs: Box::new(Ast::Value {
                node: Value {
                    location: Location::new(),
                    value: ValueType::Identifier(Identifier {
                        location: Location::new(),
                        identifier: IdentifierType::Common("hello".to_string()),
                    }),
                },
            }),
            rhs: Box::new(Ast::Expression {
                node: Box::new(Expression {
                    location: Location::new(),
                    expr: ExpressionType::Binary {
                        operation: Lexem::Dcolon,
                        lhs: Box::new(Ast::Value {
                            node: Value {
                                location: Location::new(),
                                value: ValueType::Identifier(Identifier {
                                    location: Location::new(),
                                    identifier: IdentifierType::Common("my".to_string()),
                                }),
                            },
                        }),
                        rhs: Box::new(Ast::Value {
                            node: Value {
                                location: Location::new(),
                                value: ValueType::Identifier(Identifier {
                                    location: Location::new(),
                                    identifier: IdentifierType::Common("world".to_string()),
                                }),
                            },
                        }),
                    },
                }),
            }),
        },
    };

    let expected = vec!["hello".to_string(), "my".to_string(), "world".to_string()];

    if let IdentifierType::Complex(ids) = Identifier::from_expr(&expression).unwrap().identifier {
        assert_eq!(ids, expected);
    } else {
        panic!("expecred complex identifier");
    }
}
