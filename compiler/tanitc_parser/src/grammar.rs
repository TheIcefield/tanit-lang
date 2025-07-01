use peg::parser;
use tanitc_ast::{
    attributes::{
        AliasAttributes, BlockAttributes, FieldAttributes, FunctionAttributes, Publicity, Safety,
        StructAttributes, UnionAttributes, VariableAttributes,
    },
    AliasDef, Ast, Block, FieldInfo, Fields, FunctionDef, ParsedTypeInfo, StructDef, TypeSpec,
    UnionDef, VariableDef,
};
use tanitc_ident::Ident;
use tanitc_messages::location::Location;
use tanitc_ty::{Mutability, Type};

parser! {
    grammar tanit_parser() for str {
        rule number() -> u32
          = n:$(['0'..='9']+) {? n.parse().or(Err("u32")) }

        rule text() -> String
            = "\"" s:$(['a'..='z']) "\"" { s.to_string() }

        rule name() -> Ident
            = n:$([ 'a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '_' | '0'..='9' ]*) { Ident::from(n.to_string()) }

        rule mutability() -> Mutability
            = value:$("mut"?) {
                if value == "mut" { Mutability::Mutable } else { Mutability::Immutable }
            }

        rule publicity() -> Publicity
            = value:$("pub"?) {
                if value == "pub" { Publicity::Public } else { Publicity::Private }
            }

        rule safety() -> Safety
            = value:$("safe" / "unsafe") {
                if value == "safe" { Safety::Safe } else { Safety::Unsafe }
            }

        // TYPE
        pub rule type_spec_common() -> TypeSpec
            = ty:$("i8" / "i16" / "i32" / "i64" / "i128" /
                  "u8" / "u16" / "u32" / "u64" / "u128" /
                  "f32" / "f64" / "bool" / "char") {
                      TypeSpec {
                          location: Location::default(),
                          info: ParsedTypeInfo { is_mut: true },
                          ty: Type::from(Ident::from(ty.to_string()))
                      }
                  }

        pub rule type_spec_ref() -> TypeSpec
            = "&" mutability:mutability() ref_to:type_spec() {
                    TypeSpec {
                        location: Location::default(),
                        info: ParsedTypeInfo { is_mut: true },
                        ty: Type::Ref { ref_to: Box::new(ref_to.ty), mutability }
                    }
                }

        pub rule type_spec_ptr() -> TypeSpec
            = "*" mutability:mutability() ptr_to:type_spec() {
                    TypeSpec {
                        location: Location::default(),
                        info: ParsedTypeInfo { is_mut: true },
                        ty: Type::Ptr(Box::new(ptr_to.ty))
                    }
                }

        pub rule type_spec() -> TypeSpec
            = ty:(type_spec_common()) {
                ty
            }

        pub rule alias_def_statement() -> Ast
            = "alias" identifier:name() "=" value:type_spec() {
                Ast::from(AliasDef {
                    location: Location::default(),
                    identifier,
                    attributes: AliasAttributes::default(),
                    value,
                })
            }

        pub rule field_def() -> FieldInfo
            = publicity:publicity() identifier:name() ":" ty:type_spec() {
                FieldInfo { attributes: FieldAttributes { publicity }, identifier, ty }
            }

        pub rule struct_def_statement() -> Ast
            = "struct" identifier:name() "{" internals:((struct_def_statement() / union_def_statement()) ** "\n") raw_fields:(field_def() ** "\n") "}" {
                let mut fields = Fields::new();
                raw_fields.iter().for_each(|f| { fields.insert(f.identifier, f.clone()); });

                Ast::from(StructDef {
                    location: Location::default(),
                    attributes: StructAttributes::default(),
                    identifier,
                    internals,
                    fields
                })
            }

        pub rule union_def_statement() -> Ast
            = "union" identifier:name() "{" internals:((struct_def_statement() / union_def_statement()) ** "\n") raw_fields:(field_def() ** "\n") "}" {
                let mut fields = Fields::new();
                raw_fields.iter().for_each(|f| { fields.insert(f.identifier, f.clone()); });

                Ast::from(UnionDef {
                    location: Location::default(),
                    attributes: UnionAttributes::default(),
                    identifier,
                    internals,
                    fields
                })
            }

        pub rule func_param() -> Ast
            = mutability:mutability() identifier:name() ":" var_type:type_spec() {
                    Ast::from(VariableDef {
                        location: Location::default(),
                        is_global: false,
                        attributes: VariableAttributes::default(),
                        identifier,
                        var_type,
                        mutability
                    })
                }

        pub rule func_attributes() -> FunctionAttributes
            = { FunctionAttributes::default() }

        pub rule func_return_type() -> TypeSpec
            = (":" ty:type_spec())? { ty.unwrap_or(TypeSpec {
                                                    location: Location::default(),
                                                    ty: Type::unit(),
                                                    info: ParsedTypeInfo { is_mut: false }
                                                })
            }

        pub rule func_def_statement() -> Ast
            = attributes:func_attributes() "func" identifier:name() "(" parameters:(func_param() ** ",") ")" return_type:func_return_type() body:local_block() {
                Ast::from(FunctionDef {
                    location: Location::default(),
                    attributes,
                    identifier,
                    parameters,
                    return_type,
                    body: Some(Box::new(body))
                })
            }


        pub rule statement() -> Ast
            = s:( alias_def_statement() / struct_def_statement() / union_def_statement() ) { s }

        pub rule statements() -> Vec<Ast>
            = ss:(statement() ** "\n") {
                    ss
                }

        pub rule global_block() -> Ast
        = statements:statements() {
                Ast::from(Block { location: Location::default(), attributes: BlockAttributes::default(), statements, is_global: true })
            }

        pub rule local_block() -> Ast
            = safety:safety() "{" statements:statements() "}" {
                Ast::from(Block { location: Location::default(), attributes: BlockAttributes { safety }, statements, is_global: false })
            }

        pub rule program() -> Ast
          = ast:global_block() { ast }
    }
}
