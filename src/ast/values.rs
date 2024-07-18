use crate::analyzer::{Analyzer, SymbolData};
use crate::ast::{expressions::Expression, identifiers::Identifier, types::Type, Ast, IAst};
use crate::codegen::CodeGenStream;
use crate::lexer::Lexem;
use crate::parser::Parser;

use std::io::Write;

#[derive(Clone, PartialEq)]
pub enum CallParam {
    Notified(Identifier, Box<Ast>),
    Positional(usize, Box<Ast>),
}

impl IAst for CallParam {
    fn get_type(&self, analyzer: &mut crate::analyzer::Analyzer) -> Type {
        match self {
            Self::Notified(_, expr) | Self::Positional(_, expr) => expr.get_type(analyzer),
        }
    }

    fn analyze(&mut self, _analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        Ok(())
    }

    fn serialize(&self, _writer: &mut crate::serializer::XmlWriter) -> std::io::Result<()> {
        todo!("serialize CallParam")
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        match self {
            Self::Positional(_, node) => node.codegen(stream),
            Self::Notified(..) => unreachable!("Notified CallParam is not allowed in codegen"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Call {
        identifier: Identifier,
        arguments: Vec<CallParam>,
    },
    Struct {
        identifier: Identifier,
        components: Vec<(Identifier, Ast)>,
    },
    Tuple {
        components: Vec<Ast>,
    },
    Array {
        components: Vec<Ast>,
    },
    Identifier(Identifier),
    Text(String),
    Integer(usize),
    Decimal(f64),
}

impl Value {
    pub fn parse_call_params(parser: &mut Parser) -> Result<Vec<CallParam>, &'static str> {
        parser.consume_token(Lexem::LParen)?;

        let mut args = Vec::<CallParam>::new();

        let mut i = 0;
        loop {
            let next = parser.peek_token();

            if next.lexem == Lexem::RParen {
                break;
            }

            let expr = Expression::parse(parser)?;

            let param_id = if let Ast::Value {
                node: Self::Identifier(id),
            } = &expr
            {
                if parser.peek_token().lexem == Lexem::Colon {
                    parser.consume_token(Lexem::Colon)?;
                    Some(id.clone())
                } else {
                    None
                }
            } else {
                None
            };

            let param = if let Some(id) = param_id {
                CallParam::Notified(id, Box::new(Expression::parse(parser)?))
            } else {
                CallParam::Positional(i, Box::new(expr))
            };

            args.push(param);

            i += 1;

            let next = parser.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                parser.get_token();
                continue;
            } else if next.lexem == Lexem::RParen {
                // end parsing if ')'
                break;
            } else {
                parser.error("Unexpected token when parsing call", next.get_location());
                return Err("Unexpected token");
            }
        }

        parser.consume_token(Lexem::RParen)?;

        Ok(args)
    }

    pub fn parse_array(parser: &mut Parser) -> Result<Ast, &'static str> {
        parser.consume_token(Lexem::Lsb)?;

        let mut components = Vec::<Ast>::new();

        loop {
            let next = parser.peek_token();

            if next.lexem == Lexem::Rsb {
                break;
            }
            components.push(Expression::parse(parser)?);

            let next = parser.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                parser.get_token();
                continue;
            } else if next.lexem == Lexem::Rsb {
                // end parsing if ']'
                break;
            } else {
                parser.error("Unexpected token when parsing call", next.get_location());
                return Err("Unexpected token");
            }
        }

        parser.consume_token(Lexem::Rsb)?;

        Ok(Ast::Value {
            node: Value::Array { components },
        })
    }

    pub fn parse_struct(parser: &mut Parser) -> Result<Vec<(Identifier, Ast)>, &'static str> {
        parser.consume_token(Lexem::Lcb)?;

        let mut components = Vec::<(Identifier, Ast)>::new();

        loop {
            let next = parser.peek_token();

            if next.lexem == Lexem::Rcb {
                break;
            }

            let identifier = Identifier::from_token(&parser.consume_identifier()?)?;

            parser.consume_token(Lexem::Colon)?;

            components.push((identifier, Expression::parse(parser)?));

            let next = parser.peek_token();
            if next.lexem == Lexem::Comma {
                // continue parsing if ','
                parser.get_token();
                continue;
            } else if next.lexem == Lexem::Rcb {
                // end parsing if '}'
                break;
            } else {
                parser.error(
                    "Unexpected token when parsing struct value",
                    next.get_location(),
                );
                return Err("Unexpected token");
            }
        }

        parser.consume_token(Lexem::Rcb)?;

        Ok(components)
    }
}

// Value::Call
impl Value {
    fn check_call_args(
        &mut self,
        analyzer: &mut crate::analyzer::Analyzer,
    ) -> Result<(), &'static str> {
        let (identifier, arguments) = if let Self::Call {
            identifier,
            arguments,
        } = self
        {
            (identifier, arguments)
        } else {
            return Err("Expected call node, but provided another");
        };

        if Analyzer::is_built_in_identifier(identifier) {
            return Ok(());
        }

        if let Ok(ss) = analyzer.check_identifier_existance(identifier) {
            match &ss.data {
                SymbolData::FunctionDef { args, .. } => {
                    if arguments.len() > args.len() {
                        analyzer.error(&format!(
                            "Too many arguments passed in function \"{}\", expected: {}, actually: {}",
                            identifier, args.len(), arguments.len()));
                        return Err("Too many arguments passed");
                    }

                    if arguments.len() < args.len() {
                        analyzer.error(&format!(
                            "Too few arguments passed in function \"{}\", expected: {}, actually: {}",
                            identifier, args.len(), arguments.len()));
                        return Err("Too few arguments passed");
                    }

                    let mut positional_skiped = false;
                    for call_arg in arguments.iter_mut() {
                        let arg_clone = call_arg.clone();
                        match arg_clone {
                            CallParam::Notified(param_id, param_value) => {
                                positional_skiped = true;

                                // check if such parameter declared in the function
                                let mut param_found = false;
                                let param_id = param_id.get_string();
                                for (param_index, (func_param_name, func_param_type)) in
                                    args.iter().enumerate()
                                {
                                    if *func_param_name == param_id {
                                        param_found = true;

                                        let param_type = param_value.get_type(analyzer);
                                        if *func_param_type != param_type {
                                            analyzer.error(&format!(
                                                "Mismatched type for parameter \"{}\". Expected \"{}\", actually: \"{}\"",
                                                func_param_name, func_param_type, param_type)
                                            );
                                        }

                                        let modified_param =
                                            CallParam::Positional(param_index, param_value.clone());
                                        *call_arg = modified_param;
                                    }
                                }
                                if !param_found {
                                    analyzer.error(&format!(
                                        "No parameter named \"{}\" in function \"{}\"",
                                        param_id, identifier
                                    ))
                                }
                            }
                            CallParam::Positional(..) => {
                                if positional_skiped {
                                    analyzer.error(
                                        "Positional parameters must be passed before notified",
                                    );
                                    return Err("Positional param passed after notified");
                                }
                            }
                        }
                    }

                    /* Check parameters */
                    for i in args.iter() {
                        for j in arguments.iter() {
                            let j_type = j.get_type(analyzer);
                            if j_type != i.1 {
                                return Err("Mismatched types");
                            }
                        }
                    }
                    Ok(())
                }
                _ => Err("No such function found"),
            }
        } else {
            Err("No such identifier found")
        }
    }
}

impl IAst for Value {
    fn analyze(&mut self, analyzer: &mut crate::analyzer::Analyzer) -> Result<(), &'static str> {
        match self {
            Self::Integer(_) => Ok(()),

            Self::Decimal(_) => Ok(()),

            Self::Text(_) => Ok(()),

            Self::Identifier(id) => {
                if analyzer.check_identifier_existance(id).is_err() {
                    analyzer.error(&format!("Cannot find \"{}\" in this scope", id));
                    return Err("No such identifier gound");
                }

                Ok(())
            }

            Self::Call { arguments, .. } => {
                for arg in arguments.iter_mut() {
                    arg.analyze(analyzer)?;
                }

                if self.check_call_args(analyzer).is_err() {
                    analyzer.error("Wrong call arguments");
                    return Err("Wrong call arguments");
                }

                Ok(())
            }

            Self::Struct {
                identifier,
                components: value_comps,
            } => {
                let ss = analyzer.check_identifier_existance(identifier);
                if ss.is_err() {
                    analyzer.error(&format!("Cannot find \"{}\" in this scope", identifier));
                    return Err("No such identifier found");
                }

                let ss = ss.unwrap();

                if let SymbolData::StructDef {
                    components: struct_comps,
                } = &ss.data
                {
                    if value_comps.len() != struct_comps.len() {
                        analyzer.error(&format!(
                            "Struct \"{}\" consists of {} fields, but {} were supplied",
                            identifier,
                            struct_comps.len(),
                            value_comps.len()
                        ));
                        return Err("Struct definition and declarations is different");
                    }

                    for comp_id in 0..value_comps.len() {
                        let value_comp = value_comps.get(comp_id).unwrap();
                        let value_comp_type = value_comp.1.get_type(analyzer);
                        let struct_comp_type = struct_comps.get(comp_id).unwrap();

                        if value_comp_type != *struct_comp_type {
                            analyzer.error(&format!(
                                "Field named \"{}\" is {}, but initialized like {}",
                                value_comp.0, struct_comp_type, value_comp_type
                            ));
                            return Err("Mismatched types during struct initialization");
                        }
                    }
                } else {
                    analyzer.error(&format!(
                        "Cannot find struct named \"{}\" in this scope",
                        identifier
                    ));
                    return Err("No such identifier found");
                }

                Ok(())
            }

            Self::Tuple { components } => {
                for comp in components.iter_mut() {
                    comp.analyze(analyzer)?;
                }

                Ok(())
            }

            Self::Array { components } => {
                if components.is_empty() {
                    return Ok(());
                }

                let comp_type = components[0].get_type(analyzer);

                for comp in components.iter().enumerate() {
                    let current_comp_type = comp.1.get_type(analyzer);
                    if comp_type != current_comp_type {
                        analyzer.error(&format!(
                            "Array type is declared like {}, but {}{} element has type {}",
                            comp_type,
                            comp.0 + 1,
                            match comp.0 % 10 {
                                0 => "st",
                                1 => "nd",
                                2 => "rd",
                                _ => "th",
                            },
                            current_comp_type
                        ));
                        return Err("Array components have different types");
                    }
                }

                Ok(())
            }
        }
    }

    fn get_type(&self, analyzer: &mut crate::analyzer::Analyzer) -> Type {
        match self {
            Self::Text(_) => Type::Ref {
                is_mut: false,
                ref_to: Box::new(Type::Str),
            },
            Self::Decimal(_) => Type::F32,
            Self::Integer(_) => Type::I32,
            Self::Identifier(id) => {
                if let Some(ss) = analyzer.get_symbols(id) {
                    for s in ss.iter().rev() {
                        if analyzer.scope.0.starts_with(&s.scope.0) {
                            if let SymbolData::VariableDef { var_type, .. } = &s.data {
                                return var_type.clone();
                            }
                        }
                    }
                }
                analyzer.error(&format!("No variable found with name \"{}\"", id));
                Type::new()
            }
            Self::Struct { identifier, .. } => Type::Custom(identifier.to_string()),
            Self::Tuple { components } => {
                let mut comp_vec = Vec::<Type>::new();
                for comp in components.iter() {
                    comp_vec.push(comp.get_type(analyzer));
                }
                Type::Tuple {
                    components: comp_vec,
                }
            }
            Self::Array { components } => {
                let len = components.len();
                if len == 0 {
                    return Type::Array {
                        size: None,
                        value_type: Box::new(Type::Auto),
                    };
                }

                Type::Array {
                    size: Some(Box::new(Ast::Value {
                        node: Value::Integer(len),
                    })),
                    value_type: Box::new(components[0].get_type(analyzer)),
                }
            }
            Self::Call { identifier, .. } => {
                if let Ok(ss) = analyzer.check_identifier_existance(identifier) {
                    if let SymbolData::FunctionDef { return_type, .. } = &ss.data {
                        return return_type.clone();
                    }
                }
                Type::new()
            }
        }
    }

    fn serialize(&self, writer: &mut crate::serializer::XmlWriter) -> std::io::Result<()> {
        match self {
            Self::Call {
                identifier,
                arguments,
            } => {
                writer.begin_tag("call-statement")?;

                identifier.serialize(writer)?;

                writer.begin_tag("parameters")?;
                for arg in arguments.iter() {
                    writer.begin_tag("parameter")?;
                    match arg {
                        CallParam::Notified(id, expr) => {
                            id.serialize(writer)?;
                            expr.serialize(writer)?;
                        }
                        CallParam::Positional(index, expr) => {
                            writer.put_param("index", index)?;
                            expr.serialize(writer)?;
                        }
                    }
                    writer.end_tag()?; //parameter
                }
                writer.end_tag()?; // parameters
                writer.end_tag()?; // call-statement
            }
            Self::Struct {
                identifier,
                components,
            } => {
                writer.begin_tag("struct-initialization")?;

                identifier.serialize(writer)?;

                for (comp_id, comp_type) in components.iter() {
                    writer.begin_tag("field")?;

                    comp_id.serialize(writer)?;
                    comp_type.serialize(writer)?;

                    writer.end_tag()?;
                }

                writer.end_tag()?;
            }
            Self::Tuple { components } => {
                writer.begin_tag("tuple-initialization")?;

                for component in components.iter() {
                    component.serialize(writer)?;
                }

                writer.end_tag()?;
            }
            Self::Array { components } => {
                writer.begin_tag("array-initialization")?;

                for component in components.iter() {
                    component.serialize(writer)?;
                }

                writer.end_tag()?;
            }
            Self::Identifier(id) => {
                writer.begin_tag("variable")?;
                id.serialize(writer)?;
                writer.end_tag()?;
            }
            Self::Text(value) => {
                writer.begin_tag("literal")?;
                writer.put_param("style", "text")?;
                writer.put_param("value", value)?;
                writer.end_tag()?;
            }
            Self::Integer(value) => {
                writer.begin_tag("literal")?;
                writer.put_param("style", "integer-number")?;
                writer.put_param("value", value)?;
                writer.end_tag()?;
            }
            Self::Decimal(value) => {
                writer.begin_tag("literal")?;
                writer.put_param("style", "decimal-number")?;
                writer.put_param("value", value)?;
                writer.end_tag()?;
            }
        }

        Ok(())
    }

    fn codegen(&self, stream: &mut CodeGenStream) -> std::io::Result<()> {
        match self {
            Self::Integer(val) => write!(stream, "{}", *val)?,
            Self::Decimal(val) => write!(stream, "{}", *val)?,
            Self::Identifier(val) => val.codegen(stream)?,
            Self::Call {
                identifier,
                arguments,
            } => {
                /* at this point, all arguments must be converted to positional */

                identifier.codegen(stream)?;
                write!(stream, "(")?;

                if !arguments.is_empty() {
                    arguments[0].codegen(stream)?;
                }

                for arg in arguments.iter().skip(1) {
                    write!(stream, ", ")?;
                    arg.codegen(stream)?;
                }

                write!(stream, ")")?;
            }
            _ => unimplemented!(),
        }

        Ok(())
    }
}
