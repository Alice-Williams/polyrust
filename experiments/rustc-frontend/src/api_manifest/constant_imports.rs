//! Constant import descriptions are reconstructed from exact target authority.
use portable_backend_c::{
    ast::{CGeneratedOrigin, CObjectRef},
    dialect::{CDependencyConstant, CDialect, c_imported_constants},
};
use portable_codegen::{RenderReadyPackage, RustDeclarationId, RustSourceNode};
use std::collections::BTreeMap;

pub(crate) type Expected = BTreeMap<RustDeclarationId, (CObjectRef, CDependencyConstant)>;

pub(super) fn collect(
    package: &RenderReadyPackage<CDialect>,
    root: RustDeclarationId,
    expected: &Expected,
) -> Result<Expected, String> {
    let mut result = BTreeMap::new();
    for imported in c_imported_constants(package) {
        let object = imported.object();
        let proof = imported.dependency();
        let id = proof.declaration();
        let CGeneratedOrigin::RustSource(origin) = &object.key().origin else {
            return Err("API constant import lacks compiler provenance".into());
        };
        if id.crate_id == root.crate_id
            || origin.declaration != id
            || origin.node != RustSourceNode::Declaration
            || !origin.externally_reachable
            || proof.package_identity().root().crate_id != id.crate_id
            || object.file() != proof.public_header().file()
            || expected.get(&id) != Some(&(object.clone(), proof.clone()))
        {
            return Err(
                "API constant import compiler/reference/certificate mapping disagrees".into(),
            );
        }
        if result.insert(id, (object.clone(), proof.clone())).is_some() {
            return Err("duplicate API constant import identity".into());
        }
    }
    if result.len() != expected.len() {
        return Err("API manifest misses a compiler or target constant import".into());
    }
    Ok(result)
}
