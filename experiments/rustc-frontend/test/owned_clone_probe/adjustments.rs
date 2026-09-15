//! Independent typed assertions for the selected source borrow representations.
use rustc_hir::{self as hir, def_id::LocalDefId};
use rustc_middle::ty::{
    self, Ty, TyCtxt,
    adjustment::{Adjust, AutoBorrow, AutoBorrowMutability, DerefAdjustKind},
};

pub(super) fn check<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    expression: &hir::Expr<'tcx>,
    self_ty: Ty<'tcx>,
) {
    let checked = tcx.typeck(owner);
    let (receiver, adjustments, explicit) = match expression.kind {
        hir::ExprKind::MethodCall(_, receiver, [], _) => {
            (receiver, checked.expr_adjustments(receiver), false)
        }
        hir::ExprKind::Call(_, [borrow]) => {
            let hir::ExprKind::AddrOf(hir::BorrowKind::Ref, hir::Mutability::Not, receiver) =
                borrow.kind
            else {
                panic!("explicit shared borrow")
            };
            assert!(checked.expr_adjustments(receiver).is_empty());
            assert!(
                matches!(checked.expr_ty(borrow).kind(), ty::Ref(_, target, hir::Mutability::Not) if *target == self_ty)
            );
            (receiver, checked.expr_adjustments(borrow), true)
        }
        _ => panic!("selected clone syntax"),
    };
    assert_eq!(checked.expr_ty(receiver), self_ty);
    let borrow = if explicit {
        let [deref, borrow] = adjustments else {
            panic!("one dereference and reborrow")
        };
        assert!(matches!(
            deref.kind,
            Adjust::Deref(DerefAdjustKind::Builtin)
        ));
        assert_eq!(deref.target, self_ty);
        borrow
    } else {
        let [borrow] = adjustments else {
            panic!("one autoref")
        };
        borrow
    };
    assert!(matches!(
        borrow.kind,
        Adjust::Borrow(AutoBorrow::Ref(AutoBorrowMutability::Not))
    ));
    assert!(
        matches!(borrow.target.kind(), ty::Ref(_, target, hir::Mutability::Not) if *target == self_ty)
    );
    assert!(checked.expr_adjustments(expression).is_empty());
    assert_eq!(checked.expr_ty(expression), self_ty);
}
