use nanachi_meta::ast::*;
use nanachi_meta::parser;

fn parse_fixture(name: &str) -> Grammar {
    let path = format!(
        "{}/tests/fixtures/valid/{name}.nanachi",
        env!("CARGO_MANIFEST_DIR")
    );
    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    parser::parse(&source).unwrap_or_else(|e| panic!("{path}: {e}"))
}

// ── Fixture-based tests ──

#[test]
fn parse_basic_rules() {
    let grammar = parse_fixture("basic_rules");

    // 3 rules: alpha, digit, ident
    assert_eq!(grammar.items.len(), 3);

    let Item::RuleDef(alpha) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(alpha.name, "alpha");
    assert_eq!(
        alpha.body.expr,
        Expr::Choice(vec![Expr::CharRange('a', 'z'), Expr::CharRange('A', 'Z'),])
    );

    let Item::RuleDef(ident) = &grammar.items[2] else {
        panic!("expected RuleDef");
    };
    assert_eq!(ident.name, "ident");
    // ident = { alpha (alpha | digit)* }
    assert_eq!(
        ident.body.expr,
        Expr::Seq(vec![
            Expr::Ident("alpha".to_string()),
            Expr::Repeat {
                expr: Box::new(Expr::Group(Box::new(Expr::Choice(vec![
                    Expr::Ident("alpha".to_string()),
                    Expr::Ident("digit".to_string()),
                ])))),
                kind: RepeatKind::ZeroOrMore,
            },
        ])
    );
}

#[test]
fn parse_stateful_bold() {
    let grammar = parse_fixture("stateful_bold");

    // 1 state decl + 3 rules
    assert_eq!(grammar.items.len(), 4);

    assert_eq!(
        grammar.items[0],
        Item::StateDecl(StateDecl {
            kind: StateKind::Flag,
            name: "inside_bold".to_string(),
        })
    );

    let Item::RuleDef(bold) = &grammar.items[2] else {
        panic!("expected RuleDef");
    };
    assert_eq!(bold.name, "bold");
    assert_eq!(bold.body.statements.len(), 1);
    assert_eq!(
        bold.body.statements[0],
        Statement::Guard(GuardStmt {
            condition: GuardCondition::NotFlag("inside_bold".to_string()),
        })
    );

    let Expr::With(with) = &bold.body.expr else {
        panic!("expected With expr");
    };
    assert_eq!(with.flag, "inside_bold");
}

#[test]
fn parse_nested_formatting() {
    let grammar = parse_fixture("nested_formatting");

    // 5 state decls + 6 rules = 11 items
    assert_eq!(grammar.items.len(), 11);

    // Check header has emit + guard LINE_START
    let Item::RuleDef(header) = &grammar.items[9] else {
        panic!("expected RuleDef for header");
    };
    assert_eq!(header.name, "header");
    assert_eq!(header.body.statements.len(), 3); // guard LINE_START, guard !inside_header, emit

    assert_eq!(
        header.body.statements[0],
        Statement::Guard(GuardStmt {
            condition: GuardCondition::Builtin(BuiltinPredicate::LineStart),
        })
    );
    assert_eq!(
        header.body.statements[2],
        Statement::Emit(EmitStmt {
            counter: "section_counter".to_string(),
        })
    );
}

#[test]
fn parse_depth_and_braces() {
    let grammar = parse_fixture("depth_and_braces");

    // 1 state decl + 5 rules (document, block, raw_block, raw_content, paragraph)
    assert_eq!(grammar.items.len(), 6);

    // document uses depth_limit
    let Item::RuleDef(doc) = &grammar.items[1] else {
        panic!("expected RuleDef");
    };
    assert_eq!(doc.name, "document");
    let Expr::DepthLimit(dl) = &doc.body.expr else {
        panic!("expected DepthLimit expr");
    };
    assert_eq!(dl.limit, 64);

    // raw_block uses with increment
    let Item::RuleDef(raw) = &grammar.items[3] else {
        panic!("expected RuleDef");
    };
    assert_eq!(raw.name, "raw_block");
}

#[test]
fn parse_when_conditional() {
    let grammar = parse_fixture("when_conditional");

    // 2 state decls + 1 rule
    assert_eq!(grammar.items.len(), 3);

    let Item::RuleDef(newline) = &grammar.items[2] else {
        panic!("expected RuleDef");
    };
    assert_eq!(newline.name, "newline");

    // guard !inside_header
    assert_eq!(
        newline.body.statements[0],
        Statement::Guard(GuardStmt {
            condition: GuardCondition::NotFlag("inside_header".to_string()),
        })
    );

    // Body starts with when expression in a sequence
    let Expr::Seq(seq) = &newline.body.expr else {
        panic!("expected Seq expr, got {:?}", newline.body.expr);
    };

    let Expr::When(when) = &seq[0] else {
        panic!("expected When expr");
    };
    assert_eq!(
        when.condition,
        GuardCondition::Compare {
            name: "trim_brace_depth".to_string(),
            op: CompareOp::Gt,
            value: 0,
        }
    );
}

// ── Inline unit tests (kept for fine-grained edge cases) ──

#[test]
fn parse_simple_sequence() {
    let grammar = parser::parse(r#"greeting = { "hello" "world" }"#).unwrap();
    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(
        rule.body.expr,
        Expr::Seq(vec![
            Expr::StringLit("hello".to_string()),
            Expr::StringLit("world".to_string()),
        ])
    );
}

#[test]
fn parse_choice() {
    let grammar = parser::parse(r#"ab = { "a" | "b" | "c" }"#).unwrap();
    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(
        rule.body.expr,
        Expr::Choice(vec![
            Expr::StringLit("a".to_string()),
            Expr::StringLit("b".to_string()),
            Expr::StringLit("c".to_string()),
        ])
    );
}

#[test]
fn parse_repeat_range() {
    let grammar = parser::parse("hashes = { \"#\"{1,6} }").unwrap();
    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(
        rule.body.expr,
        Expr::Repeat {
            expr: Box::new(Expr::StringLit("#".to_string())),
            kind: RepeatKind::Range(1, 6),
        }
    );
}

#[test]
fn parse_neg_lookahead() {
    let grammar = parser::parse(r#"not_end = { !("}}}" ) ANY }"#).unwrap();
    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(
        rule.body.expr,
        Expr::Seq(vec![
            Expr::NegLookahead(Box::new(Expr::Group(Box::new(Expr::StringLit(
                "}}}".to_string()
            ))))),
            Expr::Builtin(BuiltinPredicate::Any),
        ])
    );
}

#[test]
fn parse_empty_grammar() {
    let grammar = parser::parse("").unwrap();
    assert!(grammar.items.is_empty());
}

#[test]
fn parse_comments_only() {
    let grammar = parser::parse("// just a comment\n// another one\n").unwrap();
    assert!(grammar.items.is_empty());
}
