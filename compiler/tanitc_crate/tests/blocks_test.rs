use tanitc_analyzer::Analyzer;
use tanitc_parser::Parser;

use pretty_assertions::assert_str_eq;

#[test]
fn struct_in_local_scope_test() {
    const SRC_TEXT: &str = "\nfunc main() {\
                            \n    struct MyStruct {\
                            \n        f1: i32\
                            \n        f2: f32\
                            \n    }\
                            \n\
                            \n    var s = MyStruct { \
                            \n              f1: 1, f2: 2.0\
                            \n            }\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Failed to create parser");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:?}", parser.get_errors());
        }
    }

    {
        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();
        assert!(analyzer.has_errors());
    }
}

#[test]
fn if_in_global_scope_test() {
    const SRC_TEXT: &str = "\nif 1 > 0 {\
                            \n    var a = 1\
                            \n} else {\
                            \n    var b = 2\
                            \n}";

    let mut parser = Parser::from_text(SRC_TEXT).expect("Failed to create parser");

    let mut program = parser.parse_global_block().unwrap();
    {
        if parser.has_errors() {
            panic!("{:#?}", parser.get_errors());
        }
    }

    {
        const EXPECTED: &str = "Semantic error: Node \"branching\" is not allowed in global scope";

        let mut analyzer = Analyzer::new();
        program.accept_mut(&mut analyzer).unwrap();

        let errors = analyzer.get_errors();
        assert_eq!(errors.len(), 2);
        assert_str_eq!(errors[0].text, EXPECTED);
        assert_str_eq!(errors[1].text, EXPECTED);
    }
}
