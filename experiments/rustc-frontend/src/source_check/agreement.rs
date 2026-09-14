//! Typed compiler identity joined to the exact staged artifact, not its name.
use crate::metadata_dependencies::MetadataDependencies;
use portable_rustc_configuration::graph::CrateGraph;
use rustc_data_structures::svh::Svh;
use rustc_hir::def_id::{CrateNum, LOCAL_CRATE};
use rustc_metadata::creader::CStore;
use rustc_middle::ty::TyCtxt;
use rustc_span::{Symbol, def_id::StableCrateId};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct SourceEvidence {
    identity: StableCrateId,
    content: Svh,
}

impl SourceEvidence {
    pub(super) fn local(tcx: TyCtxt<'_>) -> Self {
        Self::read(tcx, LOCAL_CRATE)
    }

    fn read(tcx: TyCtxt<'_>, krate: CrateNum) -> Self {
        Self {
            identity: tcx.stable_crate_id(krate),
            content: tcx.crate_hash(krate),
        }
    }

    pub(super) fn same_owner(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

pub(super) fn verify(
    tcx: TyCtxt<'_>,
    graph: &CrateGraph,
    staged: &MetadataDependencies,
    checked: &BTreeMap<String, SourceEvidence>,
    toolchain_artifacts: &BTreeSet<PathBuf>,
) -> Result<BTreeMap<CrateNum, String>, String> {
    let mut expected = BTreeMap::new();
    for description in graph.crates() {
        if description.key() != graph.root_key() {
            let path = staged
                .artifact(description.key())
                .ok_or("dependency artifact not staged")?;
            let proof = checked
                .get(description.key())
                .ok_or("dependency source not checked")?;
            expected.insert(path, (description.key(), proof));
        }
    }
    let mut loaded = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for krate in tcx.crates(()) {
        let source = tcx.used_crate_source(*krate);
        let path = source
            .rmeta
            .as_ref()
            .or(source.rlib.as_ref())
            .ok_or("loaded crate has no metadata artifact")?;
        let path = path.canonicalize().map_err(|error| error.to_string())?;
        if let Some((key, proof)) = expected.get(path.as_path()) {
            if SourceEvidence::read(tcx, *krate) != **proof {
                return Err(format!(
                    "source/metadata identity or content mismatch for {key}"
                ));
            }
            if !seen.insert(*key) {
                return Err("declared metadata artifact loaded as multiple crates".into());
            }
            loaded.insert(*key, *krate);
        } else if !toolchain_artifacts.contains(&path) {
            return Err(
                "compiler loaded metadata outside the declared closure and pinned sysroot".into(),
            );
        }
    }
    if seen.len() != expected.len() {
        return Err("compiler did not load every declared dependency artifact".into());
    }
    let root = graph
        .crates()
        .iter()
        .find(|item| item.key() == graph.root_key())
        .ok_or("source graph root missing")?;
    let store = CStore::from_tcx(tcx);
    for (alias, key) in root.dependencies() {
        if let Some(krate) = store.resolved_extern_crate(Symbol::intern(alias))
            && loaded.get(key) != Some(&krate)
        {
            return Err(format!(
                "compiler alias {alias} resolved to the wrong declared artifact"
            ));
        }
    }
    Ok(loaded
        .into_iter()
        .map(|(key, krate)| (krate, key.to_owned()))
        .collect())
}
