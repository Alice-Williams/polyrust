//! Authenticate the original core operation, both operands and their exact width.
use super::Capability;
use rustc_abi::ExternAbi;
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::DefId,
};
use rustc_middle::ty::{self, TyCtxt, TypeckResults};

pub(crate) struct WrappingAddition;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AdditionWidth {
    I32,
    I64,
}

#[cfg(addition_ast_probe)]
#[path = "../../test/addition_input_probe.rs"]
mod probe;

pub(crate) struct AdditionInput<'tcx> {
    source: &'tcx hir::Expr<'tcx>,
    left: &'tcx hir::Expr<'tcx>,
    right: &'tcx hir::Expr<'tcx>,
    definition: DefId,
    width: AdditionWidth,
}
impl Capability for WrappingAddition {
    type Input<'tcx> = AdditionInput<'tcx>;
}

impl<'tcx> AdditionInput<'tcx> {
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
            return Err("wrapping addition requires a canonical HIR expression".into());
        };
        if !std::ptr::eq(canonical, expression)
            || !std::ptr::eq(checked, tcx.typeck(expression.hir_id.owner.def_id))
        {
            return Err(
                "wrapping addition requires its original checked function and HIR node".into(),
            );
        }
        let (definition, arguments, left, right) = match canonical.kind {
            hir::ExprKind::MethodCall(_, left, [right], _) => (
                checked
                    .type_dependent_def_id(canonical.hir_id)
                    .ok_or("wrapping addition method is unresolved")?,
                checked.node_args(canonical.hir_id),
                left,
                right,
            ),
            hir::ExprKind::Call(callee, [left, right]) => {
                let hir::ExprKind::Path(path) = &callee.kind else {
                    return Ok(None);
                };
                let Res::Def(DefKind::AssocFn, resolved) = checked.qpath_res(path, callee.hir_id)
                else {
                    return Ok(None);
                };
                let ty::FnDef(typed, arguments) = *checked.expr_ty(callee).kind() else {
                    return Err("wrapping addition requires a direct associated function".into());
                };
                if typed != resolved || !checked.expr_adjustments(callee).is_empty() {
                    return Err(
                        "wrapping addition associated identity or adjustment differs".into(),
                    );
                }
                (resolved, arguments, left, right)
            }
            _ => return Ok(None),
        };
        // Exclusion filter only. A spelling never grants builtin authority.
        if tcx.item_name(definition).as_str() != "wrapping_add" {
            return Ok(None);
        }
        let core = tcx
            .lang_items()
            .copy_trait()
            .filter(|item| !item.is_local())
            .ok_or("wrapping addition requires the standard core language items")?
            .krate;
        if definition.is_local()
            || definition.krate != core
            || tcx.def_kind(definition) != DefKind::AssocFn
        {
            return Err("only the standard primitive wrapping_add method is supported".into());
        }
        let implementation = tcx
            .inherent_impl_of_assoc(definition)
            .ok_or("wrapping addition requires an inherent primitive implementation")?;
        if !arguments.is_empty()
            || tcx.generics_of(definition).count() != 0
            || tcx.generics_of(implementation).count() != 0
            || [canonical, left, right]
                .into_iter()
                .any(|node| !checked.expr_adjustments(node).is_empty())
        {
            return Err(
                "wrapping addition requires nongeneric unadjusted by-value operands".into(),
            );
        }
        let owner = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.type_of(implementation).instantiate_identity(),
            )
            .map_err(|_| "wrapping addition primitive owner normalization failed")?;
        let width = match owner.kind() {
            ty::Int(ty::IntTy::I32) => AdditionWidth::I32,
            ty::Int(ty::IntTy::I64) => AdditionWidth::I64,
            _ => return Err("wrapping addition supports only exact i32 and i64".into()),
        };
        if [canonical, left, right]
            .into_iter()
            .any(|node| checked.expr_ty(node) != owner)
        {
            return Err("wrapping addition operand/result widths differ".into());
        }
        let signature = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.fn_sig(definition).instantiate_identity(),
            )
            .map_err(|_| "wrapping addition signature normalization failed")?;
        if !signature.bound_vars().is_empty() {
            return Err("wrapping addition cannot instantiate bound signature variables".into());
        }
        let signature = signature.skip_binder();
        if signature.abi() != ExternAbi::Rust
            || !signature.safety().is_safe()
            || signature.c_variadic()
            || signature.inputs() != [owner, owner]
            || signature.output() != owner
        {
            return Err("wrapping addition requires the exact safe primitive signature".into());
        }
        Ok(Some(Self {
            source: canonical,
            left,
            right,
            definition,
            width,
        }))
    }
    pub(crate) fn require_context(
        &self,
        tcx: TyCtxt<'tcx>,
        checked: &TypeckResults<'tcx>,
    ) -> Result<(), String> {
        let current = Self::discover(tcx, checked, self.source)?
            .ok_or("wrapping addition witness no longer names a built-in operation")?;
        if current.definition != self.definition
            || current.width != self.width
            || !std::ptr::eq(current.left, self.left)
            || !std::ptr::eq(current.right, self.right)
        {
            return Err("wrapping addition witness identity differs".into());
        }
        Ok(())
    }
    pub(crate) fn left(&self) -> &'tcx hir::Expr<'tcx> {
        self.left
    }
    pub(crate) fn right(&self) -> &'tcx hir::Expr<'tcx> {
        self.right
    }
    pub(crate) fn width(&self) -> AdditionWidth {
        self.width
    }
}
