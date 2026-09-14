//! Authenticate one concrete standard constructor using compiler identities.
use crate::source_capabilities::Capability;
use rustc_abi::ExternAbi;
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::{self, GenericArgsRef, Ty, TyCtxt};
use rustc_span::sym;

pub(crate) struct OwnedBoxConstruction;
impl Capability for OwnedBoxConstruction {
    type Input<'tcx> = BoxConstructionInput<'tcx>;
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConstructionError {
    WrongOwner,
    NonCanonicalNode,
    NotDirectCall,
    NotStandardConstructor,
    UnsupportedPayload,
    SignatureMismatch,
}

/// Operation identity only: callers must already be inside successful analysis.
/// This is not a standalone source-analysis or whole-body ownership certificate.
pub(crate) struct BoxConstructionInput<'tcx> {
    owner: LocalDefId,
    call: &'tcx hir::Expr<'tcx>,
    argument: &'tcx hir::Expr<'tcx>,
    constructor: DefId,
    arguments: GenericArgsRef<'tcx>,
    result: Ty<'tcx>,
}

impl<'tcx> BoxConstructionInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        owner: LocalDefId,
        call: &hir::Expr<'tcx>,
    ) -> Result<Self, ConstructionError> {
        if call.hir_id.owner.def_id != owner {
            return Err(ConstructionError::WrongOwner);
        }
        let hir::Node::Expr(canonical) = tcx.hir_node(call.hir_id) else {
            return Err(ConstructionError::NonCanonicalNode);
        };
        if !std::ptr::eq(canonical, call) {
            return Err(ConstructionError::NonCanonicalNode);
        }
        let call = canonical;
        let hir::ExprKind::Call(callee, [argument]) = call.kind else {
            return Err(ConstructionError::NotDirectCall);
        };
        let checked = tcx.typeck(owner);
        let ty::FnDef(constructor, arguments) = *checked.expr_ty(callee).kind() else {
            return Err(ConstructionError::NotDirectCall);
        };
        if Some(constructor) != tcx.get_diagnostic_item(sym::box_new) {
            return Err(ConstructionError::NotStandardConstructor);
        }
        let hir::ExprKind::Path(path) = &callee.kind else {
            return Err(ConstructionError::NotDirectCall);
        };
        if !matches!(checked.qpath_res(path, callee.hir_id), Res::Def(DefKind::AssocFn, resolved) if resolved == constructor)
        {
            return Err(ConstructionError::NotDirectCall);
        }
        if checked.expr_ty(argument) != tcx.types.i32 {
            return Err(ConstructionError::UnsupportedPayload);
        }
        let result = checked.expr_ty(call);
        let signature = tcx
            .fn_sig(constructor)
            .instantiate(tcx, arguments)
            .skip_binder();
        if signature.abi() != ExternAbi::Rust
            || !signature.safety().is_safe()
            || signature.c_variadic()
            || signature.inputs() != [tcx.types.i32]
            || signature.output() != result
            || !checked.expr_adjustments(argument).is_empty()
        {
            return Err(ConstructionError::SignatureMismatch);
        }
        let ty::Adt(def, result_arguments) = result.kind() else {
            return Err(ConstructionError::SignatureMismatch);
        };
        if Some(def.did()) != tcx.lang_items().owned_box()
            || result_arguments.type_at(0) != tcx.types.i32
        {
            return Err(ConstructionError::SignatureMismatch);
        }
        Ok(Self {
            owner,
            call,
            argument,
            constructor,
            arguments,
            result,
        })
    }

    pub(crate) fn owner(&self) -> LocalDefId {
        self.owner
    }
    pub(crate) fn call(&self) -> &'tcx hir::Expr<'tcx> {
        self.call
    }
    pub(crate) fn argument(&self) -> &'tcx hir::Expr<'tcx> {
        self.argument
    }
    pub(crate) fn constructor(&self) -> DefId {
        self.constructor
    }
    pub(crate) fn arguments(&self) -> GenericArgsRef<'tcx> {
        self.arguments
    }
    pub(crate) fn result(&self) -> Ty<'tcx> {
        self.result
    }
}
