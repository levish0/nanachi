use nanachi_derive::Parser;

#[derive(Parser)]
#[grammar_inline("rule = { \"x\" @ \"y\" }")]
struct InvalidInlineGrammar;

fn main() {}
