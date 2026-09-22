//! Public aliases are borrowed from exact certified target evidence.
use crate::{json::Sink, projection::scalar, serialization::path};
use portable_backend_java::dialect::{JavaDependencyApi, JavaForeignConstantExport};
use portable_codegen::{RustExportNamespace, RustExportTarget};
use std::collections::BTreeMap;

pub(crate) fn collect(api: &JavaDependencyApi) -> Result<Vec<&JavaForeignConstantExport>, String> {
    let exports = api.foreign_constants().collect::<Vec<_>>();
    let mut bindings = BTreeMap::new();
    for export in &exports {
        let proof = export.dependency();
        if export.module().crate_id != api.root().crate_id
            || export.name().namespace != RustExportNamespace::Value
            || proof.declaration().crate_id == api.root().crate_id
            || proof.package_identity().root().crate_id != proof.declaration().crate_id
            || bindings
                .insert((export.module(), export.name()), proof)
                .is_some()
        {
            return Err("Java constant export differs from original producer authority".into());
        }
    }
    let graph = crate::projection::exports(api)?;
    for (module, names) in &graph.modules {
        for (name, target) in names {
            let id = match target {
                RustExportTarget::Declaration(id) | RustExportTarget::Module(id) => id,
            };
            if id.crate_id != api.root().crate_id
                && (!matches!(target, RustExportTarget::Declaration(_))
                    || bindings
                        .remove(&(*module, name))
                        .is_none_or(|proof| proof.declaration() != *id))
            {
                return Err("Java foreign binding lacks exact certified export evidence".into());
            }
        }
    }
    if !bindings.is_empty() {
        return Err("Java foreign evidence lacks compiler binding".into());
    }
    Ok(exports)
}

pub(crate) fn write(
    out: &mut impl Sink,
    exports: &[&JavaForeignConstantExport],
) -> Result<(), String> {
    out.fixed(",\"constant_exports\":[")?;
    for (position, export) in exports.iter().enumerate() {
        if position != 0 {
            out.fixed(",")?;
        }
        let proof = export.dependency();
        out.fixed("{\"module\":")?;
        out.id(export.module())?;
        out.fixed(",\"namespace\":\"value\",\"name\":")?;
        out.string(&export.name().name)?;
        out.fixed(",\"id\":")?;
        out.id(proof.declaration())?;
        out.fixed(",\"owner\":")?;
        out.id(proof.package_identity().root())?;
        out.fixed(",\"path\":")?;
        path(out, proof.path())?;
        out.fixed(",\"scalar\":")?;
        out.string(scalar(proof.ty())?)?;
        out.fixed(",\"readonly\":true,\"value\":")?;
        crate::constants::certified_value(out, proof.value())?;
        out.fixed("}")?;
    }
    out.fixed("]")
}
