use std::path::PathBuf;

use tanitc_ast::ast::{modules::ModuleDef, Ast};
use tanitc_lexer::{token::Lexem, Lexer};
use tanitc_messages::Message;

use crate::Parser;

impl Parser {
    pub fn parse_module_def(&mut self) -> Result<Ast, Message> {
        let mut node = ModuleDef::default();

        self.parse_module_header(&mut node)?;
        self.parse_module_body(&mut node)?;

        Ok(Ast::from(node))
    }

    fn parse_module_header(&mut self, mod_def: &mut ModuleDef) -> Result<(), Message> {
        let next = self.peek_token();
        mod_def.location = next.location;

        if Lexem::KwDef == next.lexem {
            self.consume_token(Lexem::KwDef)?;
            mod_def.is_external = true;
        }

        self.consume_token(Lexem::KwModule)?;

        mod_def.identifier = self.consume_identifier()?;

        Ok(())
    }

    fn parse_module_body(&mut self, mod_def: &mut ModuleDef) -> Result<(), Message> {
        if !mod_def.is_external {
            self.parse_module_body_internal(mod_def)
        } else {
            self.parse_module_body_external(mod_def)
        }
    }

    fn parse_module_body_internal(&mut self, mod_def: &mut ModuleDef) -> Result<(), Message> {
        self.consume_token(Lexem::Lcb)?;

        let block = self.parse_global_block()?;

        self.consume_token(Lexem::Rcb)?;

        let Ast::Block(block) = block else {
            return Err(Message::unreachable(
                mod_def.location,
                format!("expected Block, actually {}", block.name()),
            ));
        };

        mod_def.body = Box::new(block);

        Ok(())
    }

    fn get_external_module_path(&mut self, mod_def: &mut ModuleDef) -> Result<PathBuf, Message> {
        let name: String = mod_def.identifier.into();

        let Some(current_path_str) = self.get_path().to_str() else {
            return Err(Message::new(mod_def.location, "Failed to get path"));
        };

        let mut path = current_path_str
            .chars()
            .rev()
            .collect::<String>()
            .splitn(2, '/')
            .collect::<Vec<&str>>()[1]
            .chars()
            .rev()
            .collect::<String>();

        path.push('/');
        path.push_str(&name);

        let mut file_exists: bool;

        {
            // Try to search in external_module.tt
            let mut path = path.clone();
            path.push_str(".tt");

            file_exists = std::path::Path::new(&path).exists();
            if file_exists {
                return Ok(PathBuf::from(path));
            }
        }

        if !file_exists {
            // Try to search in external_module/mod.tt
            let mut path = path.clone();
            path.push_str("/mod.tt");

            file_exists = std::path::Path::new(&path).exists();
            if file_exists {
                return Ok(PathBuf::from(path));
            }
        }

        if !file_exists {
            return Err(Message::from_string(
                mod_def.location,
                format!("Module \"{name}\" not found"),
            ));
        }

        Ok(PathBuf::from(path))
    }

    fn parse_module_body_external(&mut self, mod_def: &mut ModuleDef) -> Result<(), Message> {
        let path = self.get_external_module_path(mod_def)?;

        let lexer = match Lexer::from_file(&path) {
            Ok(lexer) => lexer,
            Err(msg) => return Err(Message::from_string(mod_def.location, msg)),
        };

        let mut parser = Parser::new(lexer);

        let block = parser.parse_global_block()?;

        let Ast::Block(block) = block else {
            return Err(Message::unreachable(
                mod_def.location,
                format!("expected Block, actually {}", block.name()),
            ));
        };

        mod_def.body = Box::new(block);

        Ok(())
    }
}

#[test]
fn module_test() {
    const SRC_TEXT: &str = "\nmodule M1\
                            \n{\
                            \n    unsafe module M2\
                            \n    {\
                            \n    }\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).unwrap();

    let res = parser.parse_module_def().unwrap();

    let Ast::ModuleDef(node) = &res else {
        panic!("Expected ModuleDef, actually: {}", res.name());
    };

    assert_eq!(node.identifier.to_string(), "M1");

    let body = node.body.as_ref();
    let Ast::ModuleDef(node) = &body.statements[0] else {
        panic!(
            "Expected ModuleDef, actually: {}",
            &body.statements[0].name()
        );
    };

    assert_eq!(node.identifier.to_string(), "M2");
}
