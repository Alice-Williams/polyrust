//! Consumer value bindings derive every field from a registered certificate.
use super::{CDependencyConstant, CDialect, bindings::CBindings, violation};
use crate::ast::{CObjectRef, CRegistry};
use portable_codegen::{AstViolation, DependencySpelling, DependencyValueSpec, TargetTypeRef};
use portable_diagnostics::SourceRef;

/// A name, object or copied producer metadata cannot create an import.
///
/// ```compile_fail
/// use portable_backend_c::{ast::CObjectRef, dialect::{CImportedValue, CDependencyConstant}};
/// fn forge(object: CObjectRef, dependency: CDependencyConstant) -> CImportedValue {
///     CImportedValue { object, dependency }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CImportedValue {
    object: CObjectRef,
    dependency: CDependencyConstant,
}

impl CImportedValue {
    pub(super) fn from_registry(registry: &CRegistry, object: &CObjectRef) -> Result<Self, String> {
        Ok(Self {
            object: object.clone(),
            dependency: registry
                .imported_constant(object)
                .map_err(|error| error.to_string())?
                .clone(),
        })
    }
    pub fn object(&self) -> &CObjectRef {
        &self.object
    }
    pub fn dependency(&self) -> &CDependencyConstant {
        &self.dependency
    }

    pub(super) fn spec(&self) -> DependencyValueSpec<CDialect> {
        let owner = self.dependency.package_identity();
        DependencyValueSpec {
            symbol: self.clone(),
            owner: owner.clone(),
            name: self.dependency.symbol().clone(),
            ty: CBindings::default().ty(self.dependency.read_type()),
            spelling: DependencySpelling::FixedImport(super::CImportKind::Dependency(owner)),
            source: SourceRef::logical([
                "c",
                "certified-constant",
                self.dependency.symbol().as_str(),
            ]),
        }
    }

    pub(super) fn verify_type(&self, ty: &TargetTypeRef<CDialect>) -> Result<(), AstViolation> {
        if ty == &self.spec().ty {
            Ok(())
        } else {
            Err(violation(
                "C dependency value differs from its certified scalar read type",
            ))
        }
    }
}
