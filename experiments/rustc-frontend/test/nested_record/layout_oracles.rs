//! Isolated compiler-type controls, not acceptance of unstable allocator source.
use super::{E, FieldKind, Reader};
use rustc_hir::{def::DefKind, def_id::DefId};
use rustc_middle::ty::{self, Ty, TyCtxt};
use std::collections::HashSet;

pub(crate) fn check(tcx: TyCtxt<'_>) {
    let owner = tcx
        .hir_body_owners()
        .find(|id| tcx.def_kind(*id) == DefKind::Fn && tcx.def_path_str(*id) == "allocator_type")
        .unwrap();
    let allocator = tcx
        .fn_sig(owner)
        .instantiate_identity()
        .skip_binder()
        .inputs()[0];
    let ty::Adt(allocator_definition, allocator_arguments) = allocator.kind() else {
        panic!("compiler-resolved System allocator");
    };
    assert!(!allocator_definition.did().is_local());
    assert!(allocator_arguments.is_empty());
    let standard = super::super::super::local_call::scalar_box_type(tcx).unwrap();
    let &ty::Adt(definition, args) = standard.kind() else {
        panic!("Box");
    };
    let bounds = tcx
        .predicates_of(definition.did())
        .instantiate(tcx, tcx.mk_args(&[args[0], allocator.into()]));
    let required: Vec<_> = bounds
        .predicates
        .iter()
        .filter_map(|clause| {
            let ty::ClauseKind::Trait(predicate) = clause.kind().skip_binder() else {
                return None;
            };
            let identity = allocator_bound(tcx, predicate, allocator)?;
            let mut negative = predicate;
            negative.polarity = ty::PredicatePolarity::Negative;
            assert!(allocator_bound(tcx, negative, allocator).is_none());
            Some(identity)
        })
        .collect();
    assert_eq!(
        required.len(),
        1,
        "one allocator bound on Box: {:?}",
        bounds.predicates
    );
    let allocator_trait = required[0];
    assert!(
        tcx.all_impls(allocator_trait).any(|implementation| {
            if tcx.generics_of(implementation).count() != 0 {
                return false;
            }
            let reference = tcx.impl_trait_ref(implementation).instantiate_identity();
            tcx.try_normalize_erasing_regions(ty::TypingEnv::fully_monomorphized(), reference)
                .is_ok_and(|reference| {
                    if reference.def_id == allocator_trait && reference.self_ty() == allocator {
                        assert!(allocator_impl(
                            reference,
                            ty::ImplPolarity::Positive,
                            allocator_trait,
                            allocator
                        ));
                        assert!(!allocator_impl(
                            reference,
                            ty::ImplPolarity::Negative,
                            allocator_trait,
                            allocator
                        ));
                        assert!(!allocator_impl(
                            reference,
                            ty::ImplPolarity::Reservation,
                            allocator_trait,
                            allocator
                        ));
                    }
                    allocator_impl(
                        reference,
                        tcx.impl_polarity(implementation),
                        allocator_trait,
                        allocator,
                    )
                })
        }),
        "alternate allocator must implement the actual Allocator trait"
    );
    assert_eq!(args.len(), 2);
    assert_ne!(args.type_at(1), allocator);
    let changed = Ty::new_adt(tcx, definition, tcx.mk_args(&[args[0], allocator.into()]));
    assert_ne!(standard, changed);
    let mut reader = Reader {
        tcx,
        box_ty: standard,
        ancestors: HashSet::new(),
        remaining: 128,
    };
    assert!(matches!(reader.field_kind(standard, 1), Ok(FieldKind::Box(ty)) if ty == standard));
    assert!(matches!(
        reader.field_kind(changed, 1),
        Err(E::UnsupportedField)
    ));
    let repeated = tcx
        .hir_body_owners()
        .find(|id| tcx.def_kind(*id) == DefKind::Fn && tcx.def_path_str(*id) == "repeated")
        .unwrap();
    let record = tcx
        .fn_sig(repeated)
        .instantiate_identity()
        .skip_binder()
        .output();
    let ty::Adt(record_def, _) = record.kind() else {
        panic!("record");
    };
    reader.ancestors.insert(record_def.did());
    assert!(matches!(reader.record(record, 1), Err(E::RecursiveRecord)));
    println!("nested allocator and ancestor controls passed");
}

fn allocator_bound<'tcx>(
    tcx: TyCtxt<'tcx>,
    predicate: ty::TraitPredicate<'tcx>,
    allocator: Ty<'tcx>,
) -> Option<DefId> {
    let reference = predicate.trait_ref;
    (predicate.polarity == ty::PredicatePolarity::Positive
        && reference.self_ty() == allocator
        && Some(reference.def_id) != tcx.lang_items().sized_trait())
    .then_some(reference.def_id)
}

fn allocator_impl<'tcx>(
    reference: ty::TraitRef<'tcx>,
    polarity: ty::ImplPolarity,
    required: DefId,
    allocator: Ty<'tcx>,
) -> bool {
    polarity == ty::ImplPolarity::Positive
        && reference.def_id == required
        && reference.self_ty() == allocator
}
