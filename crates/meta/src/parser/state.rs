use crate::ast::{StateDecl, StateKind};
use crate::lexer::Token;

use super::error::ParseError;
use super::tokens::TokenStream;

pub(crate) fn parse_state_decl(tokens: &mut TokenStream<'_>) -> Result<StateDecl, ParseError> {
    tokens.expect(&Token::Let)?;
    let kind = match tokens.advance() {
        Some(Token::Flag) => StateKind::Flag,
        Some(Token::Counter) => StateKind::Counter,
        other => return Err(tokens.error(format!("expected 'flag' or 'counter', got {other:?}"))),
    };
    let name = tokens.expect_ident()?;
    Ok(StateDecl { kind, name })
}
