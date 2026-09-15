//! Constant imports authenticate both the defining certificate and consumer scope.
use super::{Identity, JavaDependencyBindings, JavaDependencyScope};
use crate::{
    ast::JavaType,
    dialect::{JavaDependencyConstant, JavaDialect, JavaQualifiedName},
};
use portable_codegen::{AstViolation, DependencySpelling, DependencyValueSpec, TargetTypeRef};
use portable_diagnostics::{DiagnosticCode, SourceRef};
use std::sync::Arc;

/// A read-only field reference registered in one frozen consumer scope.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaImportedValue;
/// let forged = JavaImportedValue {};
/// ```
///
/// ```compile_fail
/// use portable_backend_java::{ast::JavaDeclaredPath, dialect::JavaImportedValue};
/// fn promote(path: JavaDeclaredPath) -> JavaImportedValue { path.into() }
/// ```
///
/// ```compile_fail
/// use portable_backend_java::dialect::{JavaDependencyScope, JavaDialect};
/// use portable_codegen::TargetAstPackage;
/// fn unchecked(package: TargetAstPackage<JavaDialect>) {
///     let _ = JavaDependencyScope::new().import_constant(package);
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaImportedValue {
    scope: Identity,
    constant: Arc<JavaDependencyConstant>,
}
impl JavaImportedValue {
    pub fn constant(&self) -> &JavaDependencyConstant {
        &self.constant
    }
    pub fn ty(&self) -> &JavaType {
        self.constant.ty()
    }
    pub(in crate::dialect) fn spec(&self) -> DependencyValueSpec<JavaDialect> {
        DependencyValueSpec {
            symbol: self.clone(),
            owner: self.constant.package_identity().clone(),
            name: self.constant.path().member().clone(),
            ty: JavaDialect.registered_type(self.ty()),
            spelling: DependencySpelling::Qualified(JavaQualifiedName::Dependency(
                self.constant.path().clone(),
            )),
            source: SourceRef::logical([
                "java",
                "certified-constant",
                self.constant.path().member().as_str(),
            ]),
        }
    }
    pub(in crate::dialect) fn verify_type(
        &self,
        ty: &TargetTypeRef<JavaDialect>,
    ) -> Result<(), AstViolation> {
        if *ty == JavaDialect.registered_type(self.ty()) {
            Ok(())
        } else {
            Err(AstViolation::new(
                DiagnosticCode::TypeMismatch,
                "Java dependency constant type disagrees with its defining certificate",
            ))
        }
    }
}
impl JavaDependencyScope {
    pub fn import_constant(
        mut self,
        constant: JavaDependencyConstant,
    ) -> (Self, JavaImportedValue) {
        let value = JavaImportedValue {
            scope: self.identity.clone(),
            constant: Arc::new(constant),
        };
        self.values.insert(value.clone());
        (self, value)
    }
}
impl JavaDependencyBindings {
    pub fn values(&self) -> impl Iterator<Item = &JavaImportedValue> {
        self.0.iter().flat_map(|frozen| &frozen.values)
    }
    pub fn contains_value(&self, value: &JavaImportedValue) -> bool {
        self.0
            .as_ref()
            .is_some_and(|frozen| frozen.identity == value.scope && frozen.values.contains(value))
    }
}
