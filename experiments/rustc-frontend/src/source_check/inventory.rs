//! Whole-graph filesystem validation before the first source compiler runs.
use portable_rustc_configuration::graph::{CrateGraph, InputMapping, ResolvedInputs};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

/// Bazel runfiles may symlink individual sysroot artifacts outside the apparent
/// sysroot directory. Trust exact pinned files, never an ambient path prefix.
pub(super) fn toolchain_artifacts(sysroot: &Path) -> Result<BTreeSet<PathBuf>, String> {
    let directory = sysroot.join("lib/rustlib/x86_64-unknown-linux-gnu/lib");
    let mut artifacts = BTreeSet::new();
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "rlib" || extension == "rmeta")
        {
            let path = path.canonicalize().map_err(|e| e.to_string())?;
            if !path.is_file() {
                return Err("pinned compiler artifact is not regular".into());
            }
            artifacts.insert(path);
        }
    }
    Ok(artifacts)
}

pub(super) fn resolve(graph: &CrateGraph) -> Result<BTreeMap<String, ResolvedInputs>, String> {
    let mut sources = BTreeMap::new();
    let mut input_identities = BTreeSet::new();
    let mut path_bytes = 0usize;
    for description in graph.crates() {
        let resolved = ResolvedInputs::load(description)?;
        for input in resolved.mappings() {
            charge(
                &mut path_bytes,
                input.physical().len() + input.logical().len(),
            )?;
            let metadata = fs::metadata(input.physical()).map_err(|error| error.to_string())?;
            if !metadata.is_file() {
                return Err("resolved graph source is no longer regular".into());
            }
            // One physical source can intentionally define two distinct crates.
            input_identities.insert((metadata.dev(), metadata.ino()));
        }
        sources.insert(description.key().to_owned(), resolved);
    }
    let mut artifact_identities = BTreeSet::new();
    let mut total_bytes = 0u64;
    for description in graph.crates() {
        if description.key() == graph.root_key() {
            continue; // Root metadata is not consumed by source checking.
        }
        let path = Path::new(description.metadata())
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let mapping = InputMapping::new(
            path.to_str().ok_or("metadata path is not UTF-8")?,
            "metadata.rmeta",
        )?;
        charge(&mut path_bytes, mapping.physical().len())?;
        let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
        if !metadata.is_file() || metadata.len() == 0 || metadata.len() > 64 * 1024 * 1024 {
            return Err("graph metadata requires a regular nonempty file within 64 MiB".into());
        }
        let identity = (metadata.dev(), metadata.ino());
        if input_identities.contains(&identity) || !artifact_identities.insert(identity) {
            return Err("graph metadata aliases a source or another artifact".into());
        }
        total_bytes = total_bytes
            .checked_add(metadata.len())
            .ok_or("graph metadata size overflow")?;
        if total_bytes > 256 * 1024 * 1024 {
            return Err("graph metadata closure exceeds 256 MiB".into());
        }
    }
    Ok(sources)
}

fn charge(total: &mut usize, amount: usize) -> Result<(), String> {
    *total = total
        .checked_add(amount)
        .ok_or("resolved graph path size overflow")?;
    if *total > 8 * 1024 * 1024 {
        return Err("resolved graph paths exceed 8 MiB".into());
    }
    Ok(())
}
