//! Independent original-field/variant oracle; does not reuse witness conversion.
use portable_codegen::{
    RustCanonicalErrorKindFacts as Facts, RustDeclarationId as Id, RustIntegerErrorKind as Kind,
};
use rustc_middle::ty::{self, Ty, TyCtxt};

pub(super) fn verify<'tcx>(
    tcx: TyCtxt<'tcx>,
    result: Ty<'tcx>,
    facts: Facts,
) -> Result<(), String> {
    let ty::Adt(_, result_args) = result.kind() else {
        return Err("error audit requires original Result".into());
    };
    let ty::Adt(wrapper, error_args) = result_args.type_at(1).kind() else {
        return Err("error audit requires original error".into());
    };
    let field = wrapper
        .all_fields()
        .next()
        .ok_or("error audit wrapper field missing")?;
    let kind = tcx
        .try_normalize_erasing_regions(
            ty::TypingEnv::fully_monomorphized(),
            field.ty(tcx, error_args),
        )
        .map_err(|_| "error audit normalization failed")?;
    let definition = kind.ty_adt_def().ok_or("error audit enum missing")?;
    let mut expected = vec![
        ("WrapperField", field.did, facts.wrapper_field()),
        ("Kind", definition.did(), facts.kind_definition()),
    ];
    for (role, name) in [
        (Kind::Empty, "Empty"),
        (Kind::InvalidDigit, "InvalidDigit"),
        (Kind::PosOverflow, "PosOverflow"),
        (Kind::NegOverflow, "NegOverflow"),
        (Kind::Zero, "Zero"),
        (Kind::NotAPowerOfTwo, "NotAPowerOfTwo"),
    ] {
        // Find by original named role, independent of witness's ordinal zip.
        let variant = definition
            .variants()
            .iter()
            .find(|item| item.name.as_str() == name)
            .ok_or("error audit original variant missing")?;
        expected.push((name, variant.def_id, facts.variant(role)));
    }
    for (role, definition, actual) in expected {
        let hash = tcx.def_path_hash(definition);
        let expected = Id {
            crate_id: hash.stable_crate_id().as_u64(),
            definition_path_hash: hash.local_hash().as_u64(),
        };
        if expected != actual {
            return Err(format!("compiler error-state role mismatch: {role}"));
        }
    }
    Ok(())
}
