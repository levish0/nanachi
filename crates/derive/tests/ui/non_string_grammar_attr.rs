use nanachi_derive::Parser;

#[derive(Parser)]
#[grammar(123)]
struct NonStringGrammarAttr;

fn main() {}
