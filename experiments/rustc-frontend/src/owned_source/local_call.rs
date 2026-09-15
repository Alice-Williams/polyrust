//! Direct local owned-call identity; role signatures do not prove callee effects.
use crate::source_capabilities::Capability;
use rustc_abi::ExternAbi;
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::{self, FnSig, GenericArgsRef, Ty, TyCtxt};

pub(crate) struct OwnedLocalCall;
impl Capability for OwnedLocalCall {
    type Input<'tcx> = LocalCallInput<'tcx>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallRole {
    Producer,
    Consumer,
    Relay,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallError {
    WrongOwner,
    NonCanonicalNode,
    NotDirectCall,
    NotLocalFunction,
    UnsupportedSignature,
    Adjustment,
    StandardType,
}
pub(crate) struct LocalCallInput<'tcx> {
    owner: LocalDefId,
    call: &'tcx hir::Expr<'tcx>,
    argument: &'tcx hir::Expr<'tcx>,
    callee: LocalDefId,
    arguments: GenericArgsRef<'tcx>,
    signature: FnSig<'tcx>,
    box_ty: Ty<'tcx>,
    role: CallRole,
}
impl<'tcx> LocalCallInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        owner: LocalDefId,
        call: &hir::Expr<'tcx>,
    ) -> Result<Self, CallError> {
        if call.hir_id.owner.def_id != owner {
            return Err(CallError::WrongOwner);
        }
        let hir::Node::Expr(canonical) = tcx.hir_node(call.hir_id) else {
            return Err(CallError::NonCanonicalNode);
        };
        if !std::ptr::eq(canonical, call) {
            return Err(CallError::NonCanonicalNode);
        }
        let call = canonical;
        let hir::ExprKind::Call(callee, [argument]) = call.kind else {
            return Err(CallError::NotDirectCall);
        };
        let checked = tcx.typeck(owner);
        let ty::FnDef(definition, arguments) = *checked.expr_ty(callee).kind() else {
            return Err(CallError::NotDirectCall);
        };
        let hir::ExprKind::Path(path) = &callee.kind else {
            return Err(CallError::NotDirectCall);
        };
        if !matches!(checked.qpath_res(path, callee.hir_id), Res::Def(DefKind::Fn, def) if def == definition)
            || tcx.def_kind(definition) != DefKind::Fn
            || !definition.is_local()
        {
            return Err(CallError::NotLocalFunction);
        }
        if !arguments.is_empty() || tcx.generics_of(definition).count() != 0 {
            return Err(CallError::UnsupportedSignature);
        }
        if !checked.expr_adjustments(callee).is_empty()
            || !checked.expr_adjustments(argument).is_empty()
            || !checked.expr_adjustments(call).is_empty()
        {
            return Err(CallError::Adjustment);
        }
        let signature = tcx
            .fn_sig(definition)
            .instantiate(tcx, arguments)
            .skip_binder();
        if signature.abi() != ExternAbi::Rust
            || !signature.safety().is_safe()
            || signature.c_variadic()
            || signature.inputs() != [checked.expr_ty(argument)]
            || signature.output() != checked.expr_ty(call)
        {
            return Err(CallError::UnsupportedSignature);
        }
        let box_ty = scalar_box_type(tcx).ok_or(CallError::StandardType)?;
        let role = match (signature.inputs()[0], signature.output()) {
            (input, output) if input == tcx.types.i32 && output == box_ty => CallRole::Producer,
            (input, output) if input == box_ty && output == tcx.types.i32 => CallRole::Consumer,
            (input, output) if input == box_ty && output == box_ty => CallRole::Relay,
            _ => return Err(CallError::UnsupportedSignature),
        };
        Ok(Self {
            owner,
            call,
            argument,
            callee: definition.expect_local(),
            arguments,
            signature,
            box_ty,
            role,
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
    pub(crate) fn callee(&self) -> LocalDefId {
        self.callee
    }
    pub(crate) fn definition(&self) -> DefId {
        self.callee.to_def_id()
    }
    pub(crate) fn arguments(&self) -> GenericArgsRef<'tcx> {
        self.arguments
    }
    pub(crate) fn signature(&self) -> FnSig<'tcx> {
        self.signature
    }
    pub(crate) fn box_ty(&self) -> Ty<'tcx> {
        self.box_ty
    }
    pub(crate) fn role(&self) -> CallRole {
        self.role
    }
}

/// Derive the full standard allocator-bearing type from the compiler library.
pub(super) fn scalar_box_type(tcx: TyCtxt<'_>) -> Option<Ty<'_>> {
    let constructor = tcx.get_diagnostic_item(rustc_span::sym::box_new)?;
    if tcx.generics_of(constructor).count() != 1 {
        return None;
    }
    let arguments = tcx.mk_args(&[tcx.types.i32.into()]);
    let signature = tcx
        .fn_sig(constructor)
        .instantiate(tcx, arguments)
        .skip_binder();
    if signature.abi() != ExternAbi::Rust
        || !signature.safety().is_safe()
        || signature.c_variadic()
        || signature.inputs() != [tcx.types.i32]
    {
        return None;
    }
    let result = signature.output();
    let ty::Adt(def, args) = result.kind() else {
        return None;
    };
    (Some(def.did()) == tcx.lang_items().owned_box() && args.type_at(0) == tcx.types.i32)
        .then_some(result)
}
