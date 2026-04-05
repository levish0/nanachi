mod expr;
mod rules;
mod state;
mod statement;

use nanachi_meta::ast::Grammar;
use proc_macro2::TokenStream;

/// Generate Rust + winnow parser code from a validated nanachi grammar.
pub fn generate(grammar: &Grammar) -> TokenStream {
    let state_code = state::generate_state(grammar);
    let rules_code = rules::generate_rules(grammar);

    quote::quote! {
        mod __nanachi {
            use nanachi::winnow::prelude::*;
            use nanachi::winnow::combinator::*;
            use nanachi::winnow::token::*;
            use nanachi::winnow::stream::Location;
            use nanachi::{Input, State};

            #state_code
            #rules_code
        }
    }
}
