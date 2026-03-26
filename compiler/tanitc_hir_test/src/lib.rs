use std::{iter::Peekable, slice::Iter};

use tanitc_attributes::Mutability;
use tanitc_hir::hir::{
    blocks::Block,
    definitions::{
        aliases::AliasDef,
        enums::{EnumDef, EnumUnits},
        functions::{FunctionDef, FunctionParam},
        methods::ImplDef,
        modules::ModuleDef,
        structs::{StructDef, StructFieldsInfo},
        unions::UnionDef,
        variables::VariableDef,
        variants::{VariantAttributes, VariantDef, VariantField, VariantFields},
    },
    expressions::{
        binary::{BinaryExpr, BinaryOperation},
        Expression,
    },
    types::{Type, TypeSpec},
    Hir,
};
use tanitc_ident::{Ident, Name};
use tanitc_lexer::location::Location;

/* Creates: program with global block of recieved statements */
pub fn create_program(statements: Vec<Hir>) -> Hir {
    create_block(statements).into()
}

/* Creates: global block of recieved statements */
pub fn create_block(statements: Vec<Hir>) -> Block {
    Block {
        is_global: true,
        statements,
        ..Default::default()
    }
}

/* Creates
 * module <name> {
 *     <definitions>
 * }
 */
pub fn create_module_def(name: &str, definitions: Vec<Hir>) -> ModuleDef {
    use tanitc_hir::hir::definitions::modules::{ModuleAttributes, ModuleDefBody};

    ModuleDef {
        location: Location::default(),
        attributes: ModuleAttributes::default(),
        name: name.to_string().into(),
        body: ModuleDefBody::Internal(Box::new(create_block(definitions))),
    }
}

pub fn create_enum_def_units(units: Vec<(&str, Option<usize>)>) -> EnumUnits {
    units
        .into_iter()
        .map(|(unit_name, unit_value)| (unit_name.to_string().into(), unit_value))
        .collect::<EnumUnits>()
}

pub fn create_enum_def(name: &str, units: Vec<(&str, Option<usize>)>) -> EnumDef {
    EnumDef {
        name: Name::from(name.to_string()),
        units: create_enum_def_units(units),
        ..Default::default()
    }
}

pub fn create_struct_fields(fields: Vec<(&str, Type)>) -> StructFieldsInfo {
    use tanitc_hir::hir::definitions::structs::StructFieldInfo;

    let mut local_fields = StructFieldsInfo::new();
    for (field_name, field_ty) in fields {
        local_fields.insert(
            Ident::from(field_name.to_string()),
            StructFieldInfo {
                ty: TypeSpec {
                    ty: field_ty,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
    }
    local_fields
}

/* Creates:
 * struct <name> {
 *     <fields[i].0>: <fields[i].1>
 * }
 */
pub fn create_struct_def(name: &str, fields: Vec<(&str, Type)>) -> StructDef {
    StructDef {
        name: Name::from(name.to_string()),
        fields: create_struct_fields(fields),
        ..Default::default()
    }
}

/* Creates:
 * union <name> {
 *     fields[i].0: fields[i].1
 * }
 */
pub fn create_union_def(name: &str, fields: &[(&str, Type)]) -> UnionDef {
    use tanitc_hir::hir::definitions::unions::{UnionFieldInfo, UnionFields};

    UnionDef {
        name: Name::from(name.to_string()),
        fields: {
            let mut local_fields = UnionFields::new();
            for (field_name, field_ty) in fields.iter() {
                local_fields.insert(
                    Ident::from(field_name.to_string()),
                    UnionFieldInfo {
                        ty: TypeSpec {
                            ty: field_ty.clone(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                );
            }
            local_fields
        },
        ..Default::default()
    }
}

/* Creates:
 * variant <name> {
 *
 * }
 */
pub fn create_variant_def(name: &str, variants: Vec<(&str, VariantField)>) -> VariantDef {
    let mut fields = VariantFields::new();

    for (variant_name, variant_data) in variants {
        fields.insert(Ident::from(variant_name.to_string()), variant_data);
    }

    VariantDef {
        name: Name::from(name.to_string()),
        fields,
        internals: vec![],
        location: Location::default(),
        attributes: VariantAttributes::default(),
    }
}

pub fn create_enum_variantfield(name: &str) -> (&str, VariantField) {
    (name, VariantField::Enum)
}

pub fn create_struct_variantfield<'a>(
    name: &'a str,
    fields: Vec<(&str, Type)>,
) -> (&'a str, VariantField) {
    (name, VariantField::Struct(create_struct_fields(fields)))
}

pub fn create_tuple_variantfield(name: &str, fields: Vec<Type>) -> (&str, VariantField) {
    (name, VariantField::Tuple(fields))
}

/* Creates:
 * func <name>() {
 *     statements
 * }
 */
pub fn create_func_def(
    name: &str,
    parameters: Vec<FunctionParam>,
    return_type: Type,
    statements: Vec<Hir>,
) -> FunctionDef {
    FunctionDef {
        name: Name::from(name.to_string()),
        parameters,
        return_type,
        body: Some(Box::new(Block {
            is_global: false,
            statements,
            ..Default::default()
        })),
        ..Default::default()
    }
}

/* Creates:
 * func main() {
 *     statements
 * }
 */
pub fn create_main_func_def(statements: Vec<Hir>) -> FunctionDef {
    create_func_def("main", vec![], Type::I32, statements)
}

/* Creates
 * alias <name> = <ty>
 */
pub fn create_alias_def(name: &str, ty: Type) -> AliasDef {
    AliasDef {
        identifier: Ident::from(name.to_string()),
        value: TypeSpec {
            ty,
            ..Default::default()
        },
        ..Default::default()
    }
}

/* Creates:
 * var mut?(mutability.is_mut()) <var_name>: <var_type> = [obj]
 */
pub fn create_var_def(
    var_name: &str,
    mutability: Mutability,
    var_type: Type,
    obj: Option<Expression>,
) -> VariableDef {
    VariableDef {
        location: Location::default(),
        identifier: Ident::from(var_name.to_string()),
        var_type,
        mutability,
        value: obj.map(Box::new),
        ..Default::default()
    }
}

pub fn create_text_lit(value: &str) -> Expression {
    use tanitc_hir::hir::expressions::literal::{Literal, Text};

    Expression::Literal(Literal::Text(Text {
        location: Location::default(),
        value: value.into(),
    }))
}

pub fn create_integer_lit(value: usize) -> Expression {
    use tanitc_hir::hir::expressions::literal::{Integer, Literal};

    Expression::Literal(Literal::Integer(Integer {
        location: Location::default(),
        value,
    }))
}

pub fn create_decimal_lit(value: f64) -> Expression {
    use tanitc_hir::hir::expressions::literal::{Decimal, Literal};

    Expression::Literal(Literal::Decimal(Decimal {
        location: Location::default(),
        value,
    }))
}

/* Creates:
 * struct struct_name {
 *     fields_raw[0].0: fields_raw[0].1
 *     fields_raw[1].0: fields_raw[1].1
 *     ...
 *     fields_raw[N].0: fields_raw[N].1
 * }
 */
pub fn create_struct_lit(struct_name: &str, fields_raw: &[(&str, Expression)]) -> Expression {
    use tanitc_hir::hir::expressions::literal::{Literal, StructLiteral};

    let mut fields = Vec::<(Name, Expression)>::new();
    for (field_id, field_val) in fields_raw {
        fields.push((field_id.to_string().into(), field_val.clone()));
    }

    Expression::Literal(Literal::Struct(StructLiteral {
        location: Location::default(),
        id: struct_name.to_string().into(),
        fields,
    }))
}

/* Creates:
 * [ elements[0], elements[1], ... elements[N] ]
 */
pub fn create_array_lit(elements: Vec<Expression>) -> Expression {
    use tanitc_hir::hir::expressions::literal::{ArrayLiteral, Literal};

    Expression::Literal(Literal::Array(ArrayLiteral {
        location: Location::default(),
        elements,
    }))
}

/* Creates:
 * ( units[0], units[1], ... units[N] )
 */
pub fn create_tuple_lit(units: Vec<Expression>) -> Expression {
    use tanitc_hir::hir::expressions::literal::{Literal, TupleLiteral};

    Expression::Literal(Literal::Tuple(TupleLiteral {
        location: Location::default(),
        units,
    }))
}

pub fn create_var(name: &str) -> Expression {
    use tanitc_hir::hir::expressions::variable::Variable;

    Expression::Variable(Variable {
        location: Location::default(),
        id: name.to_string().into(),
    })
}

/* Creates:
 * func_name(args[0], args[1], ... args[N])
 */
pub fn create_call_expr(func_name: &str, args: Vec<Expression>) -> Expression {
    use tanitc_hir::hir::expressions::call::{CallArg, CallExpr, PositionalCallArg};

    let arguments = args
        .into_iter()
        .enumerate()
        .map(|(arg_idx, arg)| {
            CallArg::Positional(PositionalCallArg {
                id: arg_idx,
                expr: Box::new(arg),
                location: Location::default(),
            })
        })
        .collect();

    Expression::Call(CallExpr {
        location: Location::default(),
        expr: Box::new(create_var(func_name)),
        arguments,
    })
}

/* Creates:
 * impl struct_name {
 *     methods[0]
 *     methods[1]
 *     ...
 *     methods[N]
 * }
 */
pub fn create_impl_def(struct_name: &str, methods: Vec<FunctionDef>) -> ImplDef {
    ImplDef {
        identifier: Ident::from(struct_name.to_string()),
        methods,
        ..Default::default()
    }
}

fn create_scope_resolutions_expr_from_iter(mut ids: Peekable<Iter<&str>>) -> Expression {
    let Some(first) = ids.next() else {
        panic!("ids is empty");
    };

    if ids.peek().is_none() {
        return create_var(first);
    }

    Expression::Binary(BinaryExpr {
        operation: BinaryOperation::ScopeRes,
        lhs: Box::new(create_var(first)),
        rhs: Box::new(create_scope_resolutions_expr_from_iter(ids)),
        location: Location::default(),
    })
}

pub fn create_scope_resolutions_expr(ids: &[&str]) -> Expression {
    create_scope_resolutions_expr_from_iter(ids.into_iter().peekable())
}
