use crate::mir::MirExpr;

use super::optimized;

#[test]
fn scan_recognized_for_chunkable_dispatch_repeat() {
    let ir = optimized(
        r#"
        body = { ("\\" ("n" | "\\") | (!("\"" | "\\") ANY))* }
    "#,
    );
    assert!(matches!(&ir.rules[0].expr, MirExpr::Scan { min: 0, .. }));
}

#[test]
fn scan_skips_plain_only_repeat() {
    let ir = optimized(r#"body = { ('a'..'z')* }"#);
    assert!(matches!(
        &ir.rules[0].expr,
        MirExpr::TakeWhile {
            min: 0,
            max: None,
            ..
        }
    ));
}
