use tanitc_ast::ast::values::{CallArg, CallArgKind, Value, ValueKind};

use crate::c_generator::CodeGenStream;

impl CodeGenStream<'_> {
    pub fn generate_value(&mut self, val: &Value) -> Result<(), std::io::Error> {
        use std::io::Write;

        match &val.kind {
            ValueKind::Integer(val) => write!(self, "{val}")?,
            ValueKind::Decimal(val) => write!(self, "{val:?}")?,
            ValueKind::Identifier(val) => write!(self, "{val}")?,
            ValueKind::Text(val) => write!(self, "\"{val}\"")?,
            ValueKind::Call {
                identifier,
                arguments,
            } => {
                /* at this point, all arguments must be converted to positional */
                write!(self, "{identifier}(")?;

                if !arguments.is_empty() {
                    self.generate_call_param(&arguments[0])?;
                }

                for arg in arguments.iter().skip(1) {
                    write!(self, ", ")?;
                    self.generate_call_param(arg)?;
                }

                write!(self, ")")?;
            }
            ValueKind::Struct { name, components } => {
                // create anonimous variable
                write!(self, "({name})")?;

                if components.is_empty() {
                    write!(self, " {{ }}")?;
                } else {
                    let indentation = self.indentation();
                    self.indent += 1;

                    writeln!(self, "\n{indentation}{{")?;
                    for (i, (field_name, field_val)) in components.iter().enumerate() {
                        write!(self, "{indentation}    .{field_name}=")?;
                        self.generate(field_val)?;

                        if i < components.len() {
                            writeln!(self, ",")?;
                        }
                    }

                    self.indent -= 1;
                    write!(self, "{indentation}}}")?;
                }
            }
            ValueKind::Array { components } => {
                write!(self, "{{ ")?;

                for (c_idx, c) in components.iter().enumerate() {
                    self.generate(c)?;

                    if c_idx != components.len() - 1 {
                        write!(self, ", ")?;
                    }
                }

                write!(self, " }}")?;
            }
            ValueKind::Tuple { components } => {
                write!(self, "{{ ")?;

                for (c_idx, c) in components.iter().enumerate() {
                    self.generate(c)?;

                    if c_idx != components.len() - 1 {
                        write!(self, ", ")?;
                    }
                }

                write!(self, " }}")?;
            }
        }

        Ok(())
    }

    fn generate_call_param(&mut self, arg: &CallArg) -> Result<(), std::io::Error> {
        match &arg.kind {
            CallArgKind::Positional(_, node) => self.generate(node.as_ref()),
            CallArgKind::Notified(name, _) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Notified argument \"{name}\" must be eliminated at this point"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        blocks::Block,
        functions::FunctionDef,
        values::{CallArg, CallArgKind, Value, ValueKind},
        Ast,
    };
    use tanitc_ident::{Ident, Name};

    use pretty_assertions::assert_str_eq;
    use tanitc_lexer::location::Location;

    use crate::c_generator::CodeGenStream;

    fn get_func(name: &str, statements: Vec<Ast>) -> FunctionDef {
        FunctionDef {
            name: Name::from(name.to_string()),
            body: Some(Box::new(Block {
                is_global: false,
                statements,
                ..Default::default()
            })),
            ..Default::default()
        }
    }

    #[test]
    fn codegen_values_test() {
        const FUNC_NAME: &str = "just_func";

        const HEADER_EXPECTED: &str = "void just_func();\n";
        const SOURCE_EXPECTED: &str = "void just_func()\
                                     \n{\
                                     \n    \"text\"\
                                     \n    var_name\
                                     \n    empty_func_name();\
                                     \n    func_with_1p(0.0);\
                                     \n    func_with_2p(0.0, 2.0);\
                                     \n    (MyEmptyStruct) { }\
                                     \n    (StructWith1F)\
                                     \n    {\
                                     \n        .f1=1.1,\
                                     \n    }\
                                     \n    (StructWith2F)\
                                     \n    {\
                                     \n        .f1=0,\
                                     \n        .f2=2.2,\
                                     \n    }\
                                     \n    {  }\
                                     \n    { 0 }\
                                     \n    { 1, 2 }\
                                     \n    {  }\
                                     \n    { 0 }\
                                     \n    { 1, 2 }\
                                     \n}\n";

        let node = Ast::from(Block {
            is_global: true,
            statements: vec![get_func(
                FUNC_NAME,
                vec![
                    Value {
                        kind: ValueKind::Text("text".to_string()),
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Identifier(Ident::from("var_name".to_string())),
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Call {
                            identifier: Ident::from("empty_func_name".to_string()),
                            arguments: vec![],
                        },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Call {
                            identifier: Ident::from("func_with_1p".to_string()),
                            arguments: vec![CallArg {
                                location: Location::default(),
                                identifier: None,
                                kind: CallArgKind::Positional(
                                    0,
                                    Box::new(
                                        Value {
                                            kind: ValueKind::Decimal(0.0),
                                            location: Location::default(),
                                        }
                                        .into(),
                                    ),
                                ),
                            }],
                        },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Call {
                            identifier: Ident::from("func_with_2p".to_string()),
                            arguments: vec![
                                CallArg {
                                    location: Location::default(),
                                    identifier: None,
                                    kind: CallArgKind::Positional(
                                        0,
                                        Box::new(
                                            Value {
                                                kind: ValueKind::Decimal(0.0),
                                                location: Location::default(),
                                            }
                                            .into(),
                                        ),
                                    ),
                                },
                                CallArg {
                                    location: Location::default(),
                                    identifier: None,
                                    kind: CallArgKind::Positional(
                                        1,
                                        Box::new(
                                            Value {
                                                kind: ValueKind::Decimal(2.0),
                                                location: Location::default(),
                                            }
                                            .into(),
                                        ),
                                    ),
                                },
                            ],
                        },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Struct {
                            name: Name::from("MyEmptyStruct".to_string()),
                            components: vec![],
                        },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Struct {
                            name: Name::from("StructWith1F".to_string()),
                            components: vec![(
                                Name::from("f1".to_string()),
                                Value {
                                    kind: ValueKind::Decimal(1.1),
                                    location: Location::default(),
                                }
                                .into(),
                            )],
                        },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Struct {
                            name: Name::from("StructWith2F".to_string()),
                            components: vec![
                                (
                                    Name::from("f1".to_string()),
                                    Value {
                                        kind: ValueKind::Integer(0),
                                        location: Location::default(),
                                    }
                                    .into(),
                                ),
                                (
                                    Name::from("f2".to_string()),
                                    Value {
                                        kind: ValueKind::Decimal(2.2),
                                        location: Location::default(),
                                    }
                                    .into(),
                                ),
                            ],
                        },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Array { components: vec![] },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Array {
                            components: vec![Value {
                                kind: ValueKind::Integer(0),
                                location: Location::default(),
                            }
                            .into()],
                        },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Array {
                            components: vec![
                                Value {
                                    kind: ValueKind::Integer(1),
                                    location: Location::default(),
                                }
                                .into(),
                                Value {
                                    kind: ValueKind::Integer(2),
                                    location: Location::default(),
                                }
                                .into(),
                            ],
                        },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Tuple { components: vec![] },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Tuple {
                            components: vec![Value {
                                kind: ValueKind::Integer(0),
                                location: Location::default(),
                            }
                            .into()],
                        },
                        location: Location::default(),
                    }
                    .into(),
                    Value {
                        kind: ValueKind::Tuple {
                            components: vec![
                                Value {
                                    kind: ValueKind::Integer(1),
                                    location: Location::default(),
                                }
                                .into(),
                                Value {
                                    kind: ValueKind::Integer(2),
                                    location: Location::default(),
                                }
                                .into(),
                            ],
                        },
                        location: Location::default(),
                    }
                    .into(),
                ],
            )
            .into()],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert_str_eq!(header_res, HEADER_EXPECTED);

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
