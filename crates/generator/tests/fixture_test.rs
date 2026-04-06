/// Fixture-based codegen tests.
///
/// Each valid .nanachi fixture from nanachi_meta is:
/// 1. Parsed + validated
/// 2. Fed to the generator
/// 3. Checked that the output is valid Rust (parseable by syn)

fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to find workspace root")
}

fn generate_fixture(name: &str) -> String {
    let path = workspace_root().join(format!("fixtures/valid/{name}.nanachi"));
    let path = path.display().to_string();
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let grammar = nanachi_meta::compile(&source).unwrap_or_else(|e| panic!("{path}: {e:?}"));
    let tokens = nanachi_generator::generate(&grammar);
    tokens.to_string()
}

fn assert_valid_rust(code: &str, fixture: &str) {
    syn::parse_str::<syn::File>(code)
        .unwrap_or_else(|e| panic!("{fixture}: generated code is not valid Rust: {e}\n\n{code}"));
}

// ── Fixture tests ──

#[test]
fn fixture_basic_rules() {
    let code = generate_fixture("basic_rules");
    assert_valid_rust(&code, "basic_rules");
    assert!(code.contains("fn alpha"));
    assert!(code.contains("fn digit"));
    assert!(code.contains("fn ident"));
}

#[test]
fn fixture_stateful_bold() {
    let code = generate_fixture("stateful_bold");
    assert_valid_rust(&code, "stateful_bold");
    assert!(code.contains("inside_bold : bool"));
    assert!(code.contains("fn bold"));
    assert!(code.contains("set_flag"));
}

#[test]
fn fixture_nested_formatting() {
    let code = generate_fixture("nested_formatting");
    assert_valid_rust(&code, "nested_formatting");
    assert!(code.contains("inside_bold : bool"));
    assert!(code.contains("inside_italic : bool"));
    assert!(code.contains("inside_header : bool"));
    assert!(code.contains("section_counter : usize"));
}

#[test]
fn fixture_depth_and_braces() {
    let code = generate_fixture("depth_and_braces");
    assert_valid_rust(&code, "depth_and_braces");
    assert!(code.contains("fn document"));
    assert!(code.contains("__recursion_depth"));
}

#[test]
fn fixture_when_conditional() {
    let code = generate_fixture("when_conditional");
    assert_valid_rust(&code, "when_conditional");
    assert!(code.contains("get_counter"));
}

#[test]
fn fixture_chaos_combo() {
    let code = generate_fixture("chaos_combo");
    assert_valid_rust(&code, "chaos_combo");
}

// ── Example file tests ──

fn generate_example(name: &str) -> String {
    let path = workspace_root().join(format!("examples/{name}.nanachi"));
    let path = path.display().to_string();
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let grammar = nanachi_meta::compile(&source).unwrap_or_else(|e| panic!("{path}: {e:?}"));
    let tokens = nanachi_generator::generate(&grammar);
    tokens.to_string()
}

#[test]
fn example_simple() {
    let code = generate_example("simple");
    assert_valid_rust(&code, "simple");
}

#[test]
fn example_markdown_bold() {
    let code = generate_example("markdown_bold");
    assert_valid_rust(&code, "markdown_bold");
}

#[test]
fn example_nested_braces() {
    let code = generate_example("nested_braces");
    assert_valid_rust(&code, "nested_braces");
}
