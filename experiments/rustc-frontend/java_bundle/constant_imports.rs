//! Descriptions of dependency values come only from certified target references.
use crate::{json::Sink, projection::scalar, serialization::path};
use portable_backend_java::dialect::{JavaDependencyApi, JavaDependencyConstant};
use portable_codegen::{RustDeclarationId, TargetSymbolRef};
use std::collections::BTreeMap;

pub(crate) fn collect(
    api: &JavaDependencyApi,
) -> Result<BTreeMap<RustDeclarationId, &JavaDependencyConstant>, String> {
    let mut constants = BTreeMap::new();
    for symbol in api
        .package()
        .ast()
        .files()
        .iter()
        .flat_map(|file| file.items())
        .flat_map(|item| item.names.keys())
    {
        if let TargetSymbolRef::DependencyValue(value) = symbol {
            let proof = value.constant();
            if let Some(previous) = constants.insert(proof.declaration(), proof)
                && previous != proof
            {
                return Err("conflicting Java constant import authorities".into());
            }
        }
    }
    Ok(constants)
}
pub(crate) fn write(
    out: &mut impl Sink,
    constants: &BTreeMap<RustDeclarationId, &JavaDependencyConstant>,
) -> Result<(), String> {
    out.fixed(",\"constant_imports\":[")?;
    for (position, (id, proof)) in constants.iter().enumerate() {
        if position != 0 {
            out.fixed(",")?;
        }
        out.fixed("{\"id\":")?;
        out.id(*id)?;
        out.fixed(",\"owner\":")?;
        out.id(proof.package_identity().root())?;
        out.fixed(",\"path\":")?;
        path(out, proof.path())?;
        out.fixed(",\"scalar\":")?;
        out.string(scalar(proof.ty())?)?;
        out.fixed(",\"readonly\":true,\"value\":")?;
        crate::constants::value(out, proof.value())?;
        out.fixed("}")?;
    }
    out.fixed("]")
}
