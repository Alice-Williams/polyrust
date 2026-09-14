//! Exact private metadata snapshots; configuration, not compiler/target proof.
use portable_rustc_configuration::graph::{CrateGraph, InputMapping, ResolvedInputs};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
const MAX_PATH_BYTES: usize = 8 * 1024 * 1024;

pub(crate) struct MetadataDependencies {
    directory: PathBuf,
    files: BTreeMap<String, PathBuf>,
}

impl MetadataDependencies {
    /// `parent` is the exclusively owned output stage, never an ambient directory.
    pub(crate) fn prepare(
        graph: &CrateGraph,
        source: &ResolvedInputs,
        parent: &Path,
    ) -> Result<Self, String> {
        let directory = parent.join("dependencies");
        fs::create_dir(&directory).map_err(|error| error.to_string())?;
        let mut stage = Self {
            directory,
            files: BTreeMap::new(),
        };
        let mut paths = BTreeSet::new();
        let mut identities = BTreeSet::new();
        let mut path_bytes = 0usize;
        for input in source.mappings() {
            let path = PathBuf::from(input.physical());
            paths.insert(path.clone());
            if !identities.insert(identity(
                &fs::metadata(path).map_err(|error| error.to_string())?,
            )) {
                return Err("source/doc identity changed to an alias after resolution".into());
            }
            charge_paths(
                &mut path_bytes,
                input.physical().len() + input.logical().len(),
            )?;
        }
        let mut total = 0u64;
        for (index, description) in graph.crates().iter().enumerate() {
            if description.key() == graph.root_key() {
                continue;
            }
            let canonical = Path::new(description.metadata())
                .canonicalize()
                .map_err(|error| format!("declared dependency metadata: {error}"))?;
            let mapping = InputMapping::new(
                canonical.to_str().ok_or("metadata path is not UTF-8")?,
                "metadata.rmeta",
            )?;
            charge_paths(&mut path_bytes, mapping.physical().len())?;
            if !fs::metadata(&canonical)
                .map_err(|error| error.to_string())?
                .is_file()
            {
                return Err("dependency metadata is not a regular file".into());
            }
            let file = fs::File::open(&canonical).map_err(|error| error.to_string())?;
            let metadata = file.metadata().map_err(|error| error.to_string())?;
            if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_FILE_BYTES {
                return Err(
                    "dependency metadata requires a regular nonempty file within 64 MiB".into(),
                );
            }
            if !paths.insert(canonical) || !identities.insert(identity(&metadata)) {
                return Err("dependency metadata aliases another artifact or source input".into());
            }
            total = total
                .checked_add(metadata.len())
                .ok_or("metadata byte budget overflow")?;
            if total > MAX_TOTAL_BYTES {
                return Err("dependency metadata closure exceeds 256 MiB".into());
            }
            let path = stage
                .directory
                .join(format!("lib{}-{index}.rmeta", description.name()));
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|error| error.to_string())?;
            // Register ownership before any copying can fail, so Drop cleans it.
            stage.files.insert(description.key().into(), path);
            let copied = std::io::copy(&mut file.take(metadata.len() + 1), &mut output)
                .map_err(|error| error.to_string())?;
            if copied != metadata.len() {
                return Err("dependency metadata changed size during staging".into());
            }
            output.flush().map_err(|error| error.to_string())?;
        }
        Ok(stage)
    }

    pub(crate) fn compiler_arguments(&self, graph: &CrateGraph) -> Result<Vec<String>, String> {
        let root = graph
            .crates()
            .iter()
            .find(|item| item.key() == graph.root_key())
            .ok_or("metadata graph root missing")?;
        let mut arguments = Vec::new();
        if !self.files.is_empty() {
            arguments.extend([
                "-L".into(),
                format!(
                    "dependency={}",
                    self.directory
                        .to_str()
                        .ok_or("metadata staging directory is not UTF-8")?
                ),
            ]);
        }
        for (alias, key) in root.dependencies() {
            let path = self
                .artifact(key)
                .ok_or("direct dependency is not staged")?;
            arguments.extend([
                "--extern".into(),
                format!(
                    "{alias}={}",
                    path.to_str().ok_or("metadata staging path is not UTF-8")?
                ),
            ]);
        }
        Ok(arguments)
    }

    pub(crate) fn artifact(&self, key: &str) -> Option<&Path> {
        self.files.get(key).map(PathBuf::as_path)
    }
}

fn charge_paths(total: &mut usize, amount: usize) -> Result<(), String> {
    *total = total
        .checked_add(amount)
        .ok_or("resolved path budget overflow")?;
    if *total > MAX_PATH_BYTES {
        return Err("resolved metadata/source paths exceed 8 MiB".into());
    }
    Ok(())
}

#[cfg(unix)]
fn identity(metadata: &fs::Metadata) -> (u64, u64) {
    use std::os::unix::fs::MetadataExt;
    (metadata.dev(), metadata.ino())
}

impl Drop for MetadataDependencies {
    fn drop(&mut self) {
        for path in self.files.values() {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_dir(&self.directory);
    }
}
