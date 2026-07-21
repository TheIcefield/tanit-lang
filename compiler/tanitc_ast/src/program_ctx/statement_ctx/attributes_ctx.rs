use tanitc_lexer::token::Token;

/// Modifier attributes that can precede a definition.
///
/// Attributes control visibility and safety semantics:
///
/// | Attribute   | Effect                                      |
/// |-------------|---------------------------------------------|
/// | `pub`       | Makes the item visible outside its module.  |
/// | `safe`      | Marks the item as safe (default).           |
/// | `unsafe`    | Marks the item as requiring unsafe context.  |
///
/// Not all attributes are valid on every definition; the parser validates
/// compatibility and reports errors for invalid combinations.
#[derive(Default, Debug, Clone)]
pub struct AttributesCtx {
    pub pub_tkn: Option<Token>,    // ('pub')?
    pub safe_tkn: Option<Token>,   // ('safe')?
    pub unsafe_tkn: Option<Token>, // ('unsafe')?
}
