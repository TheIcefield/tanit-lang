use tanitc_ident::Ident;
use tanitc_messages::{Errors, Message, Warnings};
use tanitc_ty::Type;

pub mod ast;
pub mod scope;
pub mod symbol;
pub mod symbol_table;

use scope::{Counter, Scope};
use symbol::{Symbol, SymbolData};
use symbol_table::SymbolTable;

pub trait Analyze {
    fn get_type(&self, _analyzer: &mut Analyzer) -> Type {
        Type::unit()
    }

    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message>;
}

pub struct Analyzer {
    pub table: SymbolTable,
    pub scope: Scope,
    counter: Counter,
    errors: Errors,
    warnings: Warnings,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            table: SymbolTable::new(),
            scope: Scope::new(),
            counter: 0,
            errors: Errors::new(),
            warnings: Warnings::new(),
        }
    }

    pub fn counter(&mut self) -> Counter {
        let old = self.counter;
        self.counter += 1;
        old
    }

    pub fn get_table(&self) -> &SymbolTable {
        &self.table
    }

    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.table.insert(symbol.id, symbol);
    }

    pub fn get_symbols(&self, id: &Ident) -> Option<&Vec<Symbol>> {
        self.table.get(id)
    }

    pub fn get_symbols_mut(&mut self, id: &Ident) -> Option<&mut Vec<Symbol>> {
        self.table.get_mut(id)
    }

    pub fn create_symbol(&self, id: Ident, data: SymbolData) -> Symbol {
        Symbol {
            id,
            scope: self.scope.clone(),
            data,
        }
    }

    pub fn get_first_symbol(&self, id: Ident) -> Option<Symbol> {
        if let Some(ss) = self.table.get(&id) {
            for s in ss.iter() {
                if self.scope.0.starts_with(&s.scope.0) {
                    return Some(s.clone());
                }
            }
        }

        None
    }

    pub fn has_symbol(&self, id: Ident) -> bool {
        self.get_first_symbol(id).is_some()
    }

    pub fn error(&mut self, mut error: Message) {
        error.text = format!("Semantic error: {}", error.text);
        self.errors.push(error);
    }

    pub fn warning(&mut self, mut warn: Message) {
        warn.text = format!("Semantic warning: {}", warn.text);
        self.warnings.push(warn);
    }

    pub fn get_errors(&mut self) -> Errors {
        std::mem::take(&mut self.errors)
    }

    pub fn get_warnings(&mut self) -> Warnings {
        std::mem::take(&mut self.warnings)
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

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

    use scope::ScopeUnit;

    let main_mod_id = Ident::from("Main".to_string());
    let bar_id = Ident::from("bar".to_string());
    let main_fn_id = Ident::from("main".to_string());
    let var_id = Ident::from("var".to_string());
    let baz_id = Ident::from("baz".to_string());

    let mut analyzer = Analyzer::new();
    analyzer.scope.push(ScopeUnit::Block(0)); // block-0

    analyzer.add_symbol(analyzer.create_symbol(main_mod_id, SymbolData::ModuleDef));

    analyzer.scope.push(ScopeUnit::Module(main_mod_id)); // block-0/Main
    analyzer.add_symbol(analyzer.create_symbol(
        main_fn_id,
        SymbolData::FunctionDef {
            parameters: Vec::new(),
            return_type: Type::unit(),
            is_declaration: true,
        },
    ));

    analyzer.add_symbol(analyzer.create_symbol(
        bar_id,
        SymbolData::FunctionDef {
            parameters: Vec::new(),
            return_type: Type::unit(),
            is_declaration: true,
        },
    ));

    analyzer.scope.push(ScopeUnit::Func(main_fn_id)); // block-0/Main/main
    analyzer.add_symbol(analyzer.create_symbol(
        var_id,
        SymbolData::VariableDef {
            var_type: Type::I32,
            is_mutable: false,
            is_initialization: true,
        },
    ));

    // check if var defined in main
    assert!(analyzer.has_symbol(var_id));

    // check if main inside Main
    analyzer.scope.pop(); // @s/Main
    assert!(analyzer.has_symbol(main_fn_id));

    // check if baz not defined in Main
    assert!(!analyzer.has_symbol(baz_id));

    // check if var unaccessible in Main
    assert!(!analyzer.has_symbol(var_id));

    // check if var unaccessible in bar
    analyzer.scope.push(ScopeUnit::Func(bar_id));
    assert!(!analyzer.has_symbol(var_id));

    // check if bar accessible in bar
    assert!(analyzer.has_symbol(bar_id));
}

#[test]
fn symbol_access_test() {
    /* example:
     * Module M1 {       # M1
     *     func f1() { } # M1/f1
     *     func f2() { } # M1/f2
     * }
     * Module M2 {       # M2
     *     func f2() { } # M2/f2
     * }
     */

    use scope::ScopeUnit;

    let m1_id = Ident::from("M1".to_string());
    let m2_id = Ident::from("M2".to_string());
    let f1_id = Ident::from("f1".to_string());
    let f2_id = Ident::from("f2".to_string());

    let mut analyzer = Analyzer::new();

    analyzer.add_symbol(analyzer.create_symbol(m1_id, SymbolData::ModuleDef));

    analyzer.add_symbol(analyzer.create_symbol(m2_id, SymbolData::ModuleDef));

    analyzer.scope.push(ScopeUnit::Module(m1_id)); // /M1
    analyzer.add_symbol(analyzer.create_symbol(
        f1_id,
        SymbolData::FunctionDef {
            parameters: Vec::new(),
            return_type: Type::unit(),
            is_declaration: true,
        },
    ));

    analyzer.add_symbol(analyzer.create_symbol(
        f2_id,
        SymbolData::FunctionDef {
            parameters: Vec::new(),
            return_type: Type::unit(),
            is_declaration: true,
        },
    ));

    analyzer.scope.pop(); // /
    analyzer.scope.push(ScopeUnit::Module(m2_id)); // /M2

    analyzer.add_symbol(analyzer.create_symbol(
        f2_id,
        SymbolData::FunctionDef {
            parameters: Vec::new(),
            return_type: Type::unit(),
            is_declaration: true,
        },
    ));

    analyzer.scope.pop(); // /

    let res = analyzer
        .table
        .access_symbol(&[m1_id, f2_id], &analyzer.scope);
    assert_eq!(res.len(), 1);

    let res = analyzer
        .table
        .access_symbol(&[m2_id, f1_id], &analyzer.scope);
    assert_eq!(res.len(), 0);
}
