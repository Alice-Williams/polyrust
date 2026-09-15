//! Shared identity/signature checks; not itself a public capability input.
use super::ConstructionError as Error;
use rustc_abi::ExternAbi;
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::{self, GenericArgsRef, Ty, TyCtxt};

pub(super) struct Call<'tcx> {
    pub owner: LocalDefId,
    pub call: &'tcx hir::Expr<'tcx>,
    pub argument: &'tcx hir::Expr<'tcx>,
    pub constructor: DefId,
    pub arguments: GenericArgsRef<'tcx>,
    pub result: Ty<'tcx>,
    pub payload: Ty<'tcx>,
}
impl<'tcx> Call<'tcx> {
    pub(super) fn read(
        tcx: TyCtxt<'tcx>,
        owner: LocalDefId,
        call: &hir::Expr<'tcx>,
    ) -> Result<Self, Error> {
        if call.hir_id.owner.def_id != owner {
            return Err(Error::WrongOwner);
        }
        let hir::Node::Expr(canonical) = tcx.hir_node(call.hir_id) else {
            return Err(Error::NonCanonicalNode);
        };
        if !std::ptr::eq(canonical, call) {
            return Err(Error::NonCanonicalNode);
        }
        let call = canonical;
        let hir::ExprKind::Call(callee, [argument]) = call.kind else {
            return Err(Error::NotDirectCall);
        };
        let checked = tcx.typeck(owner);
        let ty::FnDef(constructor, arguments) = *checked.expr_ty(callee).kind() else {
            return Err(Error::NotDirectCall);
        };
        if Some(constructor) != tcx.get_diagnostic_item(rustc_span::sym::box_new) {
            return Err(Error::NotStandardConstructor);
        }
        let hir::ExprKind::Path(path) = &callee.kind else {
            return Err(Error::NotDirectCall);
        };
        if !matches!(checked.qpath_res(path, callee.hir_id), Res::Def(DefKind::AssocFn, def) if def == constructor)
        {
            return Err(Error::NotDirectCall);
        }
        let payload = checked.expr_ty(argument);
        let result = checked.expr_ty(call);
        let signature = tcx
            .fn_sig(constructor)
            .instantiate(tcx, arguments)
            .skip_binder();
        if signature.abi() != ExternAbi::Rust
            || !signature.safety().is_safe()
            || signature.c_variadic()
            || signature.inputs() != [payload]
            || signature.output() != result
            || !checked.expr_adjustments(argument).is_empty()
        {
            return Err(Error::SignatureMismatch);
        }
        let ty::Adt(definition, result_arguments) = result.kind() else {
            return Err(Error::SignatureMismatch);
        };
        if Some(definition.did()) != tcx.lang_items().owned_box()
            || result_arguments.type_at(0) != payload
        {
            return Err(Error::SignatureMismatch);
        }
        Ok(Self {
            owner,
            call,
            argument,
            constructor,
            arguments,
            result,
            payload,
        })
    }
}
