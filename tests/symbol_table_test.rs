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
        Symbol {
            data: SymbolData::ModuleDef { full_name: vec!["Main".to_string()] },
            scope: vec!["g_1".to_string()],
        },
    );

    table.insert(
        "main",
        Symbol {
            data: SymbolData::FunctionDef {
                args: Vec::new(),
                return_type: Type::Tuple {
                    components: Vec::new(),
                },
                is_declaration: true
            },
            scope: vec!["g_1".to_string(), "Main".to_string()],
        },
    );

    table.insert(
        "bar",
        Symbol {
            data: SymbolData::FunctionDef {
                args: Vec::new(),
                return_type: Type::Tuple {
                    components: Vec::new(),
                },
                is_declaration: true
            },
            scope: vec!["@s".to_string(), "Main".to_string()],
        },
    );

    table.insert(
        "var",
        Symbol {
            data: SymbolData::VariableDef { var_type: Type::I32, is_initialization: true },
            scope: vec!["@s".to_string(), "Main".to_string(), "main".to_string()],
        },
    );

    // check if main inside Main
    assert!(table.check_identifier_existance("main", &vec!["g_1".to_string(), "Main".to_string()]).is_some());

    // check if baz not defined in Main
    assert!(!table.check_identifier_existance("baz", &vec!["g_1".to_string(), "Main".to_string()]).is_some());

    // check if var defined in main
    assert!(table.check_identifier_existance(
        "var",
        &vec!["g_1".to_string(), "Main".to_string(), "main".to_string()]
    ).is_some());

    // check if var unaccessible in Main
    assert!(!table.check_identifier_existance("var", &vec!["g_1".to_string(), "Main".to_string()]).is_some());

    // check if var unaccessible in bar
    assert!(!table.check_identifier_existance(
        "var",
        &vec!["g_1".to_string(), "Main".to_string(), "bar".to_string()]
    ).is_some());

    // check if bar accessible in bar
    assert!(table.check_identifier_existance(
        "bar",
        &vec!["g_1".to_string(), "Main".to_string(), "bar".to_string()]
    ).is_some());
}
