//! Declared files resolved to physical identities and stable compiler source names.
use super::{CrateDescription, InputMapping, MAX_PATH_BYTES, charge, inputs::InputPath};
use crate::Configuration;
use std::{collections::BTreeSet, path::Path};

/// Checked filesystem descriptors, not proof of source contents or Rust validity.
/// The driver must still verify rustc's reads and compiler metadata agreement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedInputs {
    configuration: Configuration,
    crate_name: String,
    root: String,
    mappings: Vec<InputMapping>,
}

impl ResolvedInputs {
    pub fn load(description: &CrateDescription) -> Result<Self, String> {
        let mut physical_names = BTreeSet::new();
        #[cfg(unix)]
        let mut file_identities = BTreeSet::new();
        let mut mappings = Vec::new();
        let mut root = None;
        let mut path_bytes = 0;
        for input in description.inputs() {
            let canonical = Path::new(input.physical())
                .canonicalize()
                .map_err(|error| format!("declared input {}: {error}", input.physical()))?;
            if !canonical.is_file() {
                return Err(format!(
                    "declared input is not a regular file: {}",
                    canonical.display()
                ));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let metadata = canonical.metadata().map_err(|error| error.to_string())?;
                if !file_identities.insert((metadata.dev(), metadata.ino())) {
                    return Err("source/doc inode alias has multiple logical mappings".into());
                }
            }
            let canonical = canonical
                .to_str()
                .ok_or("canonical input path is not UTF-8")?;
            // Recheck path syntax after symlink resolution; remapping uses '='.
            let mapping = InputMapping::new(canonical, input.logical())?;
            charge(&mut path_bytes, mapping.physical().len(), MAX_PATH_BYTES)?;
            charge(&mut path_bytes, mapping.logical().len(), MAX_PATH_BYTES)?;
            if !physical_names.insert(mapping.physical().to_owned()) {
                return Err(
                    "canonical source/doc input alias has multiple logical mappings".into(),
                );
            }
            if input.logical() == description.root().logical() {
                root = Some(mapping.physical().to_owned());
            }
            mappings.push(mapping);
        }
        Ok(Self {
            configuration: description.configuration(),
            crate_name: description.name().into(),
            root: root.ok_or("resolved input set has no crate root")?,
            mappings,
        })
    }

    pub fn root(&self) -> &str {
        &self.root
    }

    pub fn mappings(&self) -> &[InputMapping] {
        &self.mappings
    }

    /// Existing declared-read verification remains mandatory after analysis.
    pub fn declared_input_arguments(&self) -> Vec<String> {
        self.mappings
            .iter()
            .filter(|input| input.physical() != self.root)
            .flat_map(|input| ["--input".into(), input.physical().into()])
            .collect()
    }

    /// Fixed source/configuration portion of metadata-mode compiler arguments.
    /// The driver supplies its pinned sysroot, fresh staging output and separately
    /// checked dependency artifacts; this method accepts no arbitrary rustc flags.
    pub fn metadata_arguments(
        &self,
        sysroot: &str,
        staged_output: &str,
    ) -> Result<Vec<String>, String> {
        let directory = std::env::current_dir().map_err(|error| error.to_string())?;
        let directory = InputPath::new(
            directory
                .to_str()
                .ok_or("compiler working directory is not UTF-8")?,
        )?;
        let mut arguments = self.configuration.compiler_arguments(&self.root, sysroot);
        arguments.extend(["--emit=metadata".into(), "-o".into(), staged_output.into()]);
        // rustc also encodes its working directory in metadata. Include it in
        // the specificity order: a source file can textually prefix the cwd.
        let mut mappings = vec![(directory.as_str(), "/polyrust/build".to_owned())];
        // Matching is textual, not path-component-aware: `/x/doc` also matches
        // `/x/doc.rs`. Emit more specific prefixes last, independently of the
        // logical-name ordering. Equal-length unrelated names stay stable.
        mappings.extend(self.mappings.iter().map(|input| {
            (
                input.physical(),
                format!("/polyrust/inputs/{}/{}", self.crate_name, input.logical()),
            )
        }));
        mappings.sort_by_key(|(physical, _)| physical.len());
        for (physical, logical) in mappings {
            arguments.push(format!("--remap-path-prefix={physical}={logical}"));
        }
        Ok(arguments)
    }
}
