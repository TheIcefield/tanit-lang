use super::FunctionDef;
use crate::analyzer::{symbol_table::SymbolData, Analyze, Analyzer};
use crate::ast::Ast;

use tanitc_ident::Ident;
use tanitc_messages::Message;
use tanitc_ty::Type;

impl Analyze for FunctionDef {
    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message> {
        if analyzer.has_symbol(self.identifier) {
            return Err(Message::multiple_ids(self.location, self.identifier));
        }

        analyzer.scope.push(&format!("@f.{}", &self.identifier));

        let mut parameters = Vec::<(Ident, Type)>::new();
        for p in self.parameters.iter_mut() {
            if let Ast::VariableDef(node) = p {
                parameters.push((node.identifier, node.var_type.get_type()));
                p.analyze(analyzer)?;
            }
        }

        analyzer.scope.pop();

        analyzer.add_symbol(
            self.identifier,
            analyzer.create_symbol(SymbolData::FunctionDef {
                parameters,
                return_type: self.return_type.get_type(),
                is_declaration: self.body.is_some(),
            }),
        );

        analyzer.scope.push(&format!("@f.{}", &self.identifier));

        if let Some(body) = &mut self.body {
            if let Ast::Scope(scope) = body.as_mut() {
                for stmt in scope.statements.iter_mut() {
                    stmt.analyze(analyzer)?;
                }
            }
        }

        analyzer.scope.pop();

        Ok(())
    }
}
