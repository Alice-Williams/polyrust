//! Private witness invariants are observed, never forged or repaired.
use super::*;
impl<'tcx> NaNInput<'tcx> {
    pub(crate) fn probe(&self, tcx: TyCtxt<'tcx>, checked: &TypeckResults<'tcx>) {
        for expression in [self.source, self.receiver] {
            assert!(
                matches!(tcx.hir_node(expression.hir_id), hir::Node::Expr(node) if std::ptr::eq(node, expression))
            );
            assert!(checked.expr_adjustments(expression).is_empty());
        }
        let (definition, receiver, form) = match self.source.kind {
            hir::ExprKind::MethodCall(_, receiver, [], _) => (
                checked.type_dependent_def_id(self.source.hir_id).unwrap(),
                receiver,
                "method",
            ),
            hir::ExprKind::Call(callee, [receiver]) => {
                let hir::ExprKind::Path(path) = &callee.kind else {
                    panic!("associated path")
                };
                let Res::Def(DefKind::AssocFn, definition) = checked.qpath_res(path, callee.hir_id)
                else {
                    panic!("associated identity")
                };
                assert!(checked.expr_adjustments(callee).is_empty());
                (definition, receiver, "associated")
            }
            _ => panic!("primitive syntax"),
        };
        assert_eq!(definition, self.definition);
        assert!(std::ptr::eq(receiver, self.receiver));
        assert!(!definition.is_local());
        assert_eq!(
            definition.krate,
            tcx.lang_items().copy_trait().unwrap().krate
        );
        assert_eq!(tcx.item_name(definition).as_str(), "is_nan");
        let implementation = tcx.inherent_impl_of_assoc(definition).unwrap();
        let owner = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.type_of(implementation).instantiate_identity(),
            )
            .unwrap();
        assert!(matches!(owner.kind(), ty::Float(ty::FloatTy::F64)));
        assert_eq!(checked.expr_ty(self.receiver), owner);
        assert_eq!(checked.expr_ty(self.source), tcx.types.bool);
        let signature = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.fn_sig(definition).instantiate_identity(),
            )
            .unwrap()
            .skip_binder();
        assert_eq!(signature.inputs(), [owner]);
        assert_eq!(signature.output(), tcx.types.bool);
        assert!(signature.safety().is_safe() && !signature.c_variadic());
        assert_eq!(signature.abi(), ExternAbi::Rust);
        assert_eq!(tcx.generics_of(definition).count(), 0);
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
        self.require_context(tcx, checked).unwrap();
        eprintln!("NAN_INPUT\t{form}\tF64");
    }
}
