use nanachi_meta::ast::{Grammar, Item, RuleDef};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::expr::generate_expr;
use crate::statement::generate_statements;

/// Generate a function for each rule in the grammar.
pub(crate) fn generate_rules(grammar: &Grammar) -> TokenStream {
    let fns: Vec<_> = grammar
        .items
        .iter()
        .filter_map(|item| match item {
            Item::RuleDef(rule) => Some(generate_rule(rule)),
            _ => None,
        })
        .collect();

    quote! { #(#fns)* }
}

fn generate_rule(rule: &RuleDef) -> TokenStream {
    let fn_name = format_ident!("{}", rule.name);
    let rule_name = &rule.name;

    let guard_code = generate_statements(&rule.body.statements);
    let expr_code = generate_expr(&rule.body.expr);

    // Rules with guards need a manual closure since guards do early returns.
    let has_statements = !rule.body.statements.is_empty();

    if has_statements {
        quote! {
            fn #fn_name<'i>(input: &mut Input<'i, ParseState>) -> ModalResult<()> {
                winnow::combinator::trace(#rule_name, |input: &mut Input<'_, ParseState>| {
                    #guard_code
                    (#expr_code).void().parse_next(input)
                })
                .parse_next(input)
            }
        }
    } else {
        quote! {
            fn #fn_name<'i>(input: &mut Input<'i, ParseState>) -> ModalResult<()> {
                winnow::combinator::trace(#rule_name, (#expr_code).void())
                    .parse_next(input)
            }
        }
    }
}
