//! Builder for one declared crate, with typed names and separate alias edges.
use super::{
    InputMapping, MAX_EDGES, MAX_INPUTS,
    inputs::{InputPath, LogicalPath},
};
use crate::{Configuration, CrateKey, CrateName, Identity, Mode};
use std::collections::BTreeMap;

/// Declared build configuration only, never compiler or target authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrateDescription {
    name: CrateName,
    key: CrateKey,
    root: LogicalPath,
    pub(super) inputs: BTreeMap<LogicalPath, InputMapping>,
    pub(super) metadata: InputPath,
    pub(super) dependencies: BTreeMap<CrateName, CrateKey>,
}

impl CrateDescription {
    pub fn new(name: &str, key: &str, root: InputMapping, metadata: &str) -> Result<Self, String> {
        if !root.logical().ends_with(".rs") {
            return Err("crate root logical name must end in .rs".into());
        }
        if !metadata.ends_with(".rmeta") {
            return Err("metadata artifact path must end in .rmeta".into());
        }
        Ok(Self {
            name: CrateName::new(name)?,
            key: CrateKey::new(key)?,
            root: root.logical.clone(),
            inputs: BTreeMap::from([(root.logical.clone(), root)]),
            metadata: InputPath::new(metadata)?,
            dependencies: BTreeMap::new(),
        })
    }

    pub fn with_input(mut self, input: InputMapping) -> Result<Self, String> {
        if self.inputs.len() >= MAX_INPUTS {
            return Err("crate input budget exceeded".into());
        }
        if self.inputs.contains_key(&input.logical)
            || self
                .inputs
                .values()
                .any(|old| old.physical == input.physical)
        {
            return Err("duplicate physical or logical crate input".into());
        }
        self.inputs.insert(input.logical.clone(), input);
        Ok(self)
    }

    pub fn with_dependency(mut self, alias: &str, key: &str) -> Result<Self, String> {
        if self.dependencies.len() >= MAX_EDGES {
            return Err("crate dependency edge budget exceeded".into());
        }
        let alias = CrateName::new(alias)?;
        let key = CrateKey::new(key)?;
        if self.dependencies.insert(alias, key).is_some() {
            return Err("duplicate dependency alias".into());
        }
        Ok(self)
    }

    pub fn name(&self) -> &str {
        &self.name.0
    }

    pub fn key(&self) -> &str {
        &self.key_value().0
    }

    pub(super) fn key_value(&self) -> &CrateKey {
        &self.key
    }

    pub fn configuration(&self) -> Configuration {
        Configuration {
            mode: Mode::PublicPackage,
            identity: Identity::Explicit {
                name: self.name.clone(),
                key: self.key.clone(),
            },
            inputs: Vec::new(),
        }
    }

    pub fn root(&self) -> &InputMapping {
        &self.inputs[&self.root]
    }

    pub fn inputs(&self) -> impl ExactSizeIterator<Item = &InputMapping> {
        self.inputs.values()
    }

    pub fn metadata(&self) -> &str {
        self.metadata.as_str()
    }

    pub fn dependencies(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.dependencies
            .iter()
            .map(|(alias, key)| (alias.0.as_str(), key.0.as_str()))
    }
}
