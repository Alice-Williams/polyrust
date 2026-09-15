//! Typed executable local-call registration, preserving the required base slot.
use crate::{
    owned_source::{
        BoxConstructionInput, Builder, OwnedBoxConstruction,
        local_call::{LocalCallInput, OwnedLocalCall},
    },
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
pub(super) struct Context<'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub calls: usize,
    pub boxes: usize,
}
#[derive(Clone, Copy)]
pub(super) struct Observe;
impl Mapping for Observe {
    type Capability = OwnedLocalCall;
    type Context<'tcx> = Context<'tcx>;
    type Output = DefId;
    fn lower<'tcx>(
        &self,
        context: &mut Context<'tcx>,
        input: LocalCallInput<'tcx>,
    ) -> Result<DefId, String> {
        let checked = context.tcx.typeck(input.owner());
        assert_eq!(
            checked.expr_ty(input.argument()),
            input.signature().inputs()[0]
        );
        assert_eq!(checked.expr_ty(input.call()), input.signature().output());
        context.calls += 1;
        Ok(input.definition())
    }
}
#[derive(Clone, Copy)]
pub(super) struct BoxMapping;
impl Mapping for BoxMapping {
    type Capability = OwnedBoxConstruction;
    type Context<'tcx> = Context<'tcx>;
    type Output = DefId;
    fn lower<'tcx>(
        &self,
        context: &mut Context<'tcx>,
        input: BoxConstructionInput<'tcx>,
    ) -> Result<DefId, String> {
        assert_eq!(
            context.tcx.typeck(input.owner()).expr_ty(input.argument()),
            context.tcx.types.i32
        );
        assert_eq!(
            context.tcx.typeck(input.owner()).expr_ty(input.call()),
            input.result()
        );
        assert_eq!(
            context
                .tcx
                .fn_sig(input.constructor())
                .instantiate(context.tcx, input.arguments())
                .skip_binder()
                .output(),
            input.result()
        );
        context.boxes += 1;
        Ok(input.constructor())
    }
}
pub(super) fn bindings<M>(
    mapping: M,
) -> impl Supports<OwnedLocalCall, Mapping = M> + Supports<OwnedBoxConstruction, Mapping = BoxMapping>
where
    M: for<'tcx> Mapping<
            Capability = OwnedLocalCall,
            Context<'tcx> = Context<'tcx>,
            Output = DefId,
        >,
{
    Builder::new()
        .local_call(mapping)
        .construction(BoxMapping)
        .build()
}
