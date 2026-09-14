use super::{CDirectCalls, CLiteralValues, ComparisonInput, LiteralInput, Mapping};
use crate::c_lower::Reader;
use rustc_hir as hir;

#[allow(dead_code)]
fn must_not_compile<'tcx>(reader: &mut Reader<'tcx>, value: &'tcx hir::Expr<'tcx>) {
    let _ = CLiteralValues.lower(reader, ComparisonInput(value));
    let _ = CDirectCalls.lower(reader, LiteralInput(value));
}
