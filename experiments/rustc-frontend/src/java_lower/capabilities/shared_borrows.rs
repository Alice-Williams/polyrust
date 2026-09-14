use super::{BorrowInput, Mapping, SharedBorrows};
use crate::java_lower::{Reader, Result, TypePlan, Value};
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(crate) struct JavaSharedBorrows;

impl Mapping for JavaSharedBorrows {
    type Capability = SharedBorrows;
    type Context<'tcx> = Reader<'tcx>;
    type Output = Value;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: BorrowInput<'tcx>) -> Result<Value> {
        let expression = input.0;
        let hir::ExprKind::AddrOf(hir::BorrowKind::Ref, hir::Mutability::Not, source) =
            expression.kind
        else {
            return Err("only immutable built-in shared borrows are implemented".into());
        };
        if !reader.checked.expr_adjustments(expression).is_empty() {
            return Err("shared-borrow compiler adjustment is not implemented".into());
        }
        let expected = reader.ty(reader.checked.expr_ty(expression))?;
        let place = reader.place(source)?;
        let plan = TypePlan::Shared(Box::new(place.plan().clone()));
        if plan != expected {
            return Err("shared-reference representation differs from compiler type".into());
        }
        let value = Value::new(plan, place.value().into_expression())?;
        #[cfg(java_ast_probe)]
        crate::java_lower::assertions::borrow(reader, reader.checked.expr_ty(expression), &value);
        Ok(value)
    }
}
