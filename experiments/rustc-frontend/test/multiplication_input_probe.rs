//! Canonical identity, both operands, exact builtin signature and hostile copies.
use super::*;
impl<'tcx> MultiplicationInput<'tcx> {
    pub(crate) fn probe(
        &self,
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
    ) -> &'tcx hir::Expr<'tcx> {
        let expected_ty = match self.width {
            MultiplicationWidth::I32 => tcx.types.i32,
            MultiplicationWidth::I64 => tcx.types.i64,
        };
        for expression in [self.source, self.left, self.right] {
            assert!(
                matches!(tcx.hir_node(expression.hir_id), hir::Node::Expr(node) if std::ptr::eq(node, expression))
            );
            assert!(checked.expr_adjustments(expression).is_empty());
            assert_eq!(checked.expr_ty(expression), expected_ty);
        }
        let (definition, left, right, form) = match self.source.kind {
            hir::ExprKind::MethodCall(_, left, [right], _) => (
                checked.type_dependent_def_id(self.source.hir_id).unwrap(),
                left,
                right,
                "method",
            ),
            hir::ExprKind::Call(callee, [left, right]) => {
                let hir::ExprKind::Path(path) = &callee.kind else {
                    panic!("associated path")
                };
                let Res::Def(DefKind::AssocFn, definition) = checked.qpath_res(path, callee.hir_id)
                else {
                    panic!("associated identity")
                };
                assert!(checked.expr_adjustments(callee).is_empty());
                (definition, left, right, "associated")
            }
            _ => panic!("original primitive syntax"),
        };
        assert_eq!(definition, self.definition);
        assert!(std::ptr::eq(left, self.left) && std::ptr::eq(right, self.right));
        assert!(!definition.is_local());
        assert_eq!(
            definition.krate,
            tcx.lang_items().copy_trait().unwrap().krate
        );
        assert_eq!(tcx.item_name(definition).as_str(), "wrapping_mul");
        let implementation = tcx.inherent_impl_of_assoc(definition).unwrap();
        assert_eq!(
            tcx.try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.type_of(implementation).instantiate_identity(),
            )
            .unwrap(),
            expected_ty
        );
        assert_eq!(tcx.generics_of(definition).count(), 0);
        assert_eq!(tcx.generics_of(implementation).count(), 0);
        let signature = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.fn_sig(definition).instantiate_identity(),
            )
            .unwrap();
        assert!(signature.bound_vars().is_empty());
        let signature = signature.skip_binder();
        assert_eq!(signature.inputs(), [expected_ty, expected_ty]);
        assert_eq!(signature.output(), expected_ty);
        assert_eq!(signature.abi(), ExternAbi::Rust);
        assert!(signature.safety().is_safe() && !signature.c_variadic());
        let copied = hir::Expr {
            hir_id: self.source.hir_id,
            kind: self.source.kind,
            span: self.source.span,
        };
        assert!(Self::discover(tcx, checked, &copied).is_err());
        let other = tcx
            .hir_body_owners()
            .find(|id| *id != self.source.hir_id.owner.def_id && tcx.def_kind(*id) == DefKind::Fn)
            .unwrap();
        assert!(Self::discover(tcx, tcx.typeck(other), self.source).is_err());
        assert!(self.require_context(tcx, tcx.typeck(other)).is_err());
        let swapped = Self {
            source: self.source,
            left: self.right,
            right: self.left,
            definition,
            width: self.width,
        };
        assert!(swapped.require_context(tcx, checked).is_err());
        let wrong_width = Self {
            source: self.source,
            left,
            right,
            definition,
            width: match self.width {
                MultiplicationWidth::I32 => MultiplicationWidth::I64,
                MultiplicationWidth::I64 => MultiplicationWidth::I32,
            },
        };
        assert!(wrong_width.require_context(tcx, checked).is_err());
        self.require_context(tcx, checked).unwrap();
        eprintln!("MULTIPLICATION_INPUT\t{form}\t{:?}", self.width);
        self.source
    }
}
