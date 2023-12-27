use tanit::{
    analyzer::{Analyzer, SymbolData},
    ast::types::Type,
    error_listener::ErrorListener,
    parser::Id,
};

#[test]
fn scope_test() {
    /* example:
     * Module Main {       # Main: @s
     *     func bar() { }  # bar:  @s/Main
     *     func main() {   # main: @s/Main
     *         let var = 5 # var:  @s/Main/main
     *     }
     * }
     */

    let mut analyzer = Analyzer::new(ErrorListener::new());
    analyzer.scope.push("@s"); // @s

    analyzer.add_symbol(
        &Id::from("Main"),
        analyzer.create_symbol(SymbolData::ModuleDef {
            full_name: vec!["Main".to_string()],
        }),
    );

    analyzer.scope.push("Main"); // @s/Main
    analyzer.add_symbol(
        &Id::from("main"),
        analyzer.create_symbol(SymbolData::FunctionDef {
            args: Vec::new(),
            return_type: Type::Tuple {
                components: Vec::new(),
            },
            is_declaration: true,
        }),
    );

    analyzer.add_symbol(
        &Id::from("bar"),
        analyzer.create_symbol(SymbolData::FunctionDef {
            args: Vec::new(),
            return_type: Type::Tuple {
                components: Vec::new(),
            },
            is_declaration: true,
        }),
    );

    analyzer.scope.push("main"); // @s/Main/main
    analyzer.add_symbol(
        &Id::from("var"),
        analyzer.create_symbol(SymbolData::VariableDef {
            var_type: Type::I32,
            is_mutable: false,
            is_initialization: true,
        }),
    );

    // check if var defined in main
    assert!(analyzer
        .check_identifier_existance(&Id::from("var"),)
        .is_ok());

    // check if main inside Main
    analyzer.scope.pop(); // @s/Main
    assert!(analyzer
        .check_identifier_existance(&Id::from("main"),)
        .is_ok());

    // check if baz not defined in Main
    assert!(!analyzer
        .check_identifier_existance(&Id::from("baz"),)
        .is_ok());

    // check if var unaccessible in Main
    assert!(!analyzer
        .check_identifier_existance(&Id::from("var"))
        .is_ok());

    // check if var unaccessible in bar
    analyzer.scope.push("bar");
    assert!(!analyzer
        .check_identifier_existance(&Id::from("var"))
        .is_ok());

    // check if bar accessible in bar
    assert!(analyzer
        .check_identifier_existance(&Id::from("bar"))
        .is_ok());
}
