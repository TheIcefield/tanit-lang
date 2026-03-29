use tanitc_attributes::Safety;
use tanitc_hir::hir::{blocks::Block, Hir};
use tanitc_messages::Message;

use crate::{AnalyzeResult, Analyzer};

impl Analyzer {
    pub(crate) fn analyze_block(&mut self, block: &mut Block) -> AnalyzeResult<()> {
        if block.is_global {
            self.analyze_global_block(block)?;
        } else {
            self.analyze_local_block(block)?;
        }

        Ok(())
    }

    fn analyze_global_block(&mut self, block: &mut Block) -> AnalyzeResult<()> {
        self.table.set_safety(Safety::Safe);

        for stmt in block.statements.iter_mut() {
            let is_denied = matches!(
                stmt,
                Hir::ControlFlow(_)
                    | Hir::Block(_)
                    | Hir::BranchStmt(_)
                    | Hir::Expression(_)
                    | Hir::TypeSpec(_)
            );

            if is_denied {
                self.error(Message::new(
                    stmt.location(),
                    format!(
                        "Node \"{}\" is not allowed in global scope",
                        stmt.kind_str()
                    ),
                ));

                continue;
            }

            if let Err(err) = stmt.accept_mut(self) {
                self.error(err);
            }
        }

        Ok(())
    }

    fn analyze_local_block(&mut self, block: &mut Block) -> AnalyzeResult<()> {
        let mut scope_info = self.table.get_scope_info();
        scope_info.safety = block.attributes.safety;

        self.table.enter_scope(scope_info);

        for n in block.statements.iter_mut() {
            if let Err(err) = n.accept_mut(self) {
                self.error(err);
            }
        }

        self.table.exit_scope();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_hir::hir::{
        blocks::Block,
        branches::{Branch, If},
        Hir,
    };
    use tanitc_hir_test::{
        create_block, create_integer_lit, create_main_func_def, create_struct_def,
    };
    use tanitc_lexer::location::Location;
    use tanitc_options::{CompileOptions, CrateType};

    use crate::Analyzer;

    #[test]
    fn if_in_global_scope_test() {
        // Given
        let compile_options = CompileOptions {
            crate_type: CrateType::StaticLib,
            ..Default::default()
        };

        let mut analyzer = Analyzer::with_compile_options(compile_options);

        let branch = Branch::If(If {
            location: Location::default(),
            condition: Box::new(create_integer_lit(1)),
            body: Box::new(Block {
                is_global: false,
                ..Default::default()
            }),
        });

        // if 1 { }
        let mut program = Hir::Block(create_block(vec![branch.into()]));

        // When
        let res = analyzer.analyze_program(&mut program);

        // Then
        const EXPECTED: &str = "Semantic error: Node \"branching\" is not allowed in global scope";

        let messages = res.expect_err("Expected errors");
        let errors = messages.errors_ref();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, EXPECTED);
    }

    #[test]
    fn struct_in_local_scope_test() {
        // Given
        const STRUCT_NAME: &str = "MyStruct";
        let struct_def = create_struct_def(STRUCT_NAME, vec![]);

        let main_func = create_main_func_def(vec![struct_def.into()]);

        /* func main() {
         *     struct MyStruct { }
         * }
         */
        let mut hir = Hir::Block(Block {
            is_global: true,
            statements: vec![main_func.into()],
            ..Default::default()
        });

        let mut analyzer = Analyzer::new();

        // When
        let res = analyzer.analyze_program(&mut hir);

        // Then
        res.expect("Expected no errors");
    }
}
