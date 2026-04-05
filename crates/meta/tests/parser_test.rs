use nanachi_meta::ast::*;
use nanachi_meta::parser;

#[test]
fn parse_state_declarations() {
    let grammar = parser::parse("let flag inside_bold\nlet counter section_counter").unwrap();
    assert_eq!(grammar.items.len(), 2);
    assert_eq!(
        grammar.items[0],
        Item::StateDecl(StateDecl {
            kind: StateKind::Flag,
            name: "inside_bold".to_string(),
        })
    );
    assert_eq!(
        grammar.items[1],
        Item::StateDecl(StateDecl {
            kind: StateKind::Counter,
            name: "section_counter".to_string(),
        })
    );
}

#[test]
fn parse_simple_rule() {
    let grammar = parser::parse(r#"greeting = { "hello" "world" }"#).unwrap();
    assert_eq!(grammar.items.len(), 1);

    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(rule.name, "greeting");
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
fn parse_repetition() {
    let grammar = parser::parse("items = { item+ }").unwrap();
    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(
        rule.body.expr,
        Expr::Repeat {
            expr: Box::new(Expr::Ident("item".to_string())),
            kind: RepeatKind::OneOrMore,
        }
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
fn parse_char_range() {
    let grammar = parser::parse("alpha = { 'a'..'z' }").unwrap();
    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(rule.body.expr, Expr::CharRange('a', 'z'));
}

#[test]
fn parse_lookahead() {
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
fn parse_guard_flag() {
    let grammar = parser::parse(
        r#"bold = {
    guard !inside_bold
    "**" inline+ "**"
}"#,
    )
    .unwrap();

    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(rule.body.statements.len(), 1);
    assert_eq!(
        rule.body.statements[0],
        Statement::Guard(GuardStmt {
            condition: GuardCondition::NotFlag("inside_bold".to_string()),
        })
    );
}

#[test]
fn parse_guard_compare() {
    let grammar = parser::parse(
        r#"deep = {
    guard depth > 0
    item
}"#,
    )
    .unwrap();

    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(
        rule.body.statements[0],
        Statement::Guard(GuardStmt {
            condition: GuardCondition::Compare {
                name: "depth".to_string(),
                op: CompareOp::Gt,
                value: 0,
            },
        })
    );
}

#[test]
fn parse_with_flag() {
    let grammar = parser::parse(
        r#"bold = {
    with inside_bold {
        "**" inline+ "**"
    }
}"#,
    )
    .unwrap();

    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };

    let Expr::With(with) = &rule.body.expr else {
        panic!("expected With expr");
    };
    assert_eq!(with.flag, "inside_bold");
}

#[test]
fn parse_with_increment() {
    let grammar = parser::parse(
        r#"nested = {
    with depth += 1 {
        content*
    }
}"#,
    )
    .unwrap();

    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };

    let Expr::WithIncrement(w) = &rule.body.expr else {
        panic!("expected WithIncrement expr");
    };
    assert_eq!(w.counter, "depth");
    assert_eq!(w.amount, 1);
}

#[test]
fn parse_emit() {
    let grammar = parser::parse(
        "header = {\n    emit section_counter\n    \"#\"{1,6} \" \" inline+\n}",
    )
    .unwrap();

    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };
    assert_eq!(
        rule.body.statements[0],
        Statement::Emit(EmitStmt {
            counter: "section_counter".to_string(),
        })
    );
}

#[test]
fn parse_depth_limit() {
    let grammar = parser::parse(
        r#"nested = {
    depth_limit(64) {
        bold | italic
    }
}"#,
    )
    .unwrap();

    let Item::RuleDef(rule) = &grammar.items[0] else {
        panic!("expected RuleDef");
    };

    let Expr::DepthLimit(dl) = &rule.body.expr else {
        panic!("expected DepthLimit expr");
    };
    assert_eq!(dl.limit, 64);
}

#[test]
fn parse_full_grammar() {
    let input = r#"
let flag inside_bold
let flag inside_italic
let counter section_counter

inline = { bold | italic | text }

bold = {
    guard !inside_bold
    with inside_bold {
        "**" inline+ "**"
    }
}

italic = {
    guard !inside_italic
    with inside_italic {
        "*" inline+ "*"
    }
}

text = { (!("*") ANY)+ }
"#;

    let grammar = parser::parse(input).unwrap();
    assert_eq!(grammar.items.len(), 7); // 3 state + 4 rules
}
