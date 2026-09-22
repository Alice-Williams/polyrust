//! Read original compiler types; target primitives cannot reconstruct these facts.
use super::{Result, identity};
use portable_codegen::{
    RustDeclarationId, RustFieldTypes, RustFunctionTypes, RustResultKind, RustScalarKind,
    RustSourceTypes,
};
extern crate rustc_abi;
use self::rustc_abi::ExternAbi;
use rustc_hir::{
    def::DefKind,
    def_id::{CRATE_DEF_ID, DefId, LocalDefId},
};
use rustc_middle::ty::{self, Ty, TyCtxt};
use std::collections::BTreeMap;

fn scalar(value: Ty<'_>) -> Result<RustScalarKind> {
    match value.kind() {
        ty::Int(ty::IntTy::I32) => Ok(RustScalarKind::I32),
        ty::Int(ty::IntTy::I64) => Ok(RustScalarKind::I64),
        ty::Bool => Ok(RustScalarKind::Bool),
        ty::Char => Ok(RustScalarKind::Char),
        ty::Float(ty::FloatTy::F64) => Ok(RustScalarKind::F64),
        _ => Err("source type metadata requires an admitted scalar".into()),
    }
}

pub(crate) fn signature(tcx: TyCtxt<'_>, definition: DefId) -> Result<RustFunctionTypes> {
    if tcx.def_kind(definition) != DefKind::Fn || tcx.generics_of(definition).count() != 0 {
        return Err("source signature metadata requires a nongeneric ordinary function".into());
    }
    let signature = tcx.fn_sig(definition).instantiate_identity().skip_binder();
    if signature.abi() != ExternAbi::Rust
        || !signature.safety().is_safe()
        || signature.c_variadic()
        || signature.splatted().is_some()
    {
        return Err("source signature metadata requires a safe ordinary Rust ABI".into());
    }
    Ok(RustFunctionTypes {
        parameters: signature
            .inputs()
            .iter()
            .map(|ty| scalar(*ty))
            .collect::<Result<_>>()?,
        result: match signature.output().kind() {
            ty::Tuple(fields) if fields.is_empty() => RustResultKind::Unit,
            _ => RustResultKind::Scalar(scalar(signature.output())?),
        },
    })
}

pub(crate) fn collect(
    tcx: TyCtxt<'_>,
    root: RustDeclarationId,
    functions: &[LocalDefId],
    records: impl IntoIterator<Item = DefId>,
) -> Result<RustSourceTypes> {
    if root != identity(tcx, CRATE_DEF_ID.to_def_id()) {
        return Err("source type inventory belongs to another compiler owner".into());
    }
    let mut function_types = BTreeMap::new();
    for definition in functions {
        let definition = definition.to_def_id();
        if function_types
            .insert(identity(tcx, definition), signature(tcx, definition)?)
            .is_some()
        {
            return Err("duplicate original function type declaration".into());
        }
    }
    let mut fields = BTreeMap::new();
    for record in records {
        if !record.is_local()
            || tcx.def_kind(record) != DefKind::Struct
            || tcx.generics_of(record).count() != 0
        {
            return Err("source field metadata requires an original local scalar record".into());
        }
        for field in &tcx.adt_def(record).non_enum_variant().fields {
            let value = tcx
                .try_normalize_erasing_regions(
                    ty::TypingEnv::fully_monomorphized(),
                    tcx.type_of(field.did).instantiate_identity(),
                )
                .map_err(|_| "source field type normalization failed")?;
            let kind = scalar(value)?;
            let original = RustFieldTypes {
                owner: identity(tcx, record),
                kind,
            };
            if fields.insert(identity(tcx, field.did), original).is_some() {
                return Err("duplicate original field type declaration".into());
            }
        }
    }
    RustSourceTypes::new(root, function_types, fields)
}

/// Reconcile descriptive facts at attachment, using original compiler queries,
/// rather than interpreting a target primitive as evidence of a source type.
pub(crate) fn authenticate(
    tcx: TyCtxt<'_>,
    functions: &[LocalDefId],
    records: impl IntoIterator<Item = DefId>,
    facts: &RustSourceTypes,
) -> Result<()> {
    let expected = collect(
        tcx,
        identity(tcx, CRATE_DEF_ID.to_def_id()),
        functions,
        records,
    )?;
    if &expected != facts {
        return Err("source type facts differ from original compiler declarations".into());
    }
    Ok(())
}

#[cfg(character_source_probe)]
#[path = "../../test/character_type_probe.rs"]
pub(crate) mod probe;
