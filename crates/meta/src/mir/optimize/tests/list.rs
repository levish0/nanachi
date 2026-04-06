use crate::mir::MirExpr;

use super::optimized;

#[test]
fn separated_list_recognized_for_item_tail_repeat() {
    let ir = optimized(
        r#"
        ws = { (" " | "\n")* }
        digit = { '0'..'9'+ }
        comma_value = { ws "," ws digit }
        items = { digit comma_value* }
    "#,
    );
    let items = ir.rules.iter().find(|r| r.name == "items").unwrap();
    assert!(matches!(&items.expr, MirExpr::SeparatedList { .. }));
}
