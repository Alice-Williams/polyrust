//! Exercise the production field predicate with only allocator identity changed.
//! Stable source cannot spell Box<T, A>; this is a compiler-type oracle, not
//! a claim that an allocator-api Rust program passed source admission.
use super::{RecordError, check_field};
use rustc_hir::def::DefKind;
use rustc_middle::ty::{self, Ty, TyCtxt};

pub(crate) fn check(tcx: TyCtxt<'_>) {
    let owner = tcx
        .hir_body_owners()
        .find(|id| tcx.def_kind(*id) == DefKind::Fn && tcx.def_path_str(*id) == "allocator_type")
        .expect("allocator type fixture");
    let signature = tcx.fn_sig(owner).instantiate_identity().skip_binder();
    assert_eq!(signature.inputs().len(), 1);
    let allocator = signature.inputs()[0];
    let constructor = tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap();
    let standard = tcx
        .fn_sig(constructor)
        .instantiate(tcx, tcx.mk_args(&[tcx.types.i32.into()]))
        .skip_binder()
        .output();
    let &ty::Adt(definition, arguments) = standard.kind() else {
        panic!("standard Box type")
    };
    assert_eq!(Some(definition.did()), tcx.lang_items().owned_box());
    assert_eq!(arguments.len(), 2);
    assert_eq!(arguments.type_at(0), tcx.types.i32);
    assert_ne!(arguments.type_at(1), allocator);
    let changed = Ty::new_adt(
        tcx,
        definition,
        tcx.mk_args(&[arguments[0], allocator.into()]),
    );
    assert_ne!(standard, changed);
    assert_eq!(check_field(standard, standard, standard), Ok(()));
    // Both field and initializer agree: only the allocator differs from Box::new.
    assert_eq!(
        check_field(changed, standard, changed),
        Err(RecordError::UnsupportedField)
    );
    // Independently retain the initializer/declaration equality requirement.
    assert_eq!(
        check_field(standard, standard, changed),
        Err(RecordError::UnsupportedField)
    );
    println!("record allocator identity oracle passed");
}
