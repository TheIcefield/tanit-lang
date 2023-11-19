use crate::lexer::TokenType;
use crate::parser::put_intent;
use crate::ast::{Ast, IAst, Stream, calls};
use crate::parser::Parser;

use std::io::Write;

use super::values;

#[derive(Clone)]
pub struct Expression {
    pub operation: Option<TokenType>,
    pub lhs: Option<Box<Ast>>,
    pub rhs: Option<Box<Ast>>,
    pub term: Option<Box<Ast>>
}

impl IAst for Expression {
    fn traverse(&self, stream: &mut Stream, intent: usize) -> std::io::Result<()> {
        match &self.operation {
            Some(op) => {
                writeln!(stream, "{}<operation operator=\"{}\">",
                    put_intent(intent), op)?;

                match &self.lhs {
                    Some(lhs) => lhs.traverse(stream, intent + 1)?,
                    _ => { },
                }

                match &self.rhs {
                    Some(rhs) => rhs.traverse(stream, intent + 1)?,
                    _ => { },
                }

                writeln!(stream, "{}</operation>", put_intent(intent))?;

            },
            _ => {},
        }
        
        match &self.term {
            Some(t) => t.traverse(stream, intent)?,
            _ => { },
        }

        Ok(())
    }
}

pub fn parse_expression(parser: &mut Parser) -> Option<Ast> {
    parse_assign(parser)
}

fn parse_assign(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_logical_or(parser)?;

    let next = parser.peek_token();
    match next.lexem {
          TokenType::AddAssign    
        | TokenType::SubAssign    
        | TokenType::MulAssign    
        | TokenType::DivAssign    
        | TokenType::ModAssign    
        | TokenType::OrAssign     
        | TokenType::AndAssign    
        | TokenType::XorAssign    
        | TokenType::LShiftAssign 
        | TokenType::RShiftAssign => {
            parser.get_token();
            let operation = Some(next.lexem);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));

            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_logical_or(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_logical_and(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::KwOr => {
            parser.get_token();
            let operation = Some(TokenType::KwOr);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));
                                                
            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_logical_and(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_bitwise_or(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::KwOr => {
            parser.get_token();
            let operation = Some(TokenType::KwAnd);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));
                                                
            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_bitwise_or(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_bitwise_xor(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Stick => {
            parser.get_token();
            let operation = Some(TokenType::Stick);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));
                                                
            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_bitwise_xor(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_bitwise_and(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Xor => {
            parser.get_token();
            let operation = Some(TokenType::Xor);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));
                                                
            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_bitwise_and(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_logical_eq(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Xor => {
            parser.get_token();
            let operation = Some(TokenType::Ampersand);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));
                                                
            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_logical_eq(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_logical_less_or_greater(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Eq | TokenType::Neq => {
            parser.get_token();
            let operation = Some(next.lexem);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));
                                                
            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_logical_less_or_greater(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_shift(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Lt | TokenType::Lte |
        TokenType::Gt | TokenType::Gte => {
            parser.get_token();
            let operation = Some(next.lexem);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));
                                                
            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_shift(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_add_or_sub(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::LShift | TokenType::RShift  => {
            parser.get_token();
            let operation = Some(next.lexem);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));
                                                
            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_add_or_sub(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_mul_or_div(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Plus | TokenType::Minus => {
            parser.get_token();
            let operation = Some(next.lexem);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));
                                                
            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_mul_or_div(parser: &mut Parser) -> Option<Ast> {
    let lhs = parse_factor(parser)?;

    let next = parser.peek_token();
    match next.lexem {
        TokenType::Star |
        TokenType::Slash |
        TokenType::Percent => {
            parser.get_token();
            let operation = Some(next.lexem);

            let rhs = parse_expression(parser)?;
            let rhs = Some(Box::new(rhs));
                                                
            return Some(Ast::Expression { node: Box::new(
                Expression {
                    operation,
                    lhs: Some(Box::new(lhs)),
                    rhs,
                    term: None 
                }) })
        },

        _ => {
            return Some(lhs);
        }
    }
}

fn parse_factor(parser: &mut Parser) -> Option<Ast> {
    let next = parser.peek_token();

    match next.lexem {
        //   TokenType::Plus
        // | TokenType::Minus => {
        //     // parse unary 'operators'
        // }

        TokenType::Integer(val) => {
            parser.get_token();

            return Some(Ast::Value { node: values::ValueType::Integer(val) });
        },

        TokenType::Decimal(val) => {
            parser.get_token();
            return Some(Ast::Value { node: values::ValueType::Decimal(val) })
        },

        TokenType::Identifier(identifier) => {
            parser.get_token();

            let next = parser.peek_token();
            if next.lexem == TokenType::LParen { // if call
                let arguments = calls::parse_call(parser)?;

                return Some(Ast::Value { node: values::ValueType::Call(calls::Node {
                    identifier,
                    arguments
                }) });
            }

            return Some(Ast::Value { node: values::ValueType::Identifier(identifier) });
        }

        TokenType::LParen => {
            parser.consume_token(TokenType::LParen)?;
            
            let expr = parse_expression(parser)?;

            parser.consume_token(TokenType::RParen)?;

            return Some(expr);
        }

        _ => {
            parser.error(
                    "Unexpected token within expression",
                    next.get_location()
                );

            return None;
        }

    }
}


