use crate::mir::MirExpr;

use super::optimized;

#[test]
fn take_while_recognized() {
    let ir = optimized("d = { '0'..'9'* }");
    assert!(matches!(
        &ir.rules[0].expr,
        MirExpr::TakeWhile {
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
        MirExpr::TakeWhile {
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
        MirExpr::TakeWhile { min, max, .. } => {
            assert_eq!(*min, 3);
            assert_eq!(*max, Some(3));
        }
        other => panic!("expected TakeWhile, got {other:?}"),
    }
}
