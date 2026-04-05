use nanachi_meta::ast::{Grammar, Item, StateKind};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generate the `ParseState` struct and its `nanachi::State` impl.
pub(crate) fn generate_state(grammar: &Grammar) -> TokenStream {
    let flags: Vec<_> = grammar
        .items
        .iter()
        .filter_map(|item| match item {
            Item::StateDecl(decl) if decl.kind == StateKind::Flag => Some(&decl.name),
            _ => None,
        })
        .collect();

    let counters: Vec<_> = grammar
        .items
        .iter()
        .filter_map(|item| match item {
            Item::StateDecl(decl) if decl.kind == StateKind::Counter => Some(&decl.name),
            _ => None,
        })
        .collect();

    let flag_fields: Vec<_> = flags
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name);
            quote! { pub #ident: bool }
        })
        .collect();

    let counter_fields: Vec<_> = counters
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name);
            quote! { pub #ident: usize }
        })
        .collect();

    let get_flag_arms: Vec<_> = flags
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name);
            quote! { #name => self.#ident }
        })
        .collect();

    let set_flag_arms: Vec<_> = flags
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name);
            quote! { #name => self.#ident = _value }
        })
        .collect();

    let get_counter_arms: Vec<_> = counters
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name);
            quote! { #name => self.#ident }
        })
        .collect();

    let set_counter_arms: Vec<_> = counters
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name);
            quote! { #name => self.#ident = _value }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone, Default)]
        pub struct ParseState {
            original_input: Vec<u8>,
            #(#flag_fields,)*
            #(#counter_fields,)*
        }

        impl ParseState {
            pub fn new(input: &str) -> Self {
                Self {
                    original_input: input.as_bytes().to_vec(),
                    ..Default::default()
                }
            }
        }

        impl State for ParseState {
            fn original_input(&self) -> &[u8] {
                &self.original_input
            }

            fn get_flag(&self, name: &str) -> bool {
                match name {
                    #(#get_flag_arms,)*
                    _ => false,
                }
            }

            fn set_flag(&mut self, name: &str, _value: bool) {
                match name {
                    #(#set_flag_arms,)*
                    _ => {}
                }
            }

            fn get_counter(&self, name: &str) -> usize {
                match name {
                    #(#get_counter_arms,)*
                    _ => 0,
                }
            }

            fn set_counter(&mut self, name: &str, _value: usize) {
                match name {
                    #(#set_counter_arms,)*
                    _ => {}
                }
            }
        }
    }
}
