//! Standard scalar Box clone identity, independent of body ownership proof.
use super::local_call::scalar_box_type;
use crate::source_capabilities::Capability;
use rustc_abi::ExternAbi;
use rustc_hir::{
    self as hir,
    def::{DefKind, Res},
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::{
    self, FnSig, GenericArgsRef, Ty, TyCtxt,
    adjustment::{Adjust, AutoBorrow, AutoBorrowMutability, DerefAdjustKind},
};

pub(crate) struct OwnedBoxClone;
impl Capability for OwnedBoxClone {
    type Input<'tcx> = BoxCloneInput<'tcx>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CloneError {
    WrongOwner,
    NonCanonicalNode,
    UnsupportedCall,
    NotStandardClone,
    UnsupportedType,
    UnsupportedReceiver,
    Adjustment,
    Implementation,
}
#[derive(Clone, Copy)]
pub(crate) enum CloneForm<'tcx> {
    Method,
    ExplicitBorrow(&'tcx hir::Expr<'tcx>),
}
pub(crate) struct BoxCloneInput<'tcx> {
    owner: LocalDefId,
    call: &'tcx hir::Expr<'tcx>,
    receiver: &'tcx hir::Expr<'tcx>,
    binding: hir::HirId,
    form: CloneForm<'tcx>,
    definition: DefId,
    arguments: GenericArgsRef<'tcx>,
    signature: FnSig<'tcx>,
    instance: ty::Instance<'tcx>,
    box_ty: Ty<'tcx>,
}
impl<'tcx> BoxCloneInput<'tcx> {
    pub(crate) fn read(
        tcx: TyCtxt<'tcx>,
        owner: LocalDefId,
        call: &hir::Expr<'tcx>,
    ) -> Result<Self, CloneError> {
        if call.hir_id.owner.def_id != owner {
            return Err(CloneError::WrongOwner);
        }
        let hir::Node::Expr(canonical) = tcx.hir_node(call.hir_id) else {
            return Err(CloneError::NonCanonicalNode);
        };
        if !std::ptr::eq(canonical, call) {
            return Err(CloneError::NonCanonicalNode);
        }
        let call = canonical;
        let checked = tcx.typeck(owner);
        let (definition, arguments, receiver, form) = match call.kind {
            hir::ExprKind::MethodCall(_, receiver, [], _) => (
                checked
                    .type_dependent_def_id(call.hir_id)
                    .ok_or(CloneError::UnsupportedCall)?,
                checked.node_args(call.hir_id),
                receiver,
                CloneForm::Method,
            ),
            hir::ExprKind::Call(callee, [borrow]) => {
                let ty::FnDef(def, args) = *checked.expr_ty(callee).kind() else {
                    return Err(CloneError::UnsupportedCall);
                };
                let hir::ExprKind::Path(path) = &callee.kind else {
                    return Err(CloneError::UnsupportedCall);
                };
                if !matches!(checked.qpath_res(path, callee.hir_id), Res::Def(DefKind::AssocFn, actual) if actual == def)
                    || !checked.expr_adjustments(callee).is_empty()
                {
                    return Err(CloneError::UnsupportedCall);
                }
                let hir::ExprKind::AddrOf(hir::BorrowKind::Ref, hir::Mutability::Not, receiver) =
                    borrow.kind
                else {
                    return Err(CloneError::UnsupportedReceiver);
                };
                (def, args, receiver, CloneForm::ExplicitBorrow(borrow))
            }
            _ => return Err(CloneError::UnsupportedCall),
        };
        if Some(definition) != tcx.lang_items().clone_fn()
            || Some(tcx.parent(definition)) != tcx.lang_items().clone_trait()
        {
            return Err(CloneError::NotStandardClone);
        }
        let box_ty = scalar_box_type(tcx).ok_or(CloneError::UnsupportedType)?;
        if arguments.len() != 1
            || arguments.type_at(0) != box_ty
            || checked.expr_ty(receiver) != box_ty
            || checked.expr_ty(call) != box_ty
        {
            return Err(CloneError::UnsupportedType);
        }
        let hir::ExprKind::Path(path) = &receiver.kind else {
            return Err(CloneError::UnsupportedReceiver);
        };
        let Res::Local(binding) = checked.qpath_res(path, receiver.hir_id) else {
            return Err(CloneError::UnsupportedReceiver);
        };
        if binding.owner.def_id != owner || receiver.hir_id.owner.def_id != owner {
            return Err(CloneError::WrongOwner);
        }
        let shared = |target: Ty<'tcx>| matches!(target.kind(), ty::Ref(_, inner, hir::Mutability::Not) if *inner == box_ty);
        let borrow_adjust = |kind: &Adjust| {
            matches!(
                kind,
                Adjust::Borrow(AutoBorrow::Ref(AutoBorrowMutability::Not))
            )
        };
        if !checked.expr_adjustments(call).is_empty() {
            return Err(CloneError::Adjustment);
        }
        match form {
            CloneForm::Method => {
                let [borrow] = checked.expr_adjustments(receiver) else {
                    return Err(CloneError::Adjustment);
                };
                if !borrow_adjust(&borrow.kind) || !shared(borrow.target) {
                    return Err(CloneError::Adjustment);
                }
            }
            CloneForm::ExplicitBorrow(borrow) => {
                if !checked.expr_adjustments(receiver).is_empty()
                    || !shared(checked.expr_ty(borrow))
                {
                    return Err(CloneError::Adjustment);
                }
                let [deref, reborrow] = checked.expr_adjustments(borrow) else {
                    return Err(CloneError::Adjustment);
                };
                if !matches!(deref.kind, Adjust::Deref(DerefAdjustKind::Builtin))
                    || deref.target != box_ty
                    || !borrow_adjust(&reborrow.kind)
                    || !shared(reborrow.target)
                {
                    return Err(CloneError::Adjustment);
                }
            }
        }
        let signature = tcx
            .fn_sig(definition)
            .instantiate(tcx, arguments)
            .skip_binder();
        if signature.abi() != ExternAbi::Rust
            || !signature.safety().is_safe()
            || signature.c_variadic()
            || signature.inputs().len() != 1
            || !shared(signature.inputs()[0])
            || signature.output() != box_ty
        {
            return Err(CloneError::UnsupportedType);
        }
        let environment = ty::TypingEnv::post_analysis(tcx, owner);
        let instance = ty::Instance::try_resolve(tcx, environment, definition, arguments)
            .map_err(|_| CloneError::Implementation)?
            .ok_or(CloneError::Implementation)?;
        let ty::InstanceKind::Item(item) = instance.def else {
            return Err(CloneError::Implementation);
        };
        let box_def = tcx
            .lang_items()
            .owned_box()
            .ok_or(CloneError::Implementation)?;
        if item.is_local()
            || item.krate != box_def.krate
            || tcx.associated_item(item).trait_item_def_id() != Some(definition)
        {
            return Err(CloneError::Implementation);
        }
        let implementation = tcx.parent(item);
        let trait_ref = tcx
            .try_normalize_erasing_regions(
                environment,
                tcx.impl_trait_ref(implementation)
                    .instantiate(tcx, instance.args),
            )
            .map_err(|_| CloneError::Implementation)?;
        if Some(trait_ref.def_id) != tcx.lang_items().clone_trait() || trait_ref.self_ty() != box_ty
        {
            return Err(CloneError::Implementation);
        }
        Ok(Self {
            owner,
            call,
            receiver,
            binding,
            form,
            definition,
            arguments,
            signature,
            instance,
            box_ty,
        })
    }
    pub(crate) fn owner(&self) -> LocalDefId {
        self.owner
    }
    pub(crate) fn call(&self) -> &'tcx hir::Expr<'tcx> {
        self.call
    }
    pub(crate) fn receiver(&self) -> &'tcx hir::Expr<'tcx> {
        self.receiver
    }
    pub(crate) fn binding(&self) -> hir::HirId {
        self.binding
    }
    pub(crate) fn form(&self) -> CloneForm<'tcx> {
        self.form
    }
    pub(crate) fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) fn arguments(&self) -> GenericArgsRef<'tcx> {
        self.arguments
    }
    pub(crate) fn signature(&self) -> FnSig<'tcx> {
        self.signature
    }
    pub(crate) fn instance(&self) -> ty::Instance<'tcx> {
        self.instance
    }
    pub(crate) fn box_ty(&self) -> Ty<'tcx> {
        self.box_ty
    }
}
