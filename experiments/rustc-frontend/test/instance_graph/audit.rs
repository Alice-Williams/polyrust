//! Independent seven-role oracle for the observation experiment's handoff.
//! This does not reuse the witness's stable-ID conversion or root construction.
use portable_codegen::{RustCanonicalInstanceFacts as Facts, RustDeclarationId as Id};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{Ty, TyCtxt};

pub(super) fn verify<'tcx>(tcx: TyCtxt<'tcx>, value: Ty<'tcx>, facts: Facts) -> Result<(), String> {
    let definition = value.ty_adt_def().ok_or("audit requires nominal Result")?;
    let rustc_middle::ty::Adt(_, args) = value.kind() else {
        return Err("audit requires nominal Result arguments".into());
    };
    let error = args
        .type_at(1)
        .ty_adt_def()
        .ok_or("audit requires nominal error")?;
    // Walk actual definition parents, independently of the witness's crate-index
    // root construction. Both original enum and error must reach this root.
    let root = |mut item: DefId| {
        while let Some(parent) = tcx.opt_parent(item) {
            item = parent;
        }
        item
    };
    let core = root(definition.did());
    if root(error.did()) != core {
        return Err("audit error and Result roots differ".into());
    }
    let ok = tcx
        .lang_items()
        .result_ok_variant()
        .ok_or("audit missing Ok")?;
    let err = tcx
        .lang_items()
        .result_err_variant()
        .ok_or("audit missing Err")?;
    let field = |variant| {
        let item = definition
            .variants()
            .iter()
            .find(|item| item.def_id == variant)
            .ok_or("audit variant missing")?;
        if item.fields.len() != 1 {
            return Err("audit payload inventory differs");
        }
        Ok(item.fields.iter().next().unwrap().did)
    };
    let expected = [
        core,
        definition.did(),
        error.did(),
        ok,
        err,
        field(ok)?,
        field(err)?,
    ];
    let actual = [
        facts.core_root(),
        facts.key().result_definition(),
        facts.key().error_definition(),
        facts.ok().variant,
        facts.err().variant,
        facts.ok().payload,
        facts.err().payload,
    ];
    let roles = [
        "CoreRoot",
        "Result",
        "Error",
        "Ok",
        "Err",
        "OkPayload",
        "ErrPayload",
    ];
    for ((expected, actual), role) in expected.into_iter().zip(actual).zip(roles) {
        let hash = tcx.def_path_hash(expected);
        let original = Id {
            crate_id: hash.stable_crate_id().as_u64(),
            definition_path_hash: hash.local_hash().as_u64(),
        };
        if actual != original {
            return Err(format!("compiler instance role mismatch: {role}"));
        }
    }
    Ok(())
}
