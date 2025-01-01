use super::FunctionDef;
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};
use crate::ast::{types::Type, Ast};
use crate::messages::Message;

impl Analyze for FunctionDef {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if let Ok(_ss) = analyzer.check_identifier_existance(&self.identifier) {
            return Err(Message::multiple_ids(
                self.location,
                &self.identifier.get_string(),
            ));
        }

        analyzer.scope.push(&format!("@f.{}", &self.identifier));

        let mut arguments = Vec::<(String, Type)>::new();
        for p in self.parameters.iter_mut() {
            if let Ast::VariableDef { node } = p {
                arguments.push((node.identifier.get_string(), node.var_type.clone()));
                p.analyze(analyzer)?;
            }
        }

        analyzer.scope.pop();

        analyzer.add_symbol(
            &self.identifier,
            analyzer.create_symbol(SymbolData::FunctionDef {
                args: arguments.clone(),
                return_type: self.return_type.clone(),
                is_declaration: self.body.is_some(),
            }),
        );

        analyzer.scope.push(&format!("@f.{}", &self.identifier));

        if let Some(body) = &mut self.body {
            if let Ast::Scope { node } = body.as_mut() {
                for stmt in node.statements.iter_mut() {
                    stmt.analyze(analyzer)?;
                }
            }
        }

        analyzer.scope.pop();

        Ok(())
    }
}
