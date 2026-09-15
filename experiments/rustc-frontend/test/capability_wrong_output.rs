use super::{CDirectCalls, CLiteralValues, CallInput, LiteralInput, Mapping};
use crate::c_lower::Reader;
use portable_backend_c::ast::CPlace;
use rustc_hir as hir;

#[allow(dead_code)]
fn must_not_compile<'tcx>(reader: &mut Reader<'tcx>, value: &'tcx hir::Expr<'tcx>) {
    let input = LiteralInput::read(reader.checked, value).unwrap();
    let _: CPlace = CLiteralValues.lower(reader, input).unwrap();
    let _: CPlace = CDirectCalls.lower(reader, CallInput(value)).unwrap();
}
