//! Reconstruct imports from certified target authority and compiler bindings.
use portable_backend_c::{
    ast::{CFunctionRef, CGeneratedOrigin},
    dialect::{CDependencyFunction, CDialect, c_imported_functions},
};
use portable_codegen::{RenderReadyPackage, RustDeclarationId, RustSourceNode};
use std::collections::BTreeMap;

pub(crate) type ExpectedImports = BTreeMap<RustDeclarationId, (CFunctionRef, CDependencyFunction)>;

pub(super) fn collect(
    package: &RenderReadyPackage<CDialect>,
    root: RustDeclarationId,
    expected: &ExpectedImports,
) -> Result<ExpectedImports, String> {
    let mut imports = BTreeMap::new();
    for imported in c_imported_functions(package) {
        let function = imported.function();
        let proof = imported.dependency();
        let id = proof.declaration();
        let CGeneratedOrigin::RustSource(origin) = &function.key().origin else {
            return Err("API import lacks compiler provenance".into());
        };
        if id.crate_id == root.crate_id
            || origin.declaration != id
            || origin.node != RustSourceNode::Declaration
            || !origin.externally_reachable
            || proof.package_identity().root().crate_id != id.crate_id
            || function.signature() != proof.signature()
            || function.file() != proof.public_header().file()
            || expected.get(&id) != Some(&(function.clone(), proof.clone()))
        {
            return Err("API import compiler/reference/certificate mapping disagrees".into());
        }
        if imports
            .insert(id, (function.clone(), proof.clone()))
            .is_some()
        {
            return Err("duplicate API import identity".into());
        }
    }
    if imports.len() != expected.len() {
        return Err("API manifest misses a compiler or target import".into());
    }
    Ok(imports)
}
