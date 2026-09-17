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
    CContextError, CEnumRef, CFileItem, CFileRef, CFrozenRegistry, CFunctionRef, CObjectRef,
    CScalarType, CSourceFile, CStructRef, CTypedefRef, CUnionRef,
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
    scalars: BTreeSet<CScalarType>,
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
            scalars: BTreeSet::new(),
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
    pub(crate) fn scalars(&self) -> &BTreeSet<CScalarType> {
        &self.scalars
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

/// Structural discovery after the caller's profile/context gate. Platform
/// assertions are excluded so an assertion cannot authorize its own presence.
pub(crate) fn source_scalars(files: &[CSourceFile]) -> BTreeSet<CScalarType> {
    let mut scalars = BTreeSet::new();
    for file in files {
        let mut dependencies = CFileDependencies::new(file.identity().clone());
        for item in file.items() {
            if !matches!(item, CFileItem::StaticAssert(_)) {
                dependencies.item(item);
            }
        }
        scalars.extend(dependencies.scalars);
    }
    scalars
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

/// Structural object references only; package-provenance export roots are not
/// source items. The certificate-only public view calls this after admission.
pub(crate) fn object_references(file: &CSourceFile) -> BTreeSet<CObjectRef> {
    let mut dependencies = CFileDependencies::new(file.identity().clone());
    for item in file.items() {
        dependencies.item(item);
    }
    dependencies.objects
}
