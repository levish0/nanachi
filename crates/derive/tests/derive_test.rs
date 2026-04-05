use nanachi::ParseOptions;
use nanachi_derive::Parser;

// ── #[grammar_inline] ──

#[derive(Parser)]
#[grammar_inline("alpha = { 'a'..'z' | 'A'..'Z' }")]
struct AlphaParser;

#[test]
fn inline_grammar_parses() {
    assert_eq!(AlphaParser::parse_alpha("x").unwrap(), "x");
}

#[test]
fn inline_grammar_rejects() {
    AlphaParser::parse_alpha("1").unwrap_err();
}

#[test]
fn inline_grammar_detailed_errors_are_opt_in() {
    let err = AlphaParser::parse_alpha("1").unwrap_err();
    assert!(!err.contains("expected"));

    let detailed = AlphaParser::parse_alpha_detailed("1").unwrap_err();
    assert!(detailed.contains("expected"));

    let with_options =
        AlphaParser::parse_alpha_with_options("1", ParseOptions::detailed()).unwrap_err();
    assert!(with_options.contains("expected"));
}

// ── #[grammar] with file path ──

#[derive(Parser)]
#[grammar("../../examples/simple.nanachi")]
struct SimpleParser;

#[test]
fn file_grammar_ident() {
    assert_eq!(SimpleParser::parse_ident("hello123").unwrap(), "hello123");
}

#[test]
fn file_grammar_alpha() {
    assert_eq!(SimpleParser::parse_alpha("Z").unwrap(), "Z");
}

#[test]
fn file_grammar_rejects_digit_start() {
    SimpleParser::parse_ident("123").unwrap_err();
}

// ── Stateful grammar ──

#[derive(Parser)]
#[grammar("../../examples/markdown_bold.nanachi")]
struct BoldParser;

#[test]
fn bold_parses() {
    assert_eq!(BoldParser::parse_bold("**x**").unwrap(), "**x**");
}

#[test]
fn bold_rejects_empty() {
    BoldParser::parse_bold("").unwrap_err();
}
