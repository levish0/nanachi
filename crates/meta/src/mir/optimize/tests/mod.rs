use crate::ir;
use crate::mir::{MirProgram, lower};

use super::optimize;

mod dispatch;
mod list;
mod patterns;
mod scan;

pub(super) fn optimized(source: &str) -> MirProgram {
    let grammar = crate::compile(source).expect("compile failed");
    let ir = ir::lower(&grammar);
    let ir = ir::optimize(ir);
    let mir = lower(&ir);
    optimize(mir)
}
