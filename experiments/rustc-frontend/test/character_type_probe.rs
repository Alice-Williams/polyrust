//! Test-only corruption at compiler, target and foreign-call type boundaries.
use portable_codegen::{
    RustDeclarationId, RustFunctionTypes, RustResultKind, RustScalarKind, RustSourceTypes,
};
use std::collections::BTreeMap;

#[path = "character_constant_type_probe.rs"]
mod constants;

pub(crate) fn inspect(original: RustSourceTypes) -> RustSourceTypes {
    if std::env::var_os("POLYRUST_CHARACTER_CONSTANT_FAULT").is_some() {
        return constants::change(original, false);
    }
    if !original.constants().is_empty() || !original.contains_char() {
        return original;
    }
    assert!(original.contains_char());
    eprintln!(
        "CHAR_SOURCE_TYPES\t{:016x}\t{}\t{}",
        original.root().crate_id,
        original.functions().len(),
        original.fields().len()
    );
    let mut root = original.root();
    let mut functions = original.functions().clone();
    let mut fields = original.fields().clone();
    match std::env::var("POLYRUST_CHARACTER_FAULT").as_deref() {
        Err(std::env::VarError::NotPresent) => return original,
        Ok("target_field_owner" | "target_function_kind") => return original,
        Ok("function_kind") => integer_signatures(&mut functions),
        Ok("field_kind") => {
            assert!(
                fields
                    .values()
                    .any(|field| field.kind == RustScalarKind::Char)
            );
            for field in fields.values_mut() {
                if field.kind == RustScalarKind::Char {
                    field.kind = RustScalarKind::I32;
                }
            }
        }
        Ok("field_owner") => return swapped_field_owners(original),
        Ok("owner") => root.definition_path_hash ^= 1,
        Ok("declaration") => {
            let (mut id, signature) = functions.pop_first().unwrap();
            id.definition_path_hash = u64::MAX;
            assert!(functions.insert(id, signature).is_none());
        }
        _ => panic!("unknown character source fault"),
    }
    let changed = RustSourceTypes::new(root, functions, fields).unwrap();
    assert_ne!(changed, original);
    changed
}

fn integer_signatures(functions: &mut BTreeMap<RustDeclarationId, RustFunctionTypes>) {
    for signature in functions.values_mut() {
        for kind in &mut signature.parameters {
            if *kind == RustScalarKind::Char {
                *kind = RustScalarKind::I32;
            }
        }
        if signature.result == RustResultKind::Scalar(RustScalarKind::Char) {
            signature.result = RustResultKind::Scalar(RustScalarKind::I32);
        }
    }
}

fn swapped_field_owners(original: RustSourceTypes) -> RustSourceTypes {
    let mut fields = original.fields().clone();
    let selected: Vec<_> = fields
        .iter()
        .filter(|(_, field)| field.kind == RustScalarKind::Char)
        .map(|(id, field)| (*id, field.owner))
        .collect();
    assert_eq!(selected.len(), 2);
    assert_ne!(selected[0].1, selected[1].1);
    fields.get_mut(&selected[0].0).unwrap().owner = selected[1].1;
    fields.get_mut(&selected[1].0).unwrap().owner = selected[0].1;
    RustSourceTypes::new(original.root(), original.functions().clone(), fields).unwrap()
}

/// Canonical source authentication has already succeeded. Wrong field owners
/// must fail target reconciliation; Java's Int-compatible function type fault
/// must pass target representation checks and fail the consumer's source join.
pub(crate) fn after_authentication(original: RustSourceTypes) -> RustSourceTypes {
    if std::env::var_os("POLYRUST_CHARACTER_CONSTANT_FAULT").is_some() {
        return constants::change(original, true);
    }
    match std::env::var("POLYRUST_CHARACTER_FAULT").as_deref() {
        Ok("target_field_owner") => swapped_field_owners(original),
        Ok("target_function_kind") => {
            let mut functions = original.functions().clone();
            integer_signatures(&mut functions);
            RustSourceTypes::new(original.root(), functions, original.fields().clone()).unwrap()
        }
        _ => original,
    }
}

#[cfg(not(java_graph))]
pub(crate) fn call_argument<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    original: rustc_middle::ty::Ty<'tcx>,
) -> rustc_middle::ty::Ty<'tcx> {
    if original.is_char()
        && std::env::var("POLYRUST_CHARACTER_FAULT").as_deref() == Ok("call_argument")
    {
        tcx.types.i32
    } else {
        original
    }
}

#[cfg(not(java_graph))]
pub(crate) fn call_result<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    original: rustc_middle::ty::Ty<'tcx>,
) -> rustc_middle::ty::Ty<'tcx> {
    if original.is_char()
        && std::env::var("POLYRUST_CHARACTER_FAULT").as_deref() == Ok("call_result")
    {
        tcx.types.i32
    } else {
        original
    }
}
