use crate::ast::{identifiers::Identifier, types::Type, Ast};
use crate::messages::{Errors, Message, Warnings};

pub mod scope;
pub mod symbol_table;

use scope::Scope;
use symbol_table::{Symbol, SymbolData, SymbolTable};

pub trait Analyze {
    fn get_type(&self, _analyzer: &mut Analyzer) -> Type {
        Type::unit()
    }

    fn analyze(&mut self, analyzer: &mut Analyzer) -> Result<(), Message>;
}

pub struct Analyzer {
    table: SymbolTable,
    pub scope: Scope,
    counter: usize,
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

    pub fn analyze(&mut self, ast: &mut Ast) -> (SymbolTable, Errors, Warnings) {
        let table = {
            if ast.analyze(self).is_ok() {
                Ok(std::mem::take(&mut self.table))
            } else {
                Err((
                    std::mem::take(&mut self.errors),
                    std::mem::take(&mut self.warnings),
                ))
            }
        };

        if let Ok(table) = table {
            return (table, self.errors.clone(), self.warnings.clone());
        }

        (
            SymbolTable::new(),
            self.errors.clone(),
            self.warnings.clone(),
        )
    }

    pub fn counter(&mut self) -> usize {
        let old = self.counter;
        self.counter += 1;
        old
    }

    pub fn get_table(&self) -> &SymbolTable {
        &self.table
    }

    pub fn add_symbol(&mut self, id: &Identifier, symbol: Symbol) {
        self.table.insert(id, symbol);
    }

    pub fn get_symbols(&self, id: &Identifier) -> Option<&Vec<Symbol>> {
        self.table.get(id)
    }

    pub fn get_symbols_mut(&mut self, id: &Identifier) -> Option<&mut Vec<Symbol>> {
        self.table.get_mut(id)
    }

    pub fn create_symbol(&self, data: SymbolData) -> Symbol {
        Symbol {
            scope: self.scope.clone(),
            data,
        }
    }

    pub fn is_built_in_identifier(id: &Identifier) -> bool {
        use crate::ast::identifiers::IdentifierType;

        if let IdentifierType::Common(id) = &id.identifier {
            return id.starts_with("__tanit_compiler__");
        }

        false
    }

    pub fn check_identifier_existance(&self, id: &Identifier) -> Result<Symbol, Message> {
        if let Some(ss) = self.table.get(id) {
            for s in ss.iter() {
                if self.scope.0.starts_with(&s.scope.0) {
                    return Ok(s.clone());
                }
            }
        }

        Err(Message::new(
            id.location,
            &format!("Identifier {} not found in this scope", id),
        ))
    }

    pub fn error(&mut self, mut error: Message) {
        error.text = format!("Semantic error: {}", error.text);
        self.errors.push(error);
    }

    pub fn warning(&mut self, mut warn: Message) {
        warn.text = format!("Semantic warning: {}", warn.text);
        self.warnings.push(warn);
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

    use std::str::FromStr;
    let mut analyzer = Analyzer::new();
    analyzer.scope.push("@s"); // @s

    analyzer.add_symbol(
        &Identifier::from_str("Main").unwrap(),
        analyzer.create_symbol(SymbolData::ModuleDef {
            full_name: vec!["Main".to_string()],
        }),
    );

    analyzer.scope.push("Main"); // @s/Main
    analyzer.add_symbol(
        &Identifier::from_str("main").unwrap(),
        analyzer.create_symbol(SymbolData::FunctionDef {
            args: Vec::new(),
            return_type: Type::Tuple {
                components: Vec::new(),
            },
            is_declaration: true,
        }),
    );

    analyzer.add_symbol(
        &Identifier::from_str("bar").unwrap(),
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
        &Identifier::from_str("var").unwrap(),
        analyzer.create_symbol(SymbolData::VariableDef {
            var_type: Type::I32,
            is_mutable: false,
            is_initialization: true,
        }),
    );

    // check if var defined in main
    assert!(analyzer
        .check_identifier_existance(&Identifier::from_str("var").unwrap())
        .is_ok());

    // check if main inside Main
    analyzer.scope.pop(); // @s/Main
    assert!(analyzer
        .check_identifier_existance(&Identifier::from_str("main").unwrap())
        .is_ok());

    // check if baz not defined in Main
    assert!(!analyzer
        .check_identifier_existance(&Identifier::from_str("baz").unwrap())
        .is_ok());

    // check if var unaccessible in Main
    assert!(!analyzer
        .check_identifier_existance(&Identifier::from_str("var").unwrap())
        .is_ok());

    // check if var unaccessible in bar
    analyzer.scope.push("bar");
    assert!(!analyzer
        .check_identifier_existance(&Identifier::from_str("var").unwrap())
        .is_ok());

    // check if bar accessible in bar
    assert!(analyzer
        .check_identifier_existance(&Identifier::from_str("bar").unwrap())
        .is_ok());
}
