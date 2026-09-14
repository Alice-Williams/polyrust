//! Explicit input dependencies, checked against compiler expansion bookkeeping.
use rustc_middle::ty::TyCtxt;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub(crate) struct DeclaredInputs {
    files: BTreeSet<PathBuf>,
    root: String,
}

impl DeclaredInputs {
    pub(crate) fn new(root: &str, arguments: &[String]) -> Result<Self, String> {
        let root = canonical(Path::new(root))?;
        let mut files = BTreeSet::from([root.clone()]);
        for pair in arguments.chunks(2) {
            let [flag, path] = pair else {
                return Err("expected --input PATH pair".into());
            };
            if flag != "--input" {
                return Err("expected --input PATH pair".into());
            }
            files.insert(canonical(Path::new(path))?);
        }
        Ok(Self {
            files,
            root: root
                .to_str()
                .ok_or("compiler input path is not UTF-8")?
                .into(),
        })
    }

    pub(crate) fn root(&self) -> &str {
        &self.root
    }

    pub(crate) fn verify(&self, tcx: TyCtxt<'_>) -> Result<(), String> {
        if !tcx.sess.env_depinfo.lock().is_empty() {
            return Err("compiler environment-dependent input is not implemented".into());
        }
        for file in tcx.sess.file_depinfo.lock().iter() {
            self.require(Path::new(file.as_str()))?;
        }
        // include! and out-of-line modules use the compiler source map, whereas
        // include_str!/include_bytes! use the expansion dependency set above.
        for source in tcx.sess.source_map().files().iter() {
            if !source.is_imported()
                && let rustc_span::FileName::Real(name) = &source.name
            {
                self.require(
                    name.local_path()
                        .ok_or("local compiler input has no filesystem identity")?,
                )?;
            }
        }
        Ok(())
    }

    fn require(&self, input: &Path) -> Result<(), String> {
        let path = canonical(input)?;
        if self.files.contains(&path) {
            Ok(())
        } else {
            Err(format!(
                "undeclared compiler file input: {}",
                path.display()
            ))
        }
    }
}

fn canonical(path: &Path) -> Result<PathBuf, String> {
    let resolved = path
        .canonicalize()
        .map_err(|error| format!("input {}: {error}", path.display()))?;
    if !resolved.is_file() {
        return Err(format!("input is not a file: {}", path.display()));
    }
    Ok(resolved)
}
