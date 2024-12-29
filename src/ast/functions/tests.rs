use super::FunctionDef;
use crate::ast::{expressions::ExpressionType, Ast};
use crate::parser::{lexer::Lexer, token::Lexem, Parser};

#[test]
fn functions_test() {
    static SRC_PATH: &str = "./examples/functions.tt";

    let lexer = Lexer::from_file(SRC_PATH, true).unwrap();

    let mut parser = Parser::new(lexer);

    {
        let func = FunctionDef::parse(&mut parser).unwrap();

        let scope = if let Ast::FuncDef { node } = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let Ast::Scope { node } = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        assert!(matches!(node, Ast::ReturnStmt { .. }));
    }

    {
        let func = FunctionDef::parse(&mut parser).unwrap();

        let scope = if let Ast::FuncDef { node } = &func {
            node.body.as_ref()
        } else {
            panic!("node should be \'FuncDef\'");
        };

        let node = if let Ast::Scope { node } = scope.unwrap().as_ref() {
            assert_eq!(node.statements.len(), 1);
            &node.statements[0]
        } else {
            panic!("node should be \'local scope\'");
        };

        let (lhs, rhs) = if let Ast::Expression { node } = node {
            if let ExpressionType::Binary {
                operation,
                lhs,
                rhs,
            } = &node.as_ref().expr
            {
                assert_eq!(*operation, Lexem::Assign);
                (lhs.as_ref(), rhs.as_ref())
            } else {
                panic!("Expression expected to be binary");
            }
        } else {
            panic!("Expected expression");
        };

        assert!(matches!(lhs, Ast::VariableDef { .. }));
        assert!(matches!(rhs, Ast::Value { .. }));
    }
}
