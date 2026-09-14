//! Publish one compiler-produced metadata file without replacing existing output.
use crate::metadata_stage::MetadataStage;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) struct MetadataOutput {
    destination: PathBuf,
    stage: MetadataStage,
}

impl MetadataOutput {
    pub(crate) fn prepare(path: &Path) -> Result<Self, String> {
        let absolute = std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path);
        let name = absolute
            .file_name()
            .ok_or("metadata output must name a new file")?;
        let parent = absolute
            .parent()
            .ok_or("metadata output has no parent")?
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let destination = parent.join(name);
        absent(&destination)?;
        Ok(Self {
            destination,
            stage: MetadataStage::prepare(&parent)?,
        })
    }

    pub(crate) fn staged_path(&self) -> &Path {
        self.stage.output()
    }

    // The caller must already have successful rustc analysis/emission. This I/O
    // helper is not itself compiler evidence or a C dependency certificate.
    pub(crate) fn publish(self) -> Result<(), String> {
        let metadata =
            fs::symlink_metadata(self.staged_path()).map_err(|error| error.to_string())?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err("compiler did not produce regular nonempty metadata".into());
        }
        fs::File::open(self.staged_path())
            .and_then(|file| file.sync_all())
            .map_err(|error| error.to_string())?;
        // Same-filesystem hard-link creation atomically fails if the destination
        // already exists, including if it appeared after prepare(). No replace.
        fs::hard_link(self.staged_path(), &self.destination)
            .map_err(|error| format!("metadata publication: {error}"))
    }
}

fn absent(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
        Ok(_) => Err("metadata output already exists; choose a new file".into()),
    }
}
