//! Even backend siblings cannot invent the admitted scalar value of an HIR node.
use crate::source_capabilities::{LiteralInput, LiteralValue};

#[allow(dead_code)]
fn forge<'tcx>(expression: &'tcx rustc_hir::Expr<'tcx>) {
    let _ = LiteralInput {
        value: LiteralValue::I64(0),
        _expression: expression,
    };
}
