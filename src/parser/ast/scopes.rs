use crate::lexer::TokenType;
use crate::parser::{ast, ast::Ast, Parser};

#[derive(Clone)]
pub struct Scope {
    pub statements: Vec<Ast>
}

impl ast::IAst for Scope {
    fn traverse(&self, stream: &mut ast::Stream, intent: usize) -> std::io::Result<()> {
        for stmt in self.statements.iter() {
            stmt.traverse(stream, intent + 1)?;
        }

        Ok(())
    }
}


pub fn parse_global_external(parser: &mut Parser) -> Option<Scope> {
    parser.consume_token(TokenType::Lcb)?;

    let statements = parse_global_internal(parser)?;

    parser.consume_token(TokenType::Rcb)?;

    Some(Scope { statements })
}

pub fn parse_global_internal(parser: &mut Parser) -> Option<Vec<Ast>> {
    let mut children = Vec::<Ast>::new();

    loop {
        let next = parser.peek_token();

        let child = match next.lexem {
            TokenType::Rcb | TokenType::EndOfFile => {
                break;
            },

            TokenType::EndOfLine => {
                parser.get_token();
                continue;
            },

            TokenType::KwModule => ast::modules::parse(parser)?,

            TokenType::KwFunc => ast::functions::parse_func_def(parser)?,
            
            TokenType::KwStruct => ast::structs::parse_struct_def(parser)?,
            
            // TokenType::KwStatic => {
                
            //     ast::variables::parse_def_stmt(parser)?
            // },

            // TokenType::KwExtern => ast::externs::parse(parser)?,

            TokenType::KwAlias => ast::aliases::parse(parser)?,

            _ => {
                parser.error(&format!(
                    "Unexpected token \"{}\"", next),
                    next.get_location());
                return None;
            }
        };

        children.push(child);
    }

    Some(children)
}


pub fn parse_local_external(parser: &mut Parser) -> Option<Scope> {
    parser.consume_token(TokenType::Lcb)?;

    let statements = parse_local_internal(parser)?;

    parser.consume_token(TokenType::Rcb)?;

    Some(Scope{statements})
}

pub fn parse_local_internal(parser: &mut Parser) -> Option<Vec<Ast>> {
    let mut children = Vec::<Ast>::new();

    loop {
        let next = parser.peek_token();

        let child = match next.lexem {
            TokenType::Rcb => break,

            TokenType::EndOfLine => {
                parser.get_token();
                continue;
            },

            TokenType::KwLet => ast::modules::parse(parser)?,

            TokenType::KwStruct => ast::structs::parse_struct_def(parser)?,

            // TokenType::KwStatic => ast::variable_node::parse_def_stmt(parser)?,

            TokenType::KwAlias => ast::aliases::parse(parser)?,

            // TokenType::KwIf => ast::branch_node::parse_if(parser)?,

            TokenType::KwLoop => ast::branches::parse_loop(parser)?,

            TokenType::KwWhile => ast::branches::parse_while(parser)?,

            // TokenType::KwFor => ast::branch_node::parse_for(parser)?,

            TokenType::KwReturn => ast::branches::parse_return(parser)?,

            TokenType::KwBreak => ast::branches::parse_break(parser)?,

            TokenType::KwContinue => ast::branches::parse_continue(parser)?,

            TokenType::Identifier(_) => ast::expressions::parse_expression(parser)?,

            TokenType::EndOfFile => {
                parser.error("Unexpected end of file", next.get_location());
                return None;
            },

            _ => {
                parser.error(&format!(
                    "Unexpected token \"{}\"", next),
                    next.get_location());
                return None;
            }
        };

        children.push(child);

        parser.consume_new_line()?;
    }

    Some(children)
}
