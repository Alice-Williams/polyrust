//! Same-analysis compiler identity to exact public C certificate binding.
use portable_backend_c::dialect::CDependencyApi;
use portable_codegen::RustDeclarationId;
use rustc_hir::def_id::{DefId, LOCAL_CRATE};
use std::collections::BTreeMap;

#[derive(Clone)]
pub(crate) struct CheckedCrate {
    api: CDependencyApi,
    manifest: crate::api_manifest::ApiManifest,
}

impl CheckedCrate {
    pub(crate) fn api(&self) -> &CDependencyApi {
        &self.api
    }
    pub(crate) fn manifest(&self) -> &crate::api_manifest::ApiManifest {
        &self.manifest
    }
}

/// No caller-provided descriptors or deserialized manifests can construct this.
#[derive(Clone)]
pub(crate) struct CheckedGraph {
    root: RustDeclarationId,
    crates: BTreeMap<RustDeclarationId, CheckedCrate>,
}

impl CheckedGraph {
    pub(crate) fn root(&self) -> RustDeclarationId {
        self.root
    }
    pub(crate) fn crates(&self) -> &BTreeMap<RustDeclarationId, CheckedCrate> {
        &self.crates
    }
}

pub(crate) fn check(sysroot: &str, arguments: &[String]) -> Result<usize, String> {
    Ok(lower(sysroot, arguments)?.crates.len())
}

pub(crate) fn lower(sysroot: &str, arguments: &[String]) -> Result<CheckedGraph, String> {
    let graph = crate::metadata_cli::parse(arguments)?;
    let checked = crate::source_check::check::<CheckedCrate>(
        &graph,
        sysroot,
        |_, tcx, dependencies| {
            let function_lookup = |definition: DefId| {
                let owner = dependencies
                    .get(&definition.krate)
                    .map(|item| item.api())
                    .ok_or("foreign call has no source-authenticated owning C package")?;
                #[cfg(c_graph_wrong_owner)]
                let owner = crate::foreign_mutations::owner(owner, dependencies);
                if owner.root().crate_id != tcx.stable_crate_id(definition.krate).as_u64() {
                    return Err("foreign C certificate belongs to the wrong compiler crate".into());
                }
                let hash = tcx.def_path_hash(definition);
                let proof = owner
                    .function(RustDeclarationId {
                        crate_id: hash.stable_crate_id().as_u64(),
                        definition_path_hash: hash.local_hash().as_u64(),
                    })
                    .cloned()
                    .ok_or("foreign call is not in the certified public C API")?;
                #[cfg(c_graph_wrong_declaration)]
                let proof = crate::foreign_mutations::declaration(owner, proof);
                let original = crate::source_origin::types::signature(tcx, definition)?;
                if dependencies
                    .get(&definition.krate)
                    .and_then(|item| item.manifest().source_signature(proof.declaration()))
                    != Some(&original)
                {
                    return Err(
                        "foreign original Rust signature differs from C source type facts".into(),
                    );
                }
                Ok(proof)
            };
            let constant_lookup = |definition: DefId| {
                let owner = dependencies
                    .get(&definition.krate)
                    .map(|item| item.api())
                    .ok_or("foreign constant has no source-authenticated owning C package")?;
                if owner.root().crate_id != tcx.stable_crate_id(definition.krate).as_u64() {
                    return Err("foreign C constant belongs to the wrong compiler crate".into());
                }
                let id = crate::source_origin::identity(tcx, definition);
                let proof = owner
                    .constant(id)
                    .cloned()
                    .ok_or("foreign constant is not in the certified public C API")?;
                #[cfg(constant_import_wrong_owner)]
                let proof = dependencies
                    .values()
                    .filter(|other| other.api().root().crate_id != id.crate_id)
                    .find_map(|other| other.api().constants().next().cloned())
                    .unwrap_or(proof);
                #[cfg(constant_import_wrong_declaration)]
                let proof = owner
                    .constants()
                    .find(|other| other.declaration() != proof.declaration())
                    .cloned()
                    .ok_or("constant mutation requires another declaration")?;
                #[cfg(constant_import_replaced_owner)]
                let proof = {
                    let replacement = CDependencyApi::from_certificate(owner.package().clone())?;
                    replacement
                        .constant(proof.declaration())
                        .cloned()
                        .ok_or("constant mutation requires a public declaration")?
                };
                let original =
                    crate::source_capabilities::original_constant_value(tcx, definition)?;
                if dependencies
                    .get(&definition.krate)
                    .and_then(|item| item.manifest().source_constant(proof.declaration()))
                    != Some(original)
                {
                    return Err(
                        "foreign compiler identity/type/value differs: original Rust value differs from C source constant facts".into(),
                    );
                }
                Ok(proof)
            };
            let lookup = crate::c_lower::ForeignLookup {
                function: &function_lookup,
                constant: &constant_lookup,
            };
            let program = crate::extract::dependency_program(tcx, &lookup)?;
            let manifest = program
                .manifest
                .ok_or("checked C crate has no public API inventory")?;
            // Check-mode validates the descriptive bundle inventory too, but never publishes it.
            manifest.bundle_json()?;
            let api = CDependencyApi::from_certificate(program.package)?;
            if api.root().crate_id != tcx.stable_crate_id(LOCAL_CRATE).as_u64() {
                return Err("checked C package differs from its compiler source owner".into());
            }
            Ok(CheckedCrate { api, manifest })
        },
    )?;
    let root = checked
        .get(graph.root_key())
        .ok_or("checked root missing")?
        .api
        .root();
    let count = checked.len();
    let crates: BTreeMap<_, _> = checked
        .into_values()
        .map(|item| (item.api.root(), item))
        .collect();
    if crates.len() != count {
        return Err("duplicate checked C owner".into());
    }
    let graph = CheckedGraph { root, crates };
    #[cfg(c_graph_inventory_contract)]
    bundle_contract::check(&graph);
    Ok(graph)
}

#[cfg(c_graph_inventory_contract)]
#[path = "../test/c_bundle_contract.rs"]
mod bundle_contract;
