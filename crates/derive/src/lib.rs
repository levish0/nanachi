use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, LitStr};

/// Derive macro that generates a winnow parser from a `.nanachi` grammar file.
///
/// # Usage
///
/// ```ignore
/// use nanachi_derive::Parser;
///
/// #[derive(Parser)]
/// #[grammar = "src/my_grammar.nanachi"]
/// struct MyParser;
/// ```
///
/// You can also inline the grammar:
///
/// ```ignore
/// #[derive(Parser)]
/// #[grammar_inline = r#"alpha = { 'a'..'z' }"#]
/// struct MyParser;
/// ```
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

    let grammar = nanachi_meta::compile(&source).map_err(|e| {
        syn::Error::new_spanned(&input.ident, format!("nanachi compile error: {e:?}"))
    })?;

    // Hidden module: __nanachi_my_parser
    let struct_name = &input.ident;
    let mod_name = quote::format_ident!("__nanachi_{}", to_snake_case(&struct_name.to_string()));

    let generated = nanachi_generator::generate_with_mod(&grammar, &mod_name);

    // Generate impl block that delegates to the hidden module
    let rule_methods: Vec<_> = grammar
        .items
        .iter()
        .filter_map(|item| match item {
            nanachi_meta::ast::Item::RuleDef(rule) => {
                let parse_fn = quote::format_ident!("parse_{}", rule.name);
                let parse_fn_detailed = quote::format_ident!("parse_{}_detailed", rule.name);
                let parse_fn_with_options =
                    quote::format_ident!("parse_{}_with_options", rule.name);
                Some(quote! {
                    pub fn #parse_fn(source: &str) -> Result<&str, String> {
                        #mod_name::#parse_fn(source)
                    }

                    pub fn #parse_fn_detailed(source: &str) -> Result<&str, String> {
                        #mod_name::#parse_fn_detailed(source)
                    }

                    pub fn #parse_fn_with_options(
                        source: &str,
                        options: nanachi::ParseOptions,
                    ) -> Result<&str, String> {
                        #mod_name::#parse_fn_with_options(source, options)
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
    // Check for #[grammar_inline = "..."]
    for attr in &input.attrs {
        if attr.path().is_ident("grammar_inline") {
            let value: LitStr = attr.parse_args()?;
            return Ok(value.value());
        }
    }

    // Check for #[grammar = "path"]
    for attr in &input.attrs {
        if attr.path().is_ident("grammar") {
            let value: LitStr = attr.parse_args()?;
            let path = value.value();

            // Resolve relative to CARGO_MANIFEST_DIR
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
        "expected #[grammar = \"path\"] or #[grammar_inline = \"...\"]",
    ))
}
