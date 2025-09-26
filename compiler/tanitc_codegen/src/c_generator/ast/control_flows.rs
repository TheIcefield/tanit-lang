use tanitc_ast::ast::control_flows::{ControlFlow, ControlFlowKind};

use crate::c_generator::{CodeGenMode, CodeGenStream};

impl CodeGenStream<'_> {
    pub fn generate_control_flow(&mut self, cf: &ControlFlow) -> Result<(), std::io::Error> {
        use std::io::Write;

        let old_mode = self.mode;
        self.mode = CodeGenMode::SourceOnly;

        write!(self, "{}", cf.kind.to_str())?;

        if let ControlFlowKind::Return { ret: Some(expr) } = &cf.kind {
            write!(self, " ")?;
            self.generate(expr)?;
        }

        self.mode = old_mode;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tanitc_ast::ast::{
        blocks::Block,
        control_flows::{ControlFlow, ControlFlowKind},
        values::{Value, ValueKind},
        Ast,
    };
    use tanitc_lexer::location::Location;

    use pretty_assertions::assert_str_eq;

    use crate::c_generator::CodeGenStream;

    fn get_return(ret: Option<Box<Ast>>) -> ControlFlow {
        ControlFlow {
            location: Location::default(),
            kind: ControlFlowKind::Return { ret },
        }
    }

    fn get_continue() -> ControlFlow {
        ControlFlow {
            location: Location::default(),
            kind: ControlFlowKind::Continue,
        }
    }

    fn get_break() -> ControlFlow {
        ControlFlow {
            location: Location::default(),
            kind: ControlFlowKind::Break { ret: None },
        }
    }

    #[test]
    fn codegen_control_flow_test() {
        const SOURCE_EXPECTED: &str = "return\
                                       return 0\
                                       break\
                                       continue";

        let node = Ast::from(Block {
            is_global: true,
            statements: vec![Block {
                is_global: false,
                statements: vec![
                    get_return(None).into(),
                    get_return(Some(Box::new(Ast::from(Value {
                        kind: ValueKind::Integer(0),
                        location: Location::default(),
                    }))))
                    .into(),
                    get_break().into(),
                    get_continue().into(),
                ],
                ..Default::default()
            }
            .into()],
            ..Default::default()
        });

        let mut header_buffer = Vec::<u8>::new();
        let mut source_buffer = Vec::<u8>::new();
        let mut writer = CodeGenStream::new(&mut header_buffer, &mut source_buffer).unwrap();

        node.accept(&mut writer).unwrap();

        let header_res = String::from_utf8(header_buffer).unwrap();
        assert!(header_res.is_empty());

        let source_res = String::from_utf8(source_buffer).unwrap();
        assert_str_eq!(source_res, SOURCE_EXPECTED);
    }
}
