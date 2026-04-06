use crate::ir::{CharRange, IrExpr, IrProgram, lower};

use super::optimize;

fn optimized(source: &str) -> IrProgram {
    let grammar = crate::compile(source).expect("compile failed");
    let ir = lower(&grammar);
    optimize(ir)
}

#[test]
fn charset_merge_in_choice() {
    let ir = optimized(r#"alpha = { 'a'..'z' | 'A'..'Z' | "_" }"#);
    match &ir.rules[0].expr {
        IrExpr::CharSet(ranges) => assert_eq!(ranges.len(), 3),
        other => panic!("expected CharSet, got {other:?}"),
    }
}

#[test]
fn charset_merge_all_ranges() {
    let ir = optimized("alpha = { 'a'..'z' | 'A'..'Z' }");
    match &ir.rules[0].expr {
        IrExpr::CharSet(ranges) => assert_eq!(ranges.len(), 2),
        other => panic!("expected CharSet, got {other:?}"),
    }
}

#[test]
fn adjacent_ranges_coalesced() {
    let ir = optimized("az = { 'a'..'m' | 'n'..'z' }");
    match &ir.rules[0].expr {
        IrExpr::CharSet(ranges) => {
            assert_eq!(ranges.len(), 1);
            assert_eq!(ranges[0], CharRange::new('a', 'z'));
        }
        other => panic!("expected CharSet, got {other:?}"),
    }
}

#[test]
fn literal_fusion() {
    let ir = optimized(r#"kw = { "h" "e" "l" "l" "o" }"#);
    assert_eq!(ir.rules[0].expr, IrExpr::Literal("hello".into()));
}

#[test]
fn trivial_rule_inlined() {
    let ir = optimized(
        r#"
        digit = { '0'..'9' }
        number = { digit+ }
    "#,
    );
    let number = ir.rules.iter().find(|r| r.name == "number").unwrap();
    assert!(
        matches!(
            &number.expr,
            IrExpr::TakeWhile {
                min: 1,
                max: None,
                ..
            }
        ),
        "expected TakeWhile, got {:?}",
        &number.expr
    );
}

#[test]
fn inlined_rules_kept_with_inline_flag() {
    let ir = optimized(
        r#"
        digit = { '0'..'9' }
        number = { digit+ }
    "#,
    );
    assert_eq!(ir.rules.len(), 2);
    let digit = ir.rules.iter().find(|r| r.name == "digit").unwrap();
    assert!(digit.inline);
    let number = ir.rules.iter().find(|r| r.name == "number").unwrap();
    assert!(!number.inline);
}

#[test]
fn non_trivial_rule_not_inlined() {
    let ir = optimized(
        r#"
        alpha = { 'a'..'z' | 'A'..'Z' }
        digit = { '0'..'9' }
        ident = { alpha (alpha | digit)* }
    "#,
    );
    let ident = ir.rules.iter().find(|r| r.name == "ident").unwrap();
    match &ident.expr {
        IrExpr::Seq(items) => {
            assert!(matches!(&items[0], IrExpr::CharSet(_)));
            assert!(
                matches!(&items[1], IrExpr::TakeWhile { .. }),
                "expected TakeWhile, got {:?}",
                &items[1]
            );
        }
        other => panic!("expected Seq, got {other:?}"),
    }
}

#[test]
fn flatten_nested_seq() {
    let ir = optimized(r#"r = { "a" ("b" "c") }"#);
    match &ir.rules[0].expr {
        IrExpr::Literal(s) => assert_eq!(s, "abc"),
        other => panic!("expected fused Literal, got {other:?}"),
    }
}

#[test]
fn stateful_rule_not_inlined() {
    let ir = optimized(
        r#"
        let flag active
        special = {
            guard active
            "x"
        }
        main = { special }
    "#,
    );
    assert!(ir.rules.iter().any(|r| r.name == "special"));
    let main = ir.rules.iter().find(|r| r.name == "main").unwrap();
    assert!(matches!(&main.expr, IrExpr::RuleRef(_)));
}

#[test]
fn single_char_literal_to_charset_in_choice() {
    let ir = optimized(r#"ws = { " " | "\t" | "\n" | "\r" }"#);
    match &ir.rules[0].expr {
        IrExpr::CharSet(ranges) => assert_eq!(ranges.len(), 3),
        other => panic!("expected CharSet, got {other:?}"),
    }
}

#[test]
fn take_while_recognized() {
    let ir = optimized("d = { '0'..'9'* }");
    assert!(matches!(
        &ir.rules[0].expr,
        IrExpr::TakeWhile {
            min: 0,
            max: None,
            ..
        }
    ));
}

#[test]
fn take_while_from_choice_repeat() {
    let ir = optimized(r#"ws = { (" " | "\t" | "\n" | "\r")* }"#);
    assert!(matches!(
        &ir.rules[0].expr,
        IrExpr::TakeWhile {
            min: 0,
            max: None,
            ..
        }
    ));
}

#[test]
fn take_while_bounded() {
    let ir = optimized("d = { '0'..'9'{3} }");
    match &ir.rules[0].expr {
        IrExpr::TakeWhile { min, max, .. } => {
            assert_eq!(*min, 3);
            assert_eq!(*max, Some(3));
        }
        other => panic!("expected TakeWhile, got {other:?}"),
    }
}

#[test]
fn ref_count_entry_point() {
    let ir = optimized(
        r#"
        main = { "hello" }
    "#,
    );
    assert_eq!(ir.rules[0].ref_count, 0);
}

#[test]
fn ref_count_internal_rule() {
    let ir = optimized(
        r#"
        let flag active
        special = {
            guard active
            "x"
        }
        a = { special }
        b = { special }
    "#,
    );
    let special = ir.rules.iter().find(|r| r.name == "special").unwrap();
    assert_eq!(special.ref_count, 2);
}

#[test]
fn take_while_bounded_stays_take_while() {
    let ir = optimized("d = { '0'..'9'{3} }");
    assert!(matches!(&ir.rules[0].expr, IrExpr::TakeWhile { .. }));
}

#[test]
fn dispatch_recognized_for_disjoint_choice() {
    let ir = optimized(r#"value = { "true" | "false" | "null" }"#);
    assert!(matches!(&ir.rules[0].expr, IrExpr::Dispatch(_)));
}

#[test]
fn dispatch_skips_overlapping_choice() {
    let ir = optimized(r#"value = { "true" | "trick" }"#);
    assert!(matches!(&ir.rules[0].expr, IrExpr::Choice(_)));
}

#[test]
fn dispatch_works_through_rule_refs() {
    let ir = optimized(
        r#"
        number = { "-"? '0'..'9'+ }
        array = { "[" "]"? }
        value = { number | array }
    "#,
    );
    let value = ir.rules.iter().find(|r| r.name == "value").unwrap();
    assert!(matches!(&value.expr, IrExpr::Dispatch(_)));
}
