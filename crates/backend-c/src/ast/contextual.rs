//! Independent target-tree checking; successful local checks cannot render.

pub(crate) mod access_statements;
pub(crate) mod access_walk;
mod case_constants;
mod complete_objects;
mod constant_leaves;
mod errors;
mod flow_graph;
mod initialization;
mod initialized_paths;
mod lexical;
mod origin_roles;
mod package_inventory;
mod rebuild_files;
mod rebuild_initializers;
mod rebuild_places;
mod rebuild_statements;
mod rebuild_values;

pub use errors::CContextError;

use super::{CExpressions, CRegistry, CSourceFile};

struct Recheck<'a> {
    registry: &'a CRegistry,
    expressions: CExpressions<'a>,
}

impl<'a> Recheck<'a> {
    fn new(registry: &'a CRegistry) -> Self {
        Self {
            registry,
            expressions: CExpressions::new(registry),
        }
    }
}

fn exact<T: PartialEq>(actual: &T, rebuilt: T) -> Result<T, CContextError> {
    if actual == &rebuilt {
        Ok(rebuilt)
    } else {
        Err(CContextError::StoredStructureMismatch)
    }
}

impl CRegistry {
    /// Checks local, package, lexical and definite-initialization/return
    /// relations. Ownership, arithmetic, resources and linking remain separate
    /// mandatory obligations. Success is not a render-ready certificate.
    pub fn check_context(&self, files: &[CSourceFile]) -> Result<(), CContextError> {
        self.check_lexical_structure(files)?;
        initialization::check(self, files)
    }

    /// Adds lexical visibility and control-target checks, but not definite
    /// initialization, ownership, range or a rendering certificate.
    pub fn check_lexical_structure(&self, files: &[CSourceFile]) -> Result<(), CContextError> {
        self.check_package_structure(files)?;
        lexical::check(self, files)
    }

    /// Diagnostic package structure checks. Does not prove execution safety,
    /// scope/flow, name resolution, resource bounds or source validity.
    pub fn check_package_structure(&self, files: &[CSourceFile]) -> Result<(), CContextError> {
        self.check_local_structure(files)?;
        package_inventory::check(self, files)?;
        complete_objects::check(self, files)
    }

    /// Reconstruct all local type/shape relations from actual stored children.
    ///
    /// This is NOT scope, initialization, ownership, range, linking or rendering
    /// proof. It returns no certificate. The complete dialect checker must
    /// compose this pass with every contextual and safety obligation.
    pub fn check_local_structure(&self, files: &[CSourceFile]) -> Result<(), CContextError> {
        let checker = Recheck::new(self);
        for file in files {
            checker.file(file)?;
        }
        Ok(())
    }
}
