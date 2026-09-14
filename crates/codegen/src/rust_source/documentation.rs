//! Bounded source metadata coherence, deliberately not compiler authority.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod budget;
mod graph;
use budget::{Budget, Limits};

#[cfg(test)]
#[path = "../tests/rust_documentation.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RustDocumentationError {
    Structure(&'static str),
    Budget(&'static str),
    DuplicateDeclaration,
}

impl std::fmt::Display for RustDocumentationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Structure(reason) => write!(formatter, "source documentation: {reason}"),
            Self::Budget(reason) => write!(formatter, "source documentation budget: {reason}"),
            Self::DuplicateDeclaration => {
                formatter.write_str("duplicate source documentation owner")
            }
        }
    }
}

impl std::error::Error for RustDocumentationError {}
type Result<T> = std::result::Result<T, RustDocumentationError>;

/// Coherent borrowed metadata, not a Rust or target-language certificate.
///
/// Private fields prevent bypassing validation. Returned references cannot mutate
/// the checked input while this inventory is in use.
///
/// ```compile_fail
/// use portable_codegen::CheckedRustDocumentation;
/// let bypass = CheckedRustDocumentation {
///     declarations: Default::default(), modules: Default::default(), exports: None,
/// };
/// ```
#[derive(Debug)]
pub struct CheckedRustDocumentation<'a> {
    declarations: BTreeMap<RustDeclarationId, &'a RustSourceOrigin>,
    modules: BTreeMap<RustDeclarationId, &'a RustModuleDocumentation>,
    exports: Option<&'a RustCrateExports>,
}

impl<'a> CheckedRustDocumentation<'a> {
    pub fn check(origins: impl IntoIterator<Item = &'a RustSourceOrigin>) -> Result<Self> {
        check(origins, Limits::PRODUCTION)
    }

    pub fn declarations(&self) -> impl Iterator<Item = &'a RustSourceOrigin> + '_ {
        self.declarations.values().copied()
    }

    pub fn modules(&self) -> impl Iterator<Item = &'a RustModuleDocumentation> + '_ {
        self.modules.values().copied()
    }

    pub fn crate_exports(&self) -> Option<&'a RustCrateExports> {
        self.exports
    }
}

struct Checking<'a> {
    checked: CheckedRustDocumentation<'a>,
    budget: Budget,
    export_allocations: BTreeSet<*const RustCrateExports>,
    exported_declarations: BTreeSet<RustDeclarationId>,
}

fn check<'a>(
    origins: impl IntoIterator<Item = &'a RustSourceOrigin>,
    limits: Limits,
) -> Result<CheckedRustDocumentation<'a>> {
    let mut checking = Checking {
        checked: CheckedRustDocumentation {
            declarations: BTreeMap::new(),
            modules: BTreeMap::new(),
            exports: None,
        },
        budget: Budget::new(limits),
        export_allocations: BTreeSet::new(),
        exported_declarations: BTreeSet::new(),
    };
    for origin in origins {
        checking.origin(origin)?;
    }
    if checking
        .checked
        .declarations
        .keys()
        .chain(checking.exported_declarations.iter())
        .any(|id| checking.checked.modules.contains_key(id))
    {
        return Err(RustDocumentationError::Structure(
            "one identity is both a module and a declaration",
        ));
    }
    Ok(checking.checked)
}

impl<'a> Checking<'a> {
    fn origin(&mut self, origin: &'a RustSourceOrigin) -> Result<()> {
        self.budget.declaration()?;
        if origin.node != RustSourceNode::Declaration {
            return Err(RustDocumentationError::Structure("body-local origin"));
        }
        let exports = origin.crate_exports.as_ref();
        if origin.declaration.crate_id != exports.root.crate_id
            || self
                .checked
                .exports
                .is_some_and(|prior| prior.root != exports.root)
        {
            return Err(RustDocumentationError::Structure("conflicting crate roots"));
        }
        if self
            .checked
            .declarations
            .insert(origin.declaration, origin)
            .is_some()
        {
            return Err(RustDocumentationError::DuplicateDeclaration);
        }
        self.budget.text(origin.location.file.len())?;
        self.budget.attributes(&origin.documentation)?;
        // Never compare unbounded distinct graphs. Shared allocations are immutable
        // for the lifetime of the inventory and need only one complete traversal.
        if self
            .export_allocations
            .insert(Arc::as_ptr(&origin.crate_exports))
        {
            self.exported_declarations
                .extend(graph::verify(exports, &mut self.budget)?);
            for (owner, ancestry) in &exports.module_ancestries {
                self.ancestry(exports.root, *owner, ancestry)?;
            }
            if self
                .checked
                .exports
                .is_some_and(|prior| !graph::same_inventory(prior, exports))
            {
                return Err(RustDocumentationError::Structure(
                    "conflicting export inventories",
                ));
            }
        }
        self.checked.exports = Some(exports);
        self.ancestry(exports.root, origin.module, &origin.module_ancestors)?;
        if let RustVisibility::RestrictedTo(scope) = origin.visibility
            && !origin
                .module_ancestors
                .iter()
                .any(|module| module.declaration == scope)
        {
            return Err(RustDocumentationError::Structure(
                "visibility scope is not an ancestor module",
            ));
        }
        Ok(())
    }

    fn ancestry(
        &mut self,
        root: RustDeclarationId,
        owner: RustDeclarationId,
        ancestry: &'a RustModuleAncestry,
    ) -> Result<()> {
        self.budget.ancestry(ancestry.len())?;
        if ancestry.first().map(|module| module.declaration) != Some(root) {
            return Err(RustDocumentationError::Structure(
                "ancestry omits the crate root",
            ));
        }
        let mut parent = None;
        let mut seen = BTreeSet::new();
        for module in ancestry.iter() {
            if module.declaration.crate_id != root.crate_id
                || module.parent != parent
                || !seen.insert(module.declaration)
            {
                return Err(RustDocumentationError::Structure("invalid module ancestry"));
            }
            parent = Some(module.declaration);
            if let Some(prior) = self.checked.modules.get(&module.declaration)
                && std::ptr::eq(*prior, module.as_ref())
            {
                continue;
            }
            self.budget.text(module.location.file.len())?;
            self.budget.attributes(&module.documentation)?;
            if let Some(prior) = self.checked.modules.get(&module.declaration) {
                if *prior != module.as_ref() {
                    return Err(RustDocumentationError::Structure(
                        "conflicting module documentation",
                    ));
                }
            } else {
                self.checked
                    .modules
                    .insert(module.declaration, module.as_ref());
            }
        }
        if parent != Some(owner) {
            return Err(RustDocumentationError::Structure(
                "ancestry omits its owning module",
            ));
        }
        Ok(())
    }
}
