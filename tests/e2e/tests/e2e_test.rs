// ── Simple grammar: identifiers ──

mod simple {
    include!(concat!(env!("OUT_DIR"), "/example_simple.rs"));
}

#[test]
fn simple_alpha_single_letter() {
    assert_eq!(simple::__nanachi::parse_alpha("a").unwrap(), "a");
    assert_eq!(simple::__nanachi::parse_alpha("Z").unwrap(), "Z");
}

#[test]
fn simple_alpha_rejects_digit() {
    simple::__nanachi::parse_alpha("1").unwrap_err();
}

#[test]
fn simple_ident_multi_char() {
    assert_eq!(
        simple::__nanachi::parse_ident("hello123").unwrap(),
        "hello123"
    );
}

#[test]
fn simple_ident_single_letter() {
    assert_eq!(simple::__nanachi::parse_ident("x").unwrap(), "x");
}

#[test]
fn simple_ident_rejects_digit_start() {
    simple::__nanachi::parse_ident("123").unwrap_err();
}

#[test]
fn simple_ident_rejects_empty() {
    simple::__nanachi::parse_ident("").unwrap_err();
}

// ── Stateful: bold markers ──

mod bold {
    include!(concat!(env!("OUT_DIR"), "/example_markdown_bold.rs"));
}

#[test]
fn bold_text_matches_non_star() {
    assert_eq!(bold::__nanachi::parse_text("x").unwrap(), "x");
}

#[test]
fn bold_text_rejects_double_star() {
    bold::__nanachi::parse_text("**").unwrap_err();
}

#[test]
fn bold_parses_bold_single_char() {
    assert_eq!(bold::__nanachi::parse_bold("**x**").unwrap(), "**x**");
}

#[test]
fn bold_parses_bold_multi_char() {
    assert_eq!(
        bold::__nanachi::parse_bold("**hello**").unwrap(),
        "**hello**"
    );
}

#[test]
fn bold_rejects_empty() {
    bold::__nanachi::parse_bold("").unwrap_err();
}

// ── Fixture: basic_rules ──

mod basic_rules {
    include!(concat!(env!("OUT_DIR"), "/fixture_basic_rules.rs"));
}

#[test]
fn basic_rules_alpha() {
    assert_eq!(basic_rules::__nanachi::parse_alpha("z").unwrap(), "z");
}

#[test]
fn basic_rules_digit() {
    assert_eq!(basic_rules::__nanachi::parse_digit("5").unwrap(), "5");
}

#[test]
fn basic_rules_ident() {
    assert_eq!(
        basic_rules::__nanachi::parse_ident("foo42").unwrap(),
        "foo42"
    );
}

#[test]
fn basic_rules_digit_rejects_alpha() {
    basic_rules::__nanachi::parse_digit("a").unwrap_err();
}
