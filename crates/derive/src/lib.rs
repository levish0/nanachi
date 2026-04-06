use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr};

#[proc_macro_derive(Parser, attributes(grammar, grammar_inline))]
pub fn derive_parser(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    match derive_parser_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_parser_impl(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let source = extract_grammar_source(input)?;

    let grammar = faputa_meta::compile(&source).map_err(|e| {
        syn::Error::new_spanned(&input.ident, format!("faputa compile error: {e:?}"))
    })?;

    let struct_name = &input.ident;
    let mod_name = quote::format_ident!("__faputa_{}", to_snake_case(&struct_name.to_string()));

    let generated = faputa_generator::generate_with_mod(&grammar, &mod_name);

    let rule_methods: Vec<_> = grammar
        .items
        .iter()
        .filter_map(|item| match item {
            faputa_meta::ast::Item::RuleDef(rule) => {
                let parse_fn = quote::format_ident!("parse_{}", rule.name);
                Some(quote! {
                    pub fn #parse_fn(source: &str) -> Result<&str, String> {
                        #mod_name::#parse_fn(source)
                    }
                })
            }
            _ => None,
        })
        .collect();

    Ok(quote! {
        #generated

        #[allow(dead_code)]
        impl #struct_name {
            #(#rule_methods)*
        }
    })
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

fn extract_grammar_source(input: &DeriveInput) -> syn::Result<String> {
    for attr in &input.attrs {
        if attr.path().is_ident("grammar_inline") {
            let value: LitStr = attr.parse_args()?;
            return Ok(value.value());
        }
    }

    for attr in &input.attrs {
        if attr.path().is_ident("grammar") {
            let value: LitStr = attr.parse_args()?;
            let path = value.value();

            let base = std::env::var("CARGO_MANIFEST_DIR")
                .map_err(|_| syn::Error::new_spanned(&value, "CARGO_MANIFEST_DIR not set"))?;
            let full_path = std::path::Path::new(&base).join(&path);

            let source = std::fs::read_to_string(&full_path).map_err(|e| {
                syn::Error::new_spanned(&value, format!("cannot read {}: {e}", full_path.display()))
            })?;

            return Ok(source);
        }
    }

    Err(syn::Error::new_spanned(
        &input.ident,
        "expected #[grammar(\"path\")] or #[grammar_inline(\"...\")]",
    ))
}
