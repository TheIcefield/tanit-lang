use tanitc_analyzer::Analyzer;
use tanitc_ast::ast::{blocks::Block, functions::FunctionDef, types::TypeSpec, Ast};
use tanitc_ident::Name;
use tanitc_ty::Type;

fn get_func(name: &str, return_type: Type) -> FunctionDef {
    FunctionDef {
        name: Name::from(name.to_string()),
        return_type: TypeSpec {
            ty: return_type,
            ..Default::default()
        },
        parameters: vec![],
        body: Some(Box::new(Block::default())),
        ..Default::default()
    }
}

#[test]
fn main_existing_bad_test() {
    const EXPECTED_ERR: &str = "No entry point!";

    let mut program = Ast::from(Block {
        is_global: true,
        statements: vec![
            get_func("func_1", Type::I32).into(),
            get_func("func_2", Type::F64).into(),
        ],
        ..Default::default()
    });

    let mut analyzer = Analyzer::new();
    program.accept_mut(&mut analyzer).unwrap();
    if analyzer.has_errors() {
        panic!("{:#?}", analyzer.get_errors());
    }

    let Err(msg) = analyzer.check_entry_point() else {
        panic!("Expected error");
    };

    assert_eq!(msg.text, EXPECTED_ERR);
}

#[test]
fn main_existing_good_test() {
    let mut program = Ast::from(Block {
        is_global: true,
        statements: vec![
            get_func("func_1", Type::F32).into(),
            get_func("main", Type::I32).into(),
        ],
        ..Default::default()
    });

    let mut analyzer = Analyzer::new();
    program.accept_mut(&mut analyzer).unwrap();
    if analyzer.has_errors() {
        panic!("{:#?}", analyzer.get_errors());
    }

    analyzer.check_entry_point().unwrap();
}

#[test]
fn main_bad_type_test() {
    const EXPECTED_ERR: &str = "Bad type of main function: f64";

    let mut program = Ast::from(Block {
        is_global: true,
        statements: vec![
            get_func("func_1", Type::I32).into(),
            get_func("main", Type::F64).into(),
        ],
        ..Default::default()
    });

    let mut analyzer = Analyzer::new();
    program.accept_mut(&mut analyzer).unwrap();
    if analyzer.has_errors() {
        panic!("{:#?}", analyzer.get_errors());
    }

    let Err(msg) = analyzer.check_entry_point() else {
        panic!("Expected error");
    };

    assert_eq!(msg.text, EXPECTED_ERR);
}

#[test]
fn main_good_type_i32_test() {
    let mut program = Ast::from(Block {
        is_global: true,
        statements: vec![
            get_func("func_1", Type::I32).into(),
            get_func("main", Type::I32).into(),
        ],
        ..Default::default()
    });

    let mut analyzer = Analyzer::new();

    program.accept_mut(&mut analyzer).unwrap();
    if analyzer.has_errors() {
        panic!("{:#?}", analyzer.get_errors());
    }

    analyzer.check_entry_point().unwrap();
}

#[test]
fn main_good_type_unit_test() {
    let mut program = Ast::from(Block {
        is_global: true,
        statements: vec![
            get_func("func_1", Type::I32).into(),
            get_func("main", Type::unit()).into(),
        ],
        ..Default::default()
    });

    let mut analyzer = Analyzer::new();

    program.accept_mut(&mut analyzer).unwrap();
    if analyzer.has_errors() {
        panic!("{:#?}", analyzer.get_errors());
    }

    analyzer.check_entry_point().unwrap();
}
