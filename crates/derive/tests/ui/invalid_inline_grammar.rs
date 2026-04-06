use nanachi_derive::Parser;

#[derive(Parser)]
#[grammar_inline("rule = { \"x\" $ }")]
struct InvalidInlineGrammar;

fn main() {}
