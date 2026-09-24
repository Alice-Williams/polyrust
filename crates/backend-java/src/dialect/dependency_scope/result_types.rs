//! Nominal imports carry original declaration authority and consumer membership.
use super::{Identity, JavaDependencyBindings, JavaDependencyScope};
use crate::{
    ast::{JavaDeclaredPath, JavaType, JavaTypeName},
    dialect::JavaDependencyResultType,
};
use portable_diagnostics::{Diagnostic, SourceRef};
use std::sync::Arc;

/// A foreign nominal registered in exactly one consuming scope.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaImportedResultType;
/// let forged = JavaImportedResultType {};
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaImportedResultType(Arc<ImportedType>);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ImportedType {
    scope: Identity,
    original: JavaDependencyResultType,
}

impl JavaImportedResultType {
    /// Only the original closed variants implement their own family's interface.
    /// Consumer scope is part of the relation, not merely a later name check.
    pub(crate) fn implements(&self, interface: &Self) -> bool {
        self.0.scope == interface.0.scope
            && self.original().family() == interface.original().family()
            && interface.original().role() == crate::dialect::JavaResultTypeRole::Interface
            && matches!(
                self.original().role(),
                crate::dialect::JavaResultTypeRole::Success
                    | crate::dialect::JavaResultTypeRole::Error
            )
    }
    pub(in crate::dialect) fn spec(
        &self,
    ) -> portable_codegen::KnownTypeSpec<crate::dialect::JavaDialect> {
        portable_codegen::KnownTypeSpec {
            symbol: self.clone().into(),
            name: self.path().member().clone(),
            alias_stem: self.path().member().as_str().to_owned(),
            qualified_name: Some(crate::dialect::JavaQualifiedName::Dependency(
                self.path().clone(),
            )),
            origin: portable_codegen::SymbolOrigin::CertifiedDependency(
                self.original().family().package_identity().clone(),
            ),
            arity: 0,
            policy: portable_codegen::DependencyPolicy::Qualified,
            dependency: None,
            source: SourceRef::logical([
                "java",
                "certified-result-type",
                self.path().member().as_str(),
            ]),
        }
    }
    pub fn original(&self) -> &JavaDependencyResultType {
        &self.0.original
    }
    pub fn path(&self) -> &JavaDeclaredPath {
        self.original().path()
    }
    pub fn ty(&self) -> JavaType {
        JavaType::Reference(JavaTypeName::Imported(self.clone()))
    }
}

impl JavaDependencyScope {
    /// Registration is consuming and atomic: failure returns no scope or handle.
    pub fn import_result_type(
        mut self,
        original: JavaDependencyResultType,
    ) -> Result<(Self, JavaImportedResultType), Vec<Diagnostic>> {
        let imported = JavaImportedResultType(Arc::new(ImportedType {
            scope: self.identity.clone(),
            original,
        }));
        let added = self.result_types.insert(imported.clone());
        self.verify_registration(
            imported.original().family().package_identity(),
            added.then(|| imported.path().encoded_len()),
        )?;
        Ok((self, imported))
    }
}

impl JavaDependencyBindings {
    pub fn result_types(&self) -> impl Iterator<Item = &JavaImportedResultType> {
        self.0.iter().flat_map(|frozen| &frozen.result_types)
    }
    pub fn contains_result_type(&self, value: &JavaImportedResultType) -> bool {
        self.0.as_ref().is_some_and(|frozen| {
            frozen.identity == value.0.scope && frozen.result_types.contains(value)
        })
    }
}
