//! The proof consumer fixes executable signatures, not independent support flags.
use crate::{
    owned_source::{
        BoxConstructionInput, Builder, OwnedBoxConstruction,
        record::{OwnedRecordConstruction, RecordConstructionInput},
    },
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;

pub(super) struct Context<'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub boxes: usize,
    pub records: usize,
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
        assert_eq!(input.owner(), input.call().hir_id.owner.def_id);
        assert_eq!(
            context
                .tcx
                .fn_sig(input.constructor())
                .instantiate(context.tcx, input.arguments())
                .skip_binder()
                .output(),
            input.result()
        );
        assert_eq!(
            context.tcx.typeck(input.owner()).expr_ty(input.argument()),
            context.tcx.types.i32
        );
        context.boxes += 1;
        Ok(input.constructor())
    }
}
#[derive(Clone, Copy)]
pub(super) struct RecordMapping;
impl Mapping for RecordMapping {
    type Capability = OwnedRecordConstruction;
    type Context<'tcx> = Context<'tcx>;
    type Output = DefId;
    fn lower<'tcx>(
        &self,
        context: &mut Context<'tcx>,
        input: RecordConstructionInput<'tcx>,
    ) -> Result<DefId, String> {
        assert_eq!(input.owner(), input.expression().hir_id.owner.def_id);
        assert_eq!(
            context
                .tcx
                .typeck(input.owner())
                .expr_ty(input.expression()),
            input.result()
        );
        assert!(input.arguments().is_empty());
        assert_eq!(
            input.definition().non_enum_variant().fields.len(),
            input.fields().len()
        );
        context.records += 1;
        Ok(input.definition().did())
    }
}
pub(super) fn bindings<M>(
    mapping: M,
) -> impl Supports<OwnedRecordConstruction, Mapping = M>
+ Supports<OwnedBoxConstruction, Mapping = BoxMapping>
where
    M: for<'tcx> Mapping<
            Capability = OwnedRecordConstruction,
            Context<'tcx> = Context<'tcx>,
            Output = DefId,
        >,
{
    Builder::new()
        .construction(BoxMapping)
        .record_construction(mapping)
        .build()
}
