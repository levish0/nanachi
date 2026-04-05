use nanachi_meta::parser;
use nanachi_meta::validator::{self, ValidationError};

fn parse_and_validate(name: &str) -> Result<(), Vec<ValidationError>> {
    let path = format!(
        "{}/tests/fixtures/{name}.nanachi",
        env!("CARGO_MANIFEST_DIR")
    );
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let grammar = parser::parse(&source).unwrap_or_else(|e| panic!("{path}: {e}"));
    validator::validate(&grammar)
}

fn expect_errors(name: &str) -> Vec<ValidationError> {
    parse_and_validate(name).expect_err(&format!("{name} should have validation errors"))
}

// ── Valid grammars should pass ──

#[test]
fn valid_basic_rules() {
    parse_and_validate("basic_rules").unwrap();
}

#[test]
fn valid_stateful_bold() {
    parse_and_validate("stateful_bold").unwrap();
}

#[test]
fn valid_nested_formatting() {
    parse_and_validate("nested_formatting").unwrap();
}

#[test]
fn valid_depth_and_braces() {
    parse_and_validate("depth_and_braces").unwrap();
}

#[test]
fn valid_when_conditional() {
    parse_and_validate("when_conditional").unwrap();
}

// ── Invalid grammars should produce errors ──

#[test]
fn invalid_undefined_rule() {
    let errors = expect_errors("invalid_undefined_rule");
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedRule { name, .. } if name == "b"
    )));
}

#[test]
fn invalid_undefined_state() {
    let errors = expect_errors("invalid_undefined_state");
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedState { name, .. } if name == "inside_bold"
    )));
}

#[test]
fn invalid_wrong_state_kind() {
    let errors = expect_errors("invalid_wrong_state_kind");
    // emit on a flag → expected counter
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::ExpectedCounter { name, .. } if name == "inside_bold"
    )));
}

#[test]
fn invalid_duplicates() {
    let errors = expect_errors("invalid_duplicates");
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::DuplicateState { name } if name == "x")));
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::DuplicateRule { name } if name == "a")));
}

#[test]
fn invalid_shadows_builtin() {
    let errors = expect_errors("invalid_shadows_builtin");
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::ShadowsBuiltin { name } if name == "SOI"
    )));
}
