//! Executable typed consumer, including a recursive layout walk.
use crate::{
    owned_source::{
        BoxConstructionInput, Builder, OwnedBoxConstruction,
        nested_record::{
            FieldKind, NestedRecordConstructionInput, OwnedNestedRecordConstruction, RecordLayout,
        },
    },
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{self, Ty, TyCtxt};

pub(super) struct Context<'tcx> {
    pub tcx: TyCtxt<'tcx>,
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
        assert_eq!(
            context.tcx.typeck(input.owner()).expr_ty(input.call()),
            input.result()
        );
        assert_eq!(
            context.tcx.typeck(input.owner()).expr_ty(input.argument()),
            context.tcx.types.i32
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
        Ok(input.constructor())
    }
}
#[derive(Clone, Copy)]
pub(super) struct RecordMapping;
impl Mapping for RecordMapping {
    type Capability = OwnedNestedRecordConstruction;
    type Context<'tcx> = Context<'tcx>;
    type Output = DefId;
    fn lower<'tcx>(
        &self,
        context: &mut Context<'tcx>,
        input: NestedRecordConstructionInput<'tcx>,
    ) -> Result<DefId, String> {
        assert_eq!(input.owner(), input.expression().hir_id.owner.def_id);
        assert_eq!(
            context
                .tcx
                .typeck(input.owner())
                .expr_ty(input.expression()),
            input.result()
        );
        let constructor = context
            .tcx
            .get_diagnostic_item(rustc_span::sym::box_new)
            .unwrap();
        let standard = context
            .tcx
            .fn_sig(constructor)
            .instantiate(
                context.tcx,
                context.tcx.mk_args(&[context.tcx.types.i32.into()]),
            )
            .skip_binder()
            .output();
        inspect(context.tcx, input.layout(), standard);
        context.records += 1;
        Ok(input.layout().definition().did())
    }
}
/// Returns (maximum record depth, expanded field occurrences).
pub(super) fn inspect<'tcx>(
    tcx: TyCtxt<'tcx>,
    layout: &RecordLayout<'tcx>,
    standard: Ty<'tcx>,
) -> (usize, usize) {
    assert!(layout.arguments().is_empty());
    assert_eq!(
        layout.ty(),
        Ty::new_adt(tcx, layout.definition(), layout.arguments())
    );
    assert_eq!(
        layout.fields().len(),
        layout.definition().non_enum_variant().fields.len()
    );
    let mut depth = 1;
    let mut count = layout.fields().len();
    for (index, field) in layout.fields().iter().enumerate() {
        assert_eq!(field.index().as_usize(), index);
        let declaration = &layout.definition().non_enum_variant().fields[field.index()];
        assert_eq!(field.declaration(), declaration.did);
        assert_eq!(
            field.ty(),
            tcx.try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                declaration.ty(tcx, layout.arguments())
            )
            .unwrap()
        );
        match field.kind() {
            FieldKind::Box(leaf) => assert_eq!(*leaf, standard),
            FieldKind::Record(nested) => {
                assert_eq!(nested.ty(), field.ty());
                let (child_depth, child_count) = inspect(tcx, nested, standard);
                depth = depth.max(child_depth + 1);
                count += child_count;
            }
        }
    }
    (depth, count)
}
pub(super) fn bindings<M>(
    mapping: M,
) -> impl Supports<OwnedNestedRecordConstruction, Mapping = M>
+ Supports<OwnedBoxConstruction, Mapping = BoxMapping>
where
    M: for<'tcx> Mapping<
            Capability = OwnedNestedRecordConstruction,
            Context<'tcx> = Context<'tcx>,
            Output = DefId,
        >,
{
    Builder::new()
        .construction(BoxMapping)
        .nested_record(mapping)
        .build()
}
