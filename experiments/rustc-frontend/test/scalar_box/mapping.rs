//! Every support slot stores an executable mapping with a fixed consumer signature.
use super::previous::{BoxMapping, RecordMapping};
use crate::{
    owned_source::{
        Builder, OwnedBoxConstruction,
        boxed_record::{OwnedScalarRecordBoxConstruction, ScalarRecordBoxInput},
        record::OwnedRecordConstruction,
    },
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;

pub(super) struct Context<'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub calls: usize,
}
#[derive(Clone, Copy)]
pub(super) struct Observe;
impl Mapping for Observe {
    type Capability = OwnedScalarRecordBoxConstruction;
    type Context<'tcx> = Context<'tcx>;
    type Output = DefId;
    fn lower<'tcx>(
        &self,
        context: &mut Context<'tcx>,
        input: ScalarRecordBoxInput<'tcx>,
    ) -> Result<DefId, String> {
        assert_eq!(input.owner(), input.call().hir_id.owner.def_id);
        assert_eq!(
            context.tcx.typeck(input.owner()).expr_ty(input.argument()),
            input.payload().ty()
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
        context.calls += 1;
        Ok(input.payload().definition().did())
    }
}
pub(super) fn bindings<M>(
    mapping: M,
) -> impl Supports<OwnedScalarRecordBoxConstruction, Mapping = M>
+ Supports<OwnedBoxConstruction, Mapping = BoxMapping>
+ Supports<OwnedRecordConstruction, Mapping = RecordMapping>
where
    M: for<'tcx> Mapping<
            Capability = OwnedScalarRecordBoxConstruction,
            Context<'tcx> = Context<'tcx>,
            Output = DefId,
        >,
{
    Builder::new()
        .scalar_record_box(mapping)
        .construction(BoxMapping)
        .record_construction(RecordMapping)
        .build()
}
