use std::collections::HashMap;

use crate::lexer::Location;
use crate::{
    ast::{types, values, Ast, GetType},
    error_listener::ErrorListener,
};

use std::io::Write;

pub type Scope = Vec<String>;

#[derive(Clone)]
pub enum SymbolData {
    Module,
    Struct,
    Function {
        args: Vec<types::Type>,
        return_type: types::Type,
    },
    Tuple,
    Enum,
    Type,
    Variable,
}

impl SymbolData {
    pub fn traverse(&self, stream: &mut std::fs::File) -> std::io::Result<()> {
        match self {
            Self::Function { args, return_type } => {
                writeln!(stream, "function ({:?}) -> {:?}", args, return_type)
            }
            Self::Enum => writeln!(stream, "enum"),
            Self::Module => writeln!(stream, "module"),
            Self::Struct => writeln!(stream, "struct"),
            Self::Tuple => writeln!(stream, "tuple"),
            Self::Type => writeln!(stream, "type"),
            Self::Variable => writeln!(stream, "variable"),
        }
    }
}

#[derive(Clone)]
pub enum Symbol {
    Definition { stype: SymbolData, scope: Scope },
    Declaration { stype: SymbolData, scope: Scope },
}

pub struct SymbolTable {
    table: HashMap<String, Vec<Symbol>>,
    error_listener: ErrorListener,
}

impl SymbolTable {
    pub fn new(error_listener: ErrorListener) -> Self {
        Self {
            table: HashMap::new(),
            error_listener,
        }
    }

    pub fn insert(&mut self, id: &str, symbol: Symbol) {
        if !self.table.contains_key(id) {
            self.table.insert(id.to_string(), Vec::new());
        }

        if let Some(ss) = self.table.get_mut(id) {
            ss.push(symbol)
        }
    }

    pub fn check_identifier_existance(&self, id: &str, scope: &Scope) -> bool {
        if let Some(ss) = self.table.get(id) {
            for s in ss.iter() {
                let s_scope = match s {
                    Symbol::Definition { scope, .. } => scope,

                    Symbol::Declaration { scope, .. } => scope,
                };

                if scope.starts_with(s_scope) {
                    return true;
                }
            }
            false
        } else {
            false
        }
    }

    pub fn check_call_args(&mut self, node: &values::Value, in_scope: &Scope) -> bool {
        let (identifier, arguments) = if let values::Value::Call {
            identifier,
            arguments,
        } = node
        {
            (identifier, arguments)
        } else {
            return false;
        };

        if self.check_identifier_existance(identifier, in_scope) {
            let ss = self.table.get(identifier).unwrap();

            for s in ss.iter() {
                if let Symbol::Definition {
                    stype: SymbolData::Function { args, .. },
                    scope,
                } = s
                {
                    if in_scope.starts_with(scope) {
                        if args.len() != arguments.len() {
                            self.error(&format!(
                                "Expected to get \"{}\" parameters, but was \"{}\" supplied",
                                args.len(),
                                arguments.len()
                            ));
                            return false;
                        }

                        for i in args.iter() {
                            for j in arguments.iter() {
                                let j_type = j.get_type().unwrap();
                                if j_type != *i {
                                    self.error(&format!(
                                        "Mismatched types: expected \"{:?}\", but \"{:?}\" was provided",
                                        i, j_type));
                                    return false;
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    pub fn analyze(&mut self, ast: &mut Ast, scope: Scope) {
        match ast {
            Ast::Scope { node } => {
                let mut new_scope = scope.clone();
                new_scope.push("@s".to_string());
                for n in node.statements.iter_mut() {
                    self.analyze(n, new_scope.clone());
                }
            }

            Ast::FuncDef { node } => {
                if self.check_identifier_existance(&node.identifier, &scope) {
                    self.error(&format!(
                        "Identifier \"{}\" defined multiple times",
                        &node.identifier
                    ));
                    return;
                }

                let data = SymbolData::Function {
                    args: {
                        let mut arguments = Vec::<types::Type>::new();
                        for p in node.parameters.iter() {
                            arguments.push(p.var_type.clone())
                        }
                        arguments
                    },
                    return_type: node.return_type.clone(),
                };

                self.insert(
                    &node.identifier,
                    Symbol::Definition {
                        stype: data.clone(),
                        scope: scope.clone(),
                    },
                );

                if node.body.is_some() {
                    self.insert(
                        &node.identifier,
                        Symbol::Declaration {
                            stype: data,
                            scope: scope.clone(),
                        },
                    );
                }
            }

            Ast::AliasDef { node } => {
                if self.check_identifier_existance(&node.identifier, &scope) {
                    self.error(&format!(
                        "Identifier \"{}\" defined multiple times",
                        &node.identifier
                    ));
                    return;
                }

                self.insert(
                    &node.identifier,
                    Symbol::Definition {
                        stype: SymbolData::Type,
                        scope: scope.clone(),
                    },
                );
            }

            Ast::ModuleDef { node } => {
                if self.check_identifier_existance(&node.identifier, &scope) {
                    self.error(&format!(
                        "Identifier \"{}\" defined multiple times",
                        &node.identifier
                    ));
                    return;
                }

                {
                    let mut new_scope = scope.clone();
                    new_scope.push(node.identifier.clone());

                    self.analyze(node.body.as_mut(), new_scope);
                }

                self.insert(
                    &node.identifier,
                    Symbol::Definition {
                        stype: SymbolData::Module,
                        scope: scope.clone(),
                    },
                );
            }

            Ast::StructDef { node } => {
                if self.check_identifier_existance(&node.identifier, &scope) {
                    self.error(&format!(
                        "Identifier \"{}\" defined multiple times",
                        &node.identifier
                    ));
                    return;
                }

                let mut new_scope = scope.clone();
                new_scope.push(node.identifier.clone());
                for internal in node.internals.iter_mut() {
                    self.analyze(internal, new_scope.clone())
                }

                self.insert(
                    &node.identifier,
                    Symbol::Definition {
                        stype: SymbolData::Struct,
                        scope: scope.clone(),
                    },
                );
            }

            Ast::EnumDef { node } => {
                if self.check_identifier_existance(&node.identifier, &scope) {
                    self.error(&format!(
                        "Identifier \"{}\" defined multiple times",
                        &node.identifier
                    ));
                    return;
                }

                let mut new_scope = scope.clone();
                new_scope.push(node.identifier.clone());
                for internal in node.internals.iter_mut() {
                    self.analyze(internal, new_scope.clone())
                }

                self.insert(
                    &node.identifier,
                    Symbol::Definition {
                        stype: SymbolData::Enum,
                        scope: scope.clone(),
                    },
                );
            }

            Ast::VariableDef { node } => {
                if self.check_identifier_existance(&node.identifier, &scope) {
                    self.error(&format!(
                        "Identifier \"{}\" defined multiple times",
                        &node.identifier
                    ));
                    return;
                }

                self.insert(
                    &node.identifier,
                    Symbol::Definition {
                        stype: SymbolData::Variable,
                        scope: scope.clone(),
                    },
                );
            }

            Ast::Value { node } => {
                if let values::Value::Identifier(id) = node {
                    if !self.check_identifier_existance(id, &scope) {
                        self.error(&format!("Cannot find \"{}\" in this scope", id));
                    }
                }

                if let values::Value::Call { .. } = node {
                    if !self.check_call_args(node, &scope) {
                        self.error("Wrong call arguments")
                    }
                }
            }

            _ => {
                unimplemented!()
            }
        }
    }

    pub fn error(&mut self, message: &str) {
        self.error_listener.semantic_error(message, Location::new());
    }

    pub fn traverse(&self, stream: &mut std::fs::File) -> std::io::Result<()> {
        for (identifier, ss) in self.table.iter() {
            writeln!(stream, "Identifier: \"{}\":", identifier)?;

            for s in ss.iter() {
                write!(stream, "+--- ")?;

                match s {
                    Symbol::Definition { stype, scope } => {
                        write!(stream, "Definition ({:?}) ", scope)?;
                        stype.traverse(stream)?;
                    }

                    Symbol::Declaration { stype, scope } => {
                        write!(stream, "Declaration ({:?}) ", scope)?;
                        stype.traverse(stream)?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn dump_symtable(output: String, symbol_table: &SymbolTable) -> std::io::Result<()> {
    let mut stream = std::fs::File::create(format!("{}_symbol_table.txt", output)).unwrap();
    symbol_table.traverse(&mut stream)
}
