use tanit::{
    analyzer::{Symbol, SymbolData, SymbolTable},
    ast::types::Type,
    error_listener::ErrorListener,
};

#[test]
fn scope_test() {
    /* example:
     * Module Main {       # Main: g_scope
     *     func bar() { }  # bar:  g_scope/Main
     *     func main() {   # main: g_scope/Main
     *         let var = 5 # var:  g_scope/Main/main
     *     }
     * }
     */

    let mut table = SymbolTable::new(ErrorListener::new());

    table.insert(
        "Main",
        Symbol::Declaration {
            stype: SymbolData::Module,
            scope: vec!["g_1".to_string()],
        },
    );

    table.insert(
        "main",
        Symbol::Definition {
            stype: SymbolData::Function {
                args: Vec::new(),
                return_type: Type::Tuple {
                    components: Vec::new(),
                },
            },
            scope: vec!["g_1".to_string(), "Main".to_string()],
        },
    );

    table.insert(
        "bar",
        Symbol::Definition {
            stype: SymbolData::Function {
                args: Vec::new(),
                return_type: Type::Tuple {
                    components: Vec::new(),
                },
            },
            scope: vec!["g_1".to_string(), "Main".to_string()],
        },
    );

    table.insert(
        "var",
        Symbol::Definition {
            stype: SymbolData::Variable,
            scope: vec!["g_1".to_string(), "Main".to_string(), "main".to_string()],
        },
    );

    // check if main inside Main
    assert!(table.check_identifier_existance("main", &vec!["g_1".to_string(), "Main".to_string()]));

    // check if baz not defined in Main
    assert!(!table.check_identifier_existance("baz", &vec!["g_1".to_string(), "Main".to_string()]));

    // check if var defined in main
    assert!(table.check_identifier_existance(
        "var",
        &vec!["g_1".to_string(), "Main".to_string(), "main".to_string()]
    ));

    // check if var unaccessible in Main
    assert!(!table.check_identifier_existance("var", &vec!["g_1".to_string(), "Main".to_string()]));

    // check if var unaccessible in bar
    assert!(!table.check_identifier_existance(
        "var",
        &vec!["g_1".to_string(), "Main".to_string(), "bar".to_string()]
    ));

    // check if bar accessible in bar
    assert!(table.check_identifier_existance(
        "bar",
        &vec!["g_1".to_string(), "Main".to_string(), "bar".to_string()]
    ));
}
