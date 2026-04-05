use nanachi_meta::parser;
use nanachi_meta::validator::{self, ValidationError};

fn validate_valid(name: &str) {
    let path = format!(
        "{}/tests/fixtures/valid/{name}.nanachi",
        env!("CARGO_MANIFEST_DIR")
    );
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let grammar = parser::parse(&source).unwrap_or_else(|e| panic!("{path}: {e}"));
    if let Err(errors) = validator::validate(&grammar) {
        panic!("{path}: validation errors: {errors:?}");
    }
}

fn validate_invalid(name: &str) -> Vec<ValidationError> {
    let path = format!(
        "{}/tests/fixtures/invalid/{name}.nanachi",
        env!("CARGO_MANIFEST_DIR")
    );
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let grammar = parser::parse(&source).unwrap_or_else(|e| panic!("{path}: {e}"));
    validator::validate(&grammar).expect_err(&format!("{name} should have validation errors"))
}

// ── Valid grammars should pass ──

#[test]
fn valid_basic_rules() {
    validate_valid("basic_rules");
}

#[test]
fn valid_stateful_bold() {
    validate_valid("stateful_bold");
}

#[test]
fn valid_nested_formatting() {
    validate_valid("nested_formatting");
}

#[test]
fn valid_depth_and_braces() {
    validate_valid("depth_and_braces");
}

#[test]
fn valid_when_conditional() {
    validate_valid("when_conditional");
}

// ── Invalid grammars should produce errors ──

#[test]
fn invalid_undefined_rule() {
    let errors = validate_invalid("undefined_rule");
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedRule { name, .. } if name == "b"
    )));
}

#[test]
fn invalid_undefined_state() {
    let errors = validate_invalid("undefined_state");
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::UndefinedState { name, .. } if name == "inside_bold"
    )));
}

#[test]
fn invalid_wrong_state_kind() {
    let errors = validate_invalid("wrong_state_kind");
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::ExpectedCounter { name, .. } if name == "inside_bold"
    )));
}

#[test]
fn invalid_duplicates() {
    let errors = validate_invalid("duplicates");
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateState { name } if name == "x"))
    );
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateRule { name } if name == "a"))
    );
}
