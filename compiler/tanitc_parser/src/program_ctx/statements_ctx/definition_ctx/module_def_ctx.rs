use std::{io::Read, path::PathBuf};

use tanitc_ast::program_ctx::statement_ctx::definition_ctx::module_def_ctx::{
    ModuleDefBodyCtx, ModuleDefCtx,
};
use tanitc_lexer::{
    token::{lexeme::Lexeme, Token},
    Lexer,
};
use tanitc_messages::Message;

use crate::{ParseResult, Parser};

impl Parser {
    pub fn parse_module_def_ctx(&mut self) -> ParseResult<ModuleDefCtx> {
        let def_tkn = self.consume_token(Lexeme::KwDef).ok();
        let module_tkn = self.consume_token(Lexeme::KwModule)?;
        let name_ctx = Box::new(self.parse_name_ctx()?);
        let body_ctx = {
            let old_opt = self.does_ignore_nl();
            self.set_ignore_nl_option(true);

            let body = if def_tkn.is_some() {
                self.parse_module_body_external(&name_ctx.name_tkn)
            } else {
                self.parse_module_body_internal()
            };

            self.set_ignore_nl_option(old_opt);
            body?
        };

        Ok(ModuleDefCtx {
            attributes_ctx: Box::default(),
            def_tkn,
            module_tkn,
            name_ctx,
            body_ctx,
        })
    }

    fn parse_module_body_internal(&mut self) -> ParseResult<ModuleDefBodyCtx> {
        Ok(ModuleDefBodyCtx::Internal(Box::new(
            self.parse_block_ctx()?,
        )))
    }

    fn get_external_module_path(&mut self, module_tkn: &Token) -> Result<PathBuf, Message> {
        let name = module_tkn.identifier().to_string();

        let Some(current_path_str) = self.get_path().to_str() else {
            return Err(Message::new(
                module_tkn.get_location(),
                "Failed to get path",
            ));
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
            return Err(Message::new(
                module_tkn.get_location(),
                format!("Module \"{name}\" not found"),
            ));
        }

        Ok(PathBuf::from(path))
    }

    fn parse_module_body_external(&mut self, module_tkn: &Token) -> ParseResult<ModuleDefBodyCtx> {
        let path = self.get_external_module_path(module_tkn)?;

        let mut file = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(err) => return Err(Message::new(module_tkn.get_location(), err.to_string())),
        };

        let mut buffer = String::new();

        let _ = file.read_to_string(&mut buffer);

        let lexer = Lexer::new(buffer.chars().peekable(), &path);

        let mut parser = Parser::new(lexer);

        Ok(ModuleDefBodyCtx::External(Box::new(
            parser.parse_program_ctx()?,
        )))
    }
}

#[cfg(test)]
mod tests {

    use tanitc_ast::program_ctx::statement_ctx::{
        definition_ctx::{module_def_ctx::ModuleDefBodyCtx, DefinitionCtx},
        StatementCtx,
    };

    use crate::Parser;

    #[test]
    fn module_test() {
        const SRC_TEXT: &str = "\nmodule M1\
                                \n{\
                                \n    unsafe module M2\
                                \n    {\
                                \n    }\
                                \n}";

        let mut parser = Parser::from_text(SRC_TEXT);

        let module_def_ctx = parser.parse_module_def_ctx().unwrap();

        assert_eq!(module_def_ctx.name_ctx.to_string(), "M1");
        let ModuleDefBodyCtx::Internal(internal_ctx) = &module_def_ctx.body_ctx else {
            panic!("Unexpected: {}", module_def_ctx.body_ctx.kind_str());
        };

        assert_eq!(internal_ctx.statements_ctx.statements.len(), 2);

        {
            let (None, None) = &internal_ctx.statements_ctx.statements[0] else {
                panic!("Unexpected statement");
            };
        }

        {
            let (Some(StatementCtx::Definition(DefinitionCtx::Module(module_def_ctx))), Some(_)) =
                &internal_ctx.statements_ctx.statements[1]
            else {
                panic!("Unexpected statement");
            };
            assert!(module_def_ctx.attributes_ctx.unsafe_tkn.is_some());
            assert_eq!(module_def_ctx.name_ctx.to_string(), "M2");

            let ModuleDefBodyCtx::Internal(internal_ctx) = &module_def_ctx.body_ctx else {
                panic!("Unexpected: {}", module_def_ctx.body_ctx.kind_str());
            };

            assert_eq!(internal_ctx.statements_ctx.statements.len(), 1);

            {
                let (None, None) = &internal_ctx.statements_ctx.statements[0] else {
                    panic!("Unexpected statement");
                };
            }
        }
    }
}
