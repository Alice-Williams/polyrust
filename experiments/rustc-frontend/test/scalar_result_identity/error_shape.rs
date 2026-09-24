//! Original opaque error state. No private Rust field is exposed to input code.
use portable_codegen::{
    RustCanonicalErrorKindFacts as ErrorFacts, RustCanonicalInstanceFacts as Facts,
    RustIntegerErrorKind as Kind, RustIntegerErrorVariants as Variants,
};
use rustc_hir::def::DefKind;
use rustc_middle::ty::{self, Ty, TyCtxt};

pub(super) fn observe<'tcx>(
    tcx: TyCtxt<'tcx>,
    error: Ty<'tcx>,
    instance: Facts,
) -> Result<ErrorFacts, String> {
    let env = ty::TypingEnv::fully_monomorphized();
    let ty::Adt(wrapper, args) = error.kind() else {
        return Err("error state requires its original nominal wrapper".into());
    };
    if super::identity(tcx, wrapper.did()) != instance.key().error_definition()
        || !wrapper.is_struct()
        || !args.is_empty()
        || observed(
            wrapper.non_enum_variant().fields.len() as u128,
            Observation::WrapperFields,
        ) != 1
    {
        return Err("standard error wrapper inventory changed".into());
    }
    let field = wrapper.non_enum_variant().fields.iter().next().unwrap();
    if tcx.parent(field.did) != wrapper.did()
        || tcx.def_kind(field.did) != DefKind::Field
        || tcx.visibility(field.did).is_public()
    {
        return Err("standard error wrapper field ownership changed".into());
    }
    let kind = tcx
        .try_normalize_erasing_regions(env, field.ty(tcx, args))
        .map_err(|_| "standard error kind normalization failed")?;
    let ty::Adt(definition, kind_args) = kind.kind() else {
        return Err("standard error kind must be nominal".into());
    };
    if !definition.is_enum()
        || !kind_args.is_empty()
        || definition.did().krate != wrapper.did().krate
        || !tcx.visibility(definition.did()).is_public()
        || observed(
            definition.variants().len() as u128,
            Observation::VariantCount,
        ) != Kind::ALL.len() as u128
    {
        return Err("standard error kind inventory changed".into());
    }
    for value in [error, kind] {
        if value.needs_drop(tcx, env) || !tcx.type_is_copy_modulo_regions(env, value) {
            return Err("standard error state must be Copy without drop".into());
        }
        let bytes = tcx
            .layout_of(env.as_query_input(value))
            .map_err(|_| "standard error state layout unavailable")?
            .size
            .bytes();
        if observed(u128::from(bytes), Observation::Layout) != 1 {
            return Err("standard error state layout changed".into());
        }
    }
    let mut variants = Vec::new();
    for ((index, variant), role) in definition.variants().iter_enumerated().zip(Kind::ALL) {
        // The normalized original field type supplies nominal authority. Names
        // and ordinal checks validate the pinned semantic inventory, never
        // authenticate an arbitrary caller-provided same-shaped enum.
        if tcx.parent(variant.def_id) != definition.did()
            || tcx.def_kind(variant.def_id) != DefKind::Variant
            || observed(variant.fields.len() as u128, Observation::PayloadCount) != 0
            || variant.name.as_str() != name(role)
            || observed(
                definition.discriminant_for_variant(tcx, index).val,
                Observation::Discriminant,
            ) != role.transport_code() as u128
        {
            return Err("standard error kind variant mapping changed".into());
        }
        variants.push(super::identity(tcx, variant.def_id));
    }
    ErrorFacts::new(
        instance,
        super::identity(tcx, field.did),
        super::identity(tcx, definition.did()),
        Variants {
            empty: variants[0],
            invalid_digit: variants[1],
            positive_overflow: variants[2],
            negative_overflow: variants[3],
            zero: variants[4],
            not_a_power_of_two: variants[5],
        },
    )
    .map_err(|error| format!("original error state facts: {error:?}"))
}

fn name(kind: Kind) -> &'static str {
    match kind {
        Kind::Empty => "Empty",
        Kind::InvalidDigit => "InvalidDigit",
        Kind::PosOverflow => "PosOverflow",
        Kind::NegOverflow => "NegOverflow",
        Kind::Zero => "Zero",
        Kind::NotAPowerOfTwo => "NotAPowerOfTwo",
    }
}

#[derive(Clone, Copy)]
enum Observation {
    WrapperFields,
    VariantCount,
    PayloadCount,
    Layout,
    Discriminant,
}

#[cfg(not(instance_graph_error_shape_fault))]
fn observed(value: u128, _: Observation) -> u128 {
    value
}

#[cfg(instance_graph_error_shape_fault)]
fn observed(value: u128, role: Observation) -> u128 {
    let expected = match role {
        Observation::WrapperFields => "WrapperFields",
        Observation::VariantCount => "VariantCount",
        Observation::PayloadCount => "PayloadCount",
        Observation::Layout => "Layout",
        Observation::Discriminant => "Discriminant",
    };
    if std::env::var("POLYRUST_ERROR_SHAPE_FAULT").as_deref() == Ok(expected) {
        value.saturating_add(1)
    } else {
        value
    }
}
