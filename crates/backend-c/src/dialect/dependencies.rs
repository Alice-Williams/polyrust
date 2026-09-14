//! Structural dependency discovery, not linking or a rendering certificate.

mod expressions;
mod files;
mod types;

#[cfg(test)]
#[path = "../tests/dependency_types.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/dependency_files.rs"]
mod file_tests;

use std::collections::{BTreeMap, BTreeSet};

use super::{CHeader, CSystemLibrary};
use crate::ast::{
    CContextError, CEnumRef, CFileRef, CFrozenRegistry, CFunctionRef, CObjectRef, CSourceFile,
    CStructRef, CTypedefRef, CUnionRef,
};

/// C17 has separate tag and ordinary namespaces. Never key these by spelling.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CTagDependency {
    Struct(CStructRef),
    Union(CUnionRef),
    Enum(CEnumRef),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CTypeRequirement {
    Declaration,
    Complete,
}

/// Requirements of one file, including same-file references. Placement decides
/// which need imports or prior declarations; discovery never guesses filenames.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CFileDependencies {
    file: CFileRef,
    headers: BTreeSet<CHeader>,
    libraries: BTreeSet<CSystemLibrary>,
    tags: BTreeMap<CTagDependency, CTypeRequirement>,
    aliases: BTreeSet<CTypedefRef>,
    functions: BTreeSet<CFunctionRef>,
    objects: BTreeSet<CObjectRef>,
}

impl CFileDependencies {
    fn new(file: CFileRef) -> Self {
        Self {
            file,
            headers: BTreeSet::new(),
            libraries: BTreeSet::new(),
            tags: BTreeMap::new(),
            aliases: BTreeSet::new(),
            functions: BTreeSet::new(),
            objects: BTreeSet::new(),
        }
    }

    pub fn file(&self) -> &CFileRef {
        &self.file
    }
    pub fn headers(&self) -> &BTreeSet<CHeader> {
        &self.headers
    }
    pub fn libraries(&self) -> &BTreeSet<CSystemLibrary> {
        &self.libraries
    }
    pub fn tags(&self) -> &BTreeMap<CTagDependency, CTypeRequirement> {
        &self.tags
    }
    pub fn aliases(&self) -> &BTreeSet<CTypedefRef> {
        &self.aliases
    }
    pub fn functions(&self) -> &BTreeSet<CFunctionRef> {
        &self.functions
    }
    pub fn objects(&self) -> &BTreeSet<CObjectRef> {
        &self.objects
    }

    fn tag(&mut self, tag: CTagDependency, requirement: CTypeRequirement) {
        self.tags
            .entry(tag)
            .and_modify(|old| *old = (*old).max(requirement))
            .or_insert(requirement);
    }
}

/// Authenticate the whole package before collecting references. No raw source,
/// external tool or caller-authored dependency list is consulted. This does NOT
/// establish flow, resource, visibility or final linkage validity.
pub fn file_dependencies(
    registry: &CFrozenRegistry,
    files: &[CSourceFile],
) -> Result<Vec<CFileDependencies>, CContextError> {
    registry.registrations().check_context(files)?;
    let mut result = files
        .iter()
        .map(|file| {
            let mut dependencies = CFileDependencies::new(file.identity().clone());
            for item in file.items() {
                dependencies.item(item);
            }
            dependencies
        })
        .collect::<Vec<_>>();
    result.sort_by(|left, right| left.file.key().cmp(right.file.key()));
    Ok(result)
}
