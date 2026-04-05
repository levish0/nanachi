use crate::ast::{Grammar, Item, RuleBody, RuleDef};
use crate::lexer::Token;

use super::error::ParseError;
use super::expr::parse_choice;
use super::state::parse_state_decl;
use super::statement::parse_statements;
use super::tokens::TokenStream;

pub(crate) fn parse_grammar(tokens: &mut TokenStream<'_>) -> Result<Grammar, ParseError> {
    let mut items = Vec::new();

    while !tokens.at_end() {
        let item = parse_item(tokens)?;
        items.push(item);
    }

    Ok(Grammar { items })
}

fn parse_item(tokens: &mut TokenStream<'_>) -> Result<Item, ParseError> {
    match tokens.peek() {
        Some(Token::Let) => parse_state_decl(tokens).map(Item::StateDecl),
        Some(Token::Ident(_)) => parse_rule_def(tokens).map(Item::RuleDef),
        other => Err(tokens.error(format!("expected 'let' or rule name, got {other:?}"))),
    }
}

fn parse_rule_def(tokens: &mut TokenStream<'_>) -> Result<RuleDef, ParseError> {
    let name = tokens.expect_ident()?;
    tokens.expect(&Token::Eq)?;
    tokens.expect(&Token::LBrace)?;

    let statements = parse_statements(tokens)?;
    let expr = parse_choice(tokens)?;

    tokens.expect(&Token::RBrace)?;

    Ok(RuleDef {
        name,
        body: RuleBody { statements, expr },
    })
}
