//! Closed constructors/accessors bind their original owner into one consumer.
use super::{JavaDependencyBindings, JavaDependencyScope, JavaImportedResultType};
use crate::{
    ast::{JavaDeclaredPath, JavaIdentifier, JavaMethodSignature, JavaPrimitive, JavaType},
    dialect::{JavaDependencyResultAccessor, JavaDependencyResultConstructor, JavaResultTypeRole},
};
use portable_diagnostics::Diagnostic;

/// A registered constructor cannot be forged from a path or parameter list.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaImportedResultConstructor;
/// let forged = JavaImportedResultConstructor {};
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaImportedResultConstructor {
    original: JavaDependencyResultConstructor,
    owner: JavaImportedResultType,
}
impl JavaImportedResultConstructor {
    pub fn original(&self) -> &JavaDependencyResultConstructor {
        &self.original
    }
    pub fn owner(&self) -> &JavaImportedResultType {
        &self.owner
    }
    pub fn parameters(&self) -> &[JavaType] {
        match self.original.owner().role() {
            JavaResultTypeRole::Success => &[JavaType::Primitive(JavaPrimitive::Int)],
            JavaResultTypeRole::Error => &[],
            JavaResultTypeRole::Interface => unreachable!("private variant constructor"),
        }
    }
}

/// This member is the original success accessor, never a caller-named method.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaImportedResultAccessor;
/// let forged = JavaImportedResultAccessor {};
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaImportedResultAccessor(std::sync::Arc<ImportedAccessor>);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ImportedAccessor {
    original: JavaDependencyResultAccessor,
    owner: JavaImportedResultType,
    path: JavaDeclaredPath,
}
impl JavaImportedResultAccessor {
    pub fn original(&self) -> &JavaDependencyResultAccessor {
        &self.0.original
    }
    pub fn owner(&self) -> &JavaImportedResultType {
        &self.0.owner
    }
    pub fn name(&self) -> &JavaIdentifier {
        self.original().name()
    }
    pub fn path(&self) -> &JavaDeclaredPath {
        &self.0.path
    }
    pub fn signature(&self) -> JavaMethodSignature {
        JavaMethodSignature {
            receiver: Some(self.owner().ty()),
            parameters: vec![],
            result: JavaType::primitive(self.original().result()),
            checked_exceptions: vec![],
            nullable_result: false,
            pure: true,
        }
    }
}

impl JavaDependencyScope {
    pub fn import_result_constructor(
        self,
        original: JavaDependencyResultConstructor,
    ) -> Result<(Self, JavaImportedResultConstructor), Vec<Diagnostic>> {
        let (mut scope, owner) = self.import_result_type(original.owner().clone())?;
        let value = JavaImportedResultConstructor { original, owner };
        let added = scope.result_constructors.insert(value.clone());
        scope.verify_registration(
            value.owner().original().family().package_identity(),
            added.then(|| value.owner().path().encoded_len()),
        )?;
        Ok((scope, value))
    }
    pub fn import_result_accessor(
        self,
        original: JavaDependencyResultAccessor,
    ) -> Result<(Self, JavaImportedResultAccessor), Vec<Diagnostic>> {
        let (mut scope, owner) = self.import_result_type(original.owner().clone())?;
        let mut path = owner.path().clone();
        path.owners.push(path.member.clone());
        path.member = original.name().clone();
        let value = JavaImportedResultAccessor(std::sync::Arc::new(ImportedAccessor {
            original,
            owner,
            path,
        }));
        let added = scope.result_accessors.insert(value.clone());
        scope.verify_registration(
            value.owner().original().family().package_identity(),
            added.then(|| value.path().encoded_len()),
        )?;
        Ok((scope, value))
    }
}

impl JavaDependencyBindings {
    pub fn result_constructors(&self) -> impl Iterator<Item = &JavaImportedResultConstructor> {
        self.0.iter().flat_map(|frozen| &frozen.result_constructors)
    }
    pub fn result_accessors(&self) -> impl Iterator<Item = &JavaImportedResultAccessor> {
        self.0.iter().flat_map(|frozen| &frozen.result_accessors)
    }
    pub fn contains_result_constructor(&self, value: &JavaImportedResultConstructor) -> bool {
        self.contains_result_type(value.owner())
            && self
                .0
                .as_ref()
                .is_some_and(|frozen| frozen.result_constructors.contains(value))
    }
    pub fn contains_result_accessor(&self, value: &JavaImportedResultAccessor) -> bool {
        self.contains_result_type(value.owner())
            && self
                .0
                .as_ref()
                .is_some_and(|frozen| frozen.result_accessors.contains(value))
    }
}
