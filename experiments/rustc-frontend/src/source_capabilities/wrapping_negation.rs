//! Canonical compiler identity for one total built-in operation, not name dispatch.
use super::Capability;
use rustc_abi::ExternAbi;
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::DefId,
};
use rustc_middle::ty::{self, TyCtxt, TypeckResults};

pub(crate) struct WrappingNegation;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WrappingWidth {
    I32,
    I64,
}

pub(crate) struct WrappingInput<'tcx> {
    source: &'tcx hir::Expr<'tcx>,
    receiver: &'tcx hir::Expr<'tcx>,
    definition: DefId,
    width: WrappingWidth,
}
impl Capability for WrappingNegation {
    type Input<'tcx> = WrappingInput<'tcx>;
}
impl<'tcx> WrappingInput<'tcx> {
    /// Ordinary calls retain their existing mapping. Associated/method calls
    /// are admitted here only after proving the actual core primitive identity.
    pub(crate) fn discover(
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
        expression: &hir::Expr<'tcx>,
    ) -> Result<Option<Self>, String> {
        if !matches!(
            expression.kind,
            hir::ExprKind::Call(..) | hir::ExprKind::MethodCall(..)
        ) {
            return Ok(None);
        }
        let hir::Node::Expr(canonical) = tcx.hir_node(expression.hir_id) else {
            return Err("wrapping negation requires a canonical HIR expression".into());
        };
        if !std::ptr::eq(canonical, expression)
            || !std::ptr::eq(checked, tcx.typeck(expression.hir_id.owner.def_id))
        {
            return Err(
                "wrapping negation requires its original checked function and HIR node".into(),
            );
        }
        let expression = canonical;
        let (definition, arguments, receiver, extra_arguments) = match expression.kind {
            hir::ExprKind::MethodCall(_, receiver, values, _) => (
                checked
                    .type_dependent_def_id(expression.hir_id)
                    .ok_or("wrapping negation method is unresolved")?,
                checked.node_args(expression.hir_id),
                receiver,
                !values.is_empty(),
            ),
            hir::ExprKind::Call(callee, [receiver]) => {
                let hir::ExprKind::Path(path) = &callee.kind else {
                    return Ok(None);
                };
                let Res::Def(DefKind::AssocFn, resolved) = checked.qpath_res(path, callee.hir_id)
                else {
                    return Ok(None);
                };
                let ty::FnDef(typed, arguments) = *checked.expr_ty(callee).kind() else {
                    return Err("wrapping negation requires a direct associated function".into());
                };
                if typed != resolved || !checked.expr_adjustments(callee).is_empty() {
                    return Err(
                        "wrapping negation associated identity or adjustment differs".into(),
                    );
                }
                (resolved, arguments, receiver, false)
            }
            _ => return Ok(None),
        };
        // Leave unrelated calls to their existing boundary, including its
        // established unsupported-operation diagnostic. This name filter is
        // exclusion only; admission below still proves primitive identity.
        if tcx.item_name(definition).as_str() != "wrapping_neg" {
            return Ok(None);
        }
        if extra_arguments {
            return Err("wrapping negation takes no extra arguments".into());
        }
        // A language-item DefId, rather than a crate-name string, anchors the
        // trusted core crate supplied by the pinned compiler configuration.
        let core = tcx
            .lang_items()
            .copy_trait()
            .filter(|item| !item.is_local())
            .ok_or("wrapping negation requires the standard core language items")?
            .krate;
        if definition.is_local()
            || definition.krate != core
            || tcx.def_kind(definition) != DefKind::AssocFn
            || tcx.item_name(definition).as_str() != "wrapping_neg"
        {
            return Err("only the standard primitive wrapping_neg method is supported".into());
        }
        let implementation = tcx
            .inherent_impl_of_assoc(definition)
            .ok_or("wrapping negation requires an inherent primitive implementation")?;
        if !arguments.is_empty()
            || tcx.generics_of(definition).count() != 0
            || tcx.generics_of(implementation).count() != 0
            || !checked.expr_adjustments(expression).is_empty()
            || !checked.expr_adjustments(receiver).is_empty()
        {
            return Err(
                "wrapping negation requires nongeneric unadjusted by-value arguments".into(),
            );
        }
        let owner = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.type_of(implementation).instantiate_identity(),
            )
            .map_err(|_| "wrapping negation primitive owner normalization failed")?;
        let width = match owner.kind() {
            ty::Int(ty::IntTy::I32) => WrappingWidth::I32,
            ty::Int(ty::IntTy::I64) => WrappingWidth::I64,
            _ => return Err("wrapping negation supports only exact i32 and i64".into()),
        };
        if checked.expr_ty(receiver) != owner || checked.expr_ty(expression) != owner {
            return Err("wrapping negation receiver/result widths differ".into());
        }
        let signature = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.fn_sig(definition).instantiate_identity(),
            )
            .map_err(|_| "wrapping negation signature normalization failed")?;
        if !signature.bound_vars().is_empty() {
            return Err("wrapping negation cannot instantiate bound signature variables".into());
        }
        let signature = signature.skip_binder();
        if signature.abi() != ExternAbi::Rust
            || !signature.safety().is_safe()
            || signature.c_variadic()
            || signature.inputs() != [owner]
            || signature.output() != owner
        {
            return Err("wrapping negation requires the exact safe primitive signature".into());
        }
        Ok(Some(Self {
            source: canonical,
            receiver,
            definition,
            width,
        }))
    }

    /// Keep compiler witnesses tied to the reader's original function context.
    pub(crate) fn require_context(
        &self,
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
    ) -> Result<(), String> {
        let current = Self::discover(tcx, checked, self.source)?
            .ok_or("wrapping negation witness no longer names a built-in operation")?;
        if current.definition != self.definition
            || current.width != self.width
            || !std::ptr::eq(current.receiver, self.receiver)
        {
            return Err("wrapping negation witness identity differs".into());
        }
        Ok(())
    }
    pub(crate) fn receiver(&self) -> &'tcx hir::Expr<'tcx> {
        self.receiver
    }
    pub(crate) fn width(&self) -> WrappingWidth {
        self.width
    }
}

#[cfg(wrapping_ast_probe)]
#[path = "../../test/wrapping_input_probe.rs"]
mod input_probe;
