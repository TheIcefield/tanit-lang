use std::collections::HashMap;

use crate::ast::structs::EnumField;
use crate::ast::{expressions, structs};
use crate::lexer::{Location, TokenType};
use crate::{
    ast::{types, values, Ast, GetType},
    error_listener::ErrorListener,
};

use std::io::Write;

pub type Scope = Vec<String>;

#[derive(Clone)]
pub enum SymbolData {
    ModuleDef {
        full_name: Vec<String>
    },
    StructDef {
        components: Vec<types::Type>
    },
    EnumDef {
        components: Vec<structs::EnumField>
    },
    FunctionDef {
        args: Vec<types::Type>,
        return_type: types::Type,
        is_declaration: bool,
    },
    VariableDef {
        var_type: types::Type,
        is_initialization: bool,
    },
    Type,
}

impl SymbolData {
    pub fn traverse(&self, stream: &mut std::fs::File) -> std::io::Result<()> {
        match self {
            Self::ModuleDef { full_name } =>
                writeln!(stream, "{:?}", full_name),

            Self::FunctionDef { args, return_type, is_declaration } =>
                writeln!(stream, "Function {}: ({:?}) -> {:?}",
                    if *is_declaration { "declaration" }
                    else { "definition" },
                    args,
                    return_type),

            Self::StructDef { components } =>
                writeln!(stream, "Struct definition: {{{:?}}}", components),
            
            Self::EnumDef { components } => {
                write!(stream, "Enum definition: <")?;

                for comp in components.iter() {
                    match comp {
                        EnumField::Common => write!(stream, "common")?,
                        EnumField::TupleLike(t) => {
                            write!(stream, "( ")?;
                            for tc in t.iter() {
                                write!(stream, "{:?}", *tc)?;
                            }
                            write!(stream, ")")?;
                        }
                        EnumField::StructLike(s) => {
                            write!(stream, "{{ ")?;
                            for sc in s.iter() {
                                write!(stream, "{:?}", *sc.1)?;
                            }
                            write!(stream, "}}")?;
                        }
                    }
                }

                writeln!(stream, ">")
            }

            Self::VariableDef { var_type, is_initialization } =>
                writeln!(stream, "Variable {}: {:?}",
                    if *is_initialization { "initialization" }
                    else { "definition" },
                    var_type),

            Self::Type => writeln!(stream, "type"),
        }
    }
}

pub struct Symbol {
    pub scope: Scope,
    pub data: SymbolData
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

    pub fn check_identifier_existance(&self, id: &str, scope: &Scope) -> Option<&Symbol> {
        if let Some(ss) = self.table.get(id) {
            for s in ss.iter() {
                if scope.starts_with(&s.scope) {
                    return Some(s);
                }
            }
            None
        } else {
            None
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

        if let Some(ss) = self.check_identifier_existance(identifier, in_scope) {
            match &ss.data {
                SymbolData::FunctionDef { args, .. } => {
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
                _ => {}
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
                if let Some(_ss) = self.check_identifier_existance(&node.identifier, &scope) {
                    self.error(&format!(
                        "Identifier \"{}\" defined multiple times",
                        &node.identifier
                    ));
                    return;
                }

                let mut arguments = Vec::<types::Type>::new();
                for p in node.parameters.iter() {
                    arguments.push(p.var_type.clone())
                }

                self.insert(
                    &node.identifier,
                    Symbol {
                        scope: scope.clone(),
                        data: SymbolData::FunctionDef {
                            args: arguments.clone(),
                            return_type: node.return_type.clone(),
                            is_declaration: node.body.is_some()
                        }
                    }
                );
            }

            Ast::AliasDef { node } => {
                if let Some(_ss) = self.check_identifier_existance(&node.identifier, &scope) {
                    self.error(&format!(
                        "Identifier \"{}\" defined multiple times",
                        &node.identifier
                    ));
                    return;
                }

                self.insert(
                    &node.identifier,
                    Symbol {
                        data: SymbolData::Type,
                        scope: scope.clone(),
                    },
                );
            }

            Ast::ModuleDef { node } => {
                if self.check_identifier_existance(&node.identifier, &scope).is_some() {
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
                    Symbol {
                        data: SymbolData::ModuleDef {
                            full_name: vec![node.identifier.clone()]
                        },
                        scope: scope.clone(),
                    },
                );
            }

            Ast::StructDef { node } => {
                if self.check_identifier_existance(&node.identifier, &scope).is_some() {
                    self.error(&format!(
                        "Identifier \"{}\" defined multiple times",
                        &node.identifier
                    ));
                    return;
                }

                let mut new_scope = scope.clone();
                new_scope.push(node.identifier.clone());
                for internal in node.internals.iter_mut() {
                    self.analyze(internal, new_scope.clone());
                }

                let mut components = Vec::<types::Type>::new();
                for field in node.fields.iter() {
                    components.push(field.1.clone());
                }

                self.insert(
                    &node.identifier,
                    Symbol {
                        data: SymbolData::StructDef { components },
                        scope: scope.clone()
                    }
                );
            }

            Ast::EnumDef { node } => {
                if let Some(_ss) = self.check_identifier_existance(&node.identifier, &scope) {
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

                let mut components = Vec::<EnumField>::new();
                for field in node.fields.iter() {
                    components.push(field.1.clone());
                }

                self.insert(
                    &node.identifier,
                    Symbol {
                        data: SymbolData::EnumDef { components },
                        scope: scope.clone(),
                    },
                );
            }

            Ast::VariableDef { node } => {
                if self.check_identifier_existance(&node.identifier, &scope).is_some() {
                    self.error(&format!(
                        "Identifier \"{}\" defined multiple times",
                        &node.identifier
                    ));
                    return;
                }

                self.insert(
                    &node.identifier,
                    Symbol {
                        scope: scope.clone(),
                        data: SymbolData::VariableDef {
                            var_type: node.var_type.clone(),
                            is_initialization: false,
                        }
                    }
                );
            }

            Ast::Value { node } => {
                if let values::Value::Identifier(id) = node {
                    if self.check_identifier_existance(id, &scope).is_none() {
                        self.error(&format!("Cannot find \"{}\" in this scope", id));
                    }
                }

                if let values::Value::Call { .. } = node {
                    if !self.check_call_args(node, &scope) {
                        self.error("Wrong call arguments")
                    }
                }
            }

            Ast::Expression { node } => {
                match node.as_mut() {
                    expressions::Expression::Binary { operation, lhs, rhs } => {
                        if let Ast::VariableDef { node } = lhs.as_ref() {
                            if *operation == TokenType::Assign {
                                if self.check_identifier_existance(&node.identifier, &scope).is_some() {
                                    self.error(&format!(
                                        "Identifier \"{}\" defined multiple times",
                                        &node.identifier
                                    ));
                                    return;
                                }
                
                                self.insert(
                                    &node.identifier,
                                    Symbol {
                                        scope: scope.clone(),
                                        data: SymbolData::VariableDef {
                                            var_type: node.var_type.clone(),
                                            is_initialization: true,
                                        }
                                    }
                                );
                            } else {
                                self.analyze(lhs.as_mut(), scope.clone());
                            }

                            self.analyze(rhs.as_mut(), scope);
                        }
                    }
                    expressions::Expression::Unary { node, .. } => {
                        self.analyze(node.as_mut(), scope)
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

                s.data.traverse(stream)?;

                writeln!(stream, " at [{:?}]", s.scope)?;
            }
        }
        Ok(())
    }
}

pub fn dump_symtable(output: String, symbol_table: &SymbolTable) -> std::io::Result<()> {
    let mut stream = std::fs::File::create(format!("{}_symbol_table.txt", output)).unwrap();
    symbol_table.traverse(&mut stream)
}
