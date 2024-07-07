use std::collections::HashMap;
use std::fmt::Debug;

use crate::ast::{identifiers::Identifier, structs, structs::EnumField, types, values, Ast, IAst};
use crate::error_listener::{
    ErrorListener, ANALYZING_FAILED_ERROR_STR, FUNCTION_NOT_FOUND_ERROR_STR,
    IDENTIFIER_NOT_FOUND_ERROR_STR, MISMATCHED_TYPES_ERROR_STR, UNEXPECTED_NODE_PARSED_ERROR_STR,
    WRONG_CALL_ARGUMENTS_ERROR_STR,
};

use std::io::Write;

#[derive(Clone)]
pub struct Scope(pub Vec<String>);

impl Scope {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, block: &str) {
        self.0.push(block.to_string());
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, String> {
        self.0.iter_mut()
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for i in self.0.iter() {
            write!(f, "/{}", i)?;
        }
        write!(f, "]")
    }
}

#[derive(Clone)]
pub enum SymbolData {
    ModuleDef {
        full_name: Vec<String>,
    },
    StructDef {
        components: Vec<types::Type>,
    },
    EnumDef {
        components: Vec<structs::EnumField>,
    },
    FunctionDef {
        args: Vec<types::Type>,
        return_type: types::Type,
        is_declaration: bool,
    },
    VariableDef {
        var_type: types::Type,
        is_mutable: bool,
        is_initialization: bool,
    },
    Type,
}

impl SymbolData {
    pub fn traverse(&self, stream: &mut std::fs::File) -> std::io::Result<()> {
        match self {
            Self::ModuleDef { full_name } => write!(stream, "{:?}", full_name),

            Self::FunctionDef {
                args,
                return_type,
                is_declaration,
            } => {
                write!(
                    stream,
                    "Function {}: ( ",
                    if *is_declaration {
                        "declaration"
                    } else {
                        "definition"
                    }
                )?;

                for arg in args.iter() {
                    write!(stream, "{} ", arg)?;
                }

                write!(stream, ") -> {}", return_type)
            }

            Self::StructDef { components } => {
                write!(stream, "Struct definition: {{{:?}}}", components)
            }

            Self::EnumDef { components } => {
                write!(stream, "Enum definition: <")?;

                for comp in components.iter() {
                    match comp {
                        EnumField::Common => write!(stream, "common ")?,
                        EnumField::TupleLike(t) => {
                            write!(stream, "( ")?;
                            for tc in t.iter() {
                                write!(stream, "{} ", *tc)?;
                            }
                            write!(stream, ")")?;
                        }
                        EnumField::StructLike(s) => {
                            write!(stream, "{{ ")?;
                            for sc in s.iter() {
                                write!(stream, "{} ", *sc.1)?;
                            }
                            write!(stream, "}}")?;
                        }
                    }
                }

                write!(stream, ">")
            }

            Self::VariableDef {
                var_type,
                is_mutable,
                is_initialization,
            } => write!(
                stream,
                "Variable {}: {} {}",
                if *is_initialization {
                    "initialization"
                } else {
                    "definition"
                },
                if *is_mutable { "mut" } else { "" },
                var_type
            ),

            Self::Type => write!(stream, "type"),
        }
    }
}

#[derive(Clone)]
pub struct Symbol {
    pub scope: Scope,
    pub data: SymbolData,
}

#[derive(Clone)]
pub struct SymbolTable {
    table: HashMap<Identifier, Vec<Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn get(&self, id: &Identifier) -> Option<&Vec<Symbol>> {
        self.table.get(id)
    }

    pub fn get_mut(&mut self, id: &Identifier) -> Option<&mut Vec<Symbol>> {
        self.table.get_mut(id)
    }

    pub fn insert(&mut self, id: &Identifier, symbol: Symbol) {
        if !self.table.contains_key(id) {
            self.table.insert(id.clone(), Vec::new());
        }

        if let Some(ss) = self.table.get_mut(id) {
            ss.push(symbol)
        }
    }

    pub fn traverse(&self, stream: &mut std::fs::File) -> std::io::Result<()> {
        for (identifier, ss) in self.table.iter() {
            writeln!(stream, "Identifier: \"{}\":", identifier)?;

            for s in ss.iter() {
                write!(stream, "+--- ")?;

                s.data.traverse(stream)?;

                writeln!(stream, " at {:?}", s.scope)?;
            }
        }
        Ok(())
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

pub fn dump_symtable(output: &str, symbol_table: &SymbolTable) -> std::io::Result<()> {
    let mut stream = std::fs::File::create(format!("{}_symbol_table.txt", output)).unwrap();
    symbol_table.traverse(&mut stream)
}

pub struct Analyzer {
    table: SymbolTable,
    pub scope: Scope,
    counter: usize,
    error_listener: ErrorListener,
}

impl Analyzer {
    pub fn new(error_listener: ErrorListener) -> Self {
        Self {
            table: SymbolTable::new(),
            scope: Scope::new(),
            counter: 0,
            error_listener,
        }
    }

    pub fn analyze(&mut self, ast: &mut Ast) -> (SymbolTable, ErrorListener) {
        let table = {
            if ast.analyze(self).is_ok() {
                Ok(std::mem::take(&mut self.table))
            } else {
                Err(ANALYZING_FAILED_ERROR_STR)
            }
        };

        if let Ok(table) = table {
            return (table, self.error_listener.clone());
        }

        (SymbolTable::new(), self.error_listener.clone())
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

    fn is_built_in_identifier(id: &Identifier) -> bool {
        if let Identifier::Common(id) = id {
            return id.starts_with("__tanit_compiler__");
        }

        false
    }

    pub fn check_identifier_existance(&self, id: &Identifier) -> Result<Symbol, &'static str> {
        if let Some(ss) = self.table.get(id) {
            for s in ss.iter() {
                if self.scope.0.starts_with(&s.scope.0) {
                    return Ok(s.clone());
                }
            }
        }

        Err(IDENTIFIER_NOT_FOUND_ERROR_STR)
    }

    pub fn check_call_args(&mut self, node: &values::Value) -> Result<Symbol, &'static str> {
        let (identifier, arguments) = if let values::Value::Call {
            identifier,
            arguments,
        } = node
        {
            (identifier, arguments)
        } else {
            return Err(UNEXPECTED_NODE_PARSED_ERROR_STR);
        };

        if Self::is_built_in_identifier(identifier) {
            return Ok(Symbol {
                scope: Scope(vec!["@s.0".to_string()]),
                data: SymbolData::FunctionDef {
                    args: vec![],
                    return_type: types::Type::new(),
                    is_declaration: false,
                },
            });
        }

        if let Ok(ss) = self.check_identifier_existance(identifier) {
            match &ss.data {
                SymbolData::FunctionDef { args, .. } => {
                    /* Check parameters */
                    if args.len() == arguments.len() {
                        for i in args.iter() {
                            for j in arguments.iter() {
                                let j_type = j.get_type(self);
                                if j_type != *i {
                                    return Err(MISMATCHED_TYPES_ERROR_STR);
                                }
                            }
                        }
                        Ok(ss.clone())
                    } else {
                        Err(WRONG_CALL_ARGUMENTS_ERROR_STR)
                    }
                }
                _ => Err(FUNCTION_NOT_FOUND_ERROR_STR),
            }
        } else {
            Err(IDENTIFIER_NOT_FOUND_ERROR_STR)
        }
    }

    pub fn error(&mut self, message: &str) {
        self.error_listener.semantic_error(message, &self.scope);
    }

    pub fn error_listener(&mut self) -> ErrorListener {
        std::mem::take(&mut self.error_listener)
    }
}
