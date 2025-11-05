use tanitc_lexer::token::Token;

#[derive(Default, Debug, Clone)]
pub struct AttributesCtx {
    pub pub_tkn: Option<Token>,    // ('pub')?
    pub safe_tkn: Option<Token>,   // ('safe')?
    pub unsafe_tkn: Option<Token>, // ('unsafe')?
}
