//! An immutable source reference becomes an exactly qualified C object pointer.
use super::{BorrowInput, Mapping, SharedBorrows};
use crate::c_lower::{Reader, Result, c};
use portable_backend_c::ast::CValue;
use rustc_hir as hir;

#[derive(Clone, Copy)]
pub(crate) struct CSharedBorrows;

impl Mapping for CSharedBorrows {
    type Capability = SharedBorrows;
    type Context<'tcx> = Reader<'tcx>;
    type Output = CValue;

    fn lower<'tcx>(&self, reader: &mut Reader<'tcx>, input: BorrowInput<'tcx>) -> Result<CValue> {
        let expression = input.0;
        let hir::ExprKind::AddrOf(hir::BorrowKind::Ref, hir::Mutability::Not, source) =
            expression.kind
        else {
            return Err("only immutable built-in shared borrows are implemented".into());
        };
        if !reader.checked.expr_adjustments(expression).is_empty() {
            return Err("shared-borrow compiler adjustment is not implemented".into());
        }
        let ty = reader.ty(reader.checked.expr_ty(expression))?;
        let place = reader.place(source)?;
        let address = c(reader.expressions().address_of(place))?;
        if address.ty() == &ty {
            Ok(address)
        } else {
            c(reader.expressions().add_const(ty, address))
        }
    }
}
