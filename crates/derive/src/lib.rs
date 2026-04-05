use proc_macro::TokenStream;

#[proc_macro_derive(Parser, attributes(grammar, grammar_inline))]
pub fn derive_parser(input: TokenStream) -> TokenStream {
    input
}
