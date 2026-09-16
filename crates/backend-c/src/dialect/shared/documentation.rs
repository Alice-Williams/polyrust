//! Normalize documentation into exact owner attachments before certification.
use crate::ast::{
    CAggregateRef, CComment, CDeclarationKey, CDeclarationKind, CFileItem, CFileRef, CFunctionRef,
    CGeneratedOrigin, CMemberRef, CObjectRef, CRegistry, CSourceFile, CSourcePackage, CStructRef,
};
use portable_codegen::{
    RustCrateExports, RustDeclarationId, RustModuleAncestry, RustModuleDocumentation,
    RustSourceNode,
};
mod exports;

pub(super) fn check_export_graph(
    exports: &portable_codegen::RustCrateExports,
) -> Result<(), String> {
    exports::graph(exports)
}
mod routing;
use routing::FilePolicy;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[cfg(test)]
#[path = "../../tests/shared_package_documentation.rs"]
mod package_tests;
#[cfg(test)]
#[path = "../../tests/shared_documentation.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum CDocumentationOwner {
    Object(CObjectRef),
    Module(CFileRef, RustDeclarationId),
    Struct(CStructRef),
    Member(CMemberRef),
    Function(CFunctionRef),
}

impl CDocumentationOwner {
    fn file(&self) -> &CFileRef {
        match self {
            Self::Object(object) => object.file(),
            Self::Module(file, _) => file,
            Self::Struct(record) => record.file(),
            Self::Function(function) => function.file(),
            Self::Member(member) => match member.owner() {
                CAggregateRef::Struct(record) => record.file(),
                CAggregateRef::Union(record) => record.file(),
            },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CDocumentation {
    attachments: BTreeMap<CDocumentationOwner, Vec<CComment>>,
    module_order: Vec<CDocumentationOwner>,
}

impl CDocumentation {
    pub fn comments(&self, owner: &CDocumentationOwner) -> &[CComment] {
        self.attachments.get(owner).map_or(&[], Vec::as_slice)
    }

    pub fn modules(&self) -> impl Iterator<Item = &CComment> {
        self.module_order
            .iter()
            .flat_map(|owner| self.comments(owner))
    }

    pub fn all(&self) -> impl Iterator<Item = &CComment> {
        self.attachments.values().flatten()
    }
}

struct Lowering<'a> {
    policy: FilePolicy<'a>,
    modules: BTreeMap<RustDeclarationId, Arc<RustModuleDocumentation>>,
    declarations: BTreeMap<RustDeclarationId, CDocumentationOwner>,
    roots: BTreeMap<u64, RustDeclarationId>,
    exports: BTreeMap<u64, Arc<RustCrateExports>>,
    documentation: CDocumentation,
    raw_bytes: usize,
    attributes: usize,
    module_order: Vec<(usize, CDocumentationOwner)>,
}

#[cfg(test)]
pub(super) fn lower_package(
    sources: &[CSourceFile],
) -> Result<BTreeMap<CFileRef, CDocumentation>, String> {
    lower(sources, None)
}

pub(super) fn lower_registered_package(
    registry: &CRegistry,
    sources: &[CSourceFile],
) -> Result<BTreeMap<CFileRef, CDocumentation>, String> {
    super::source_package::check(registry, sources)?;
    lower(sources, registry.source_package())
}

fn lower(
    sources: &[CSourceFile],
    package: Option<&CSourcePackage>,
) -> Result<BTreeMap<CFileRef, CDocumentation>, String> {
    let mut lowering = Lowering {
        policy: match package {
            Some(package) => FilePolicy::explicit(package, sources)?,
            None => FilePolicy::new(sources)?,
        },
        modules: BTreeMap::new(),
        declarations: BTreeMap::new(),
        roots: BTreeMap::new(),
        exports: BTreeMap::new(),
        documentation: CDocumentation::default(),
        raw_bytes: 0,
        attributes: 0,
        module_order: Vec::new(),
    };
    if let Some(package) = package {
        lowering.package(package.exports())?;
    }
    // The closed profile has one complete record declaration and one prototype
    // per function. Uses and definitions do not duplicate primary documentation.
    for item in sources.iter().flat_map(|source| source.items()) {
        let CFileItem::Declaration(declaration) = item else {
            continue;
        };
        match declaration.kind() {
            CDeclarationKind::Aggregate {
                owner: CAggregateRef::Struct(owner),
                members,
            } => {
                lowering.owner(CDocumentationOwner::Struct(owner.clone()), owner.key())?;
                for member in members {
                    lowering.owner(CDocumentationOwner::Member(member.clone()), member.key())?;
                }
            }
            CDeclarationKind::FunctionPrototype { function, .. } => {
                lowering.owner(
                    CDocumentationOwner::Function(function.clone()),
                    function.key(),
                )?;
            }
            CDeclarationKind::ObjectDeclaration(object) => {
                lowering.owner(CDocumentationOwner::Object(object.clone()), object.key())?;
            }
            _ => return Err("documentation owner is outside the checked C profile".into()),
        }
    }
    lowering.module_order.sort();
    let mut files: BTreeMap<_, _> = sources
        .iter()
        .map(|source| (source.identity().clone(), CDocumentation::default()))
        .collect();
    for (owner, comments) in lowering.documentation.attachments {
        files
            .get_mut(owner.file())
            .ok_or("documentation owner has no package file")?
            .attachments
            .insert(owner, comments);
    }
    for (_, owner) in lowering.module_order {
        files
            .get_mut(owner.file())
            .ok_or("module documentation has no package file")?
            .module_order
            .push(owner);
    }
    Ok(files)
}

impl Lowering<'_> {
    fn package(&mut self, exports: &Arc<RustCrateExports>) -> Result<(), String> {
        exports::graph(exports)?;
        for (module, ancestry) in &exports.module_ancestries {
            self.ancestry(exports, *module, ancestry)?;
        }
        self.exports.insert(exports.root.crate_id, exports.clone());
        Ok(())
    }

    fn owner(&mut self, owner: CDocumentationOwner, key: &CDeclarationKey) -> Result<(), String> {
        let CGeneratedOrigin::RustSource(origin) = &key.origin else {
            return Ok(());
        };
        if origin.node != RustSourceNode::Declaration {
            return Err("documented declaration has body-local source provenance".into());
        }
        exports::root(origin)?;
        if let Some(previous) = self.exports.get(&origin.declaration.crate_id) {
            if !Arc::ptr_eq(previous, &origin.crate_exports) && previous != &origin.crate_exports {
                return Err(
                    "conflicting compiler export metadata across documentation owners".into(),
                );
            }
        } else {
            if matches!(self.policy, FilePolicy::PublicPair { .. }) {
                exports::graph(&origin.crate_exports)?;
                for (module, ancestry) in &origin.crate_exports.module_ancestries {
                    self.ancestry(&origin.crate_exports, *module, ancestry)?;
                }
            }
            self.exports
                .insert(origin.declaration.crate_id, origin.crate_exports.clone());
        }
        if self.modules.contains_key(&origin.declaration) {
            return Err("source identity is both module and declaration".into());
        }
        if let Some(previous) = self.declarations.insert(origin.declaration, owner.clone())
            && previous != owner
        {
            return Err("source declaration maps to conflicting documentation owners".into());
        }
        self.ancestry(
            &origin.crate_exports,
            origin.module,
            &origin.module_ancestors,
        )?;
        self.attach(owner, &origin.documentation)
    }

    fn ancestry(
        &mut self,
        exports: &RustCrateExports,
        owner: RustDeclarationId,
        ancestry: &RustModuleAncestry,
    ) -> Result<(), String> {
        if ancestry.first().map(|module| module.declaration) != Some(exports.root) {
            return Err("export module documentation disagrees with the crate root".into());
        }
        let mut parent = None;
        let mut seen = BTreeSet::new();
        for (depth, module) in ancestry.iter().enumerate() {
            if depth == 0
                && let Some(root) = self.roots.insert(exports.root.crate_id, module.declaration)
                && root != module.declaration
            {
                return Err("one source crate has conflicting root modules".into());
            }
            if module.parent != parent
                || module.declaration.crate_id != exports.root.crate_id
                || !seen.insert(module.declaration)
                || self.declarations.contains_key(&module.declaration)
            {
                return Err("invalid documentation module ancestry".into());
            }
            parent = Some(module.declaration);
            if let Some(previous) = self.modules.get(&module.declaration) {
                if !Arc::ptr_eq(previous, module) && previous != module {
                    return Err("conflicting documentation for one source module".into());
                }
            } else {
                let file = self.policy.module(exports, module).clone();
                self.module_order.push((
                    depth,
                    CDocumentationOwner::Module(file.clone(), module.declaration),
                ));
                self.attach(
                    CDocumentationOwner::Module(file, module.declaration),
                    &module.documentation,
                )?;
                self.modules.insert(module.declaration, module.clone());
            }
        }
        if parent != Some(owner) {
            return Err("documentation ancestry omits the owning source module".into());
        }
        Ok(())
    }

    fn attach(&mut self, owner: CDocumentationOwner, attributes: &[String]) -> Result<(), String> {
        let mut comments = Vec::new();
        for attribute in attributes {
            self.raw_bytes = self
                .raw_bytes
                .checked_add(attribute.len())
                .ok_or("documentation input accounting overflow")?;
            self.attributes = self
                .attributes
                .checked_add(1)
                .ok_or("documentation attribute accounting overflow")?;
            // Preliminary allocation guard, not permission to mint a certificate.
            // The existing smaller node/comment/source policies count normalized
            // attachments together with all other output before certification.
            if self.raw_bytes > 16 * 1024 * 1024 || self.attributes > 100_000 {
                return Err("documentation projection verifier budget exceeded".into());
            }
            comments.push(CComment::new(attribute));
        }
        if !comments.is_empty()
            && self
                .documentation
                .attachments
                .insert(owner, comments)
                .is_some()
        {
            return Err("duplicate primary documentation owner".into());
        }
        Ok(())
    }
}
