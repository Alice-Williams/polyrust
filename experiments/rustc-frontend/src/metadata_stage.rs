//! Exclusively owned compiler scratch directory with nonrecursive cleanup.
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) struct MetadataStage {
    directory: PathBuf,
    output: PathBuf,
}

impl MetadataStage {
    pub(crate) fn prepare(parent: &Path) -> Result<Self, String> {
        let parent = parent.canonicalize().map_err(|error| error.to_string())?;
        for attempt in 0..128 {
            let directory = parent.join(format!(
                ".polyrust-metadata-{}-{attempt}",
                std::process::id()
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&directory) {
                Ok(()) => {
                    return Ok(Self {
                        output: directory.join("output.rmeta"),
                        directory,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.to_string()),
            }
        }
        Err("metadata staging directory budget exhausted".into())
    }

    pub(crate) fn output(&self) -> &Path {
        &self.output
    }
}

impl Drop for MetadataStage {
    fn drop(&mut self) {
        // Never traverse unknown entries or delete preexisting staging names.
        let _ = fs::remove_file(&self.output);
        let _ = fs::remove_dir(&self.directory);
    }
}
