//! A consuming registration scope binds certified functions and values to one consumer.
mod registration;
mod result_catalogue;
mod result_members;
mod result_types;
pub use result_members::{JavaImportedResultAccessor, JavaImportedResultConstructor};
mod signatures;
#[cfg(test)]
#[path = "../tests/dependency_scope.rs"]
mod tests;
mod values;
pub use result_types::JavaImportedResultType;
mod verification;
pub use values::JavaImportedValue;
pub(super) use verification::catalogue;

use super::JavaDependencyFunction;
use crate::ast::{JavaFileItem, JavaMethodSignature};
use portable_codegen::{AstViolation, TargetSymbolRef};
use portable_diagnostics::DiagnosticCode;
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone, Debug)]
struct Identity(Arc<u8>);

impl PartialEq for Identity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Identity {}
impl PartialOrd for Identity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Identity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Arc::as_ptr(&self.0).cmp(&Arc::as_ptr(&other.0))
    }
}

/// A function reference authenticated by both its owner and consumer scope.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaImportedCallable;
/// let forged = JavaImportedCallable {};
/// ```
#[derive(Clone, Debug)]
pub struct JavaImportedCallable {
    scope: Identity,
    function: Arc<JavaDependencyFunction>,
    signature: Arc<JavaMethodSignature>,
}

impl JavaImportedCallable {
    pub fn function(&self) -> &JavaDependencyFunction {
        &self.function
    }
    pub fn signature(&self) -> &JavaMethodSignature {
        &self.signature
    }
}

impl PartialEq for JavaImportedCallable {
    fn eq(&self, other: &Self) -> bool {
        self.scope == other.scope && self.function == other.function
    }
}
impl Eq for JavaImportedCallable {}
impl PartialOrd for JavaImportedCallable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for JavaImportedCallable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (&self.scope, &self.function).cmp(&(&other.scope, &other.function))
    }
}

/// Import registration is consuming; freezing prevents subsequent mutation.
///
/// ```compile_fail,E0382
/// use portable_backend_java::dialect::{JavaDependencyScope, JavaDependencyFunction};
/// fn frozen(function: JavaDependencyFunction) {
///     let scope = JavaDependencyScope::new();
///     let bindings = scope.finish();
///     let _ = scope.import(function);
/// }
/// ```
///
/// ```compile_fail
/// use portable_backend_java::dialect::{JavaDependencyScope, JavaDialect};
/// use portable_codegen::TargetAstPackage;
/// fn unchecked(package: TargetAstPackage<JavaDialect>) {
///     let _ = JavaDependencyScope::new().import(package);
/// }
/// ```
#[derive(Debug)]
pub struct JavaDependencyScope {
    identity: Identity,
    owners: std::collections::BTreeMap<u64, super::JavaDependencyPackage>,
    name_bytes: usize,
    functions: BTreeSet<JavaImportedCallable>,
    values: BTreeSet<JavaImportedValue>,
    result_types: BTreeSet<JavaImportedResultType>,
    result_constructors: BTreeSet<JavaImportedResultConstructor>,
    result_accessors: BTreeSet<JavaImportedResultAccessor>,
}

impl Default for JavaDependencyScope {
    fn default() -> Self {
        Self::new()
    }
}
impl JavaDependencyScope {
    pub fn new() -> Self {
        Self {
            identity: Identity(Arc::new(0)),
            owners: Default::default(),
            name_bytes: 0,
            functions: BTreeSet::new(),
            values: BTreeSet::new(),
            result_types: BTreeSet::new(),
            result_constructors: BTreeSet::new(),
            result_accessors: BTreeSet::new(),
        }
    }
    pub fn import(
        self,
        function: JavaDependencyFunction,
    ) -> Result<(Self, JavaImportedCallable), Vec<portable_diagnostics::Diagnostic>> {
        let (mut scope, signature) = self.bind_signature(function.exported_signature())?;
        let imported = JavaImportedCallable {
            scope: scope.identity.clone(),
            function: Arc::new(function),
            signature: Arc::new(signature),
        };
        let added = scope.functions.insert(imported.clone());
        scope.verify_registration(
            imported.function().package_identity(),
            added.then(|| imported.function().path().encoded_len()),
        )?;
        Ok((scope, imported))
    }
    pub fn finish(self) -> JavaDependencyBindings {
        if self.functions.is_empty() && self.values.is_empty() && self.result_types.is_empty() {
            JavaDependencyBindings::default()
        } else {
            JavaDependencyBindings(Some(Arc::new(Frozen {
                identity: self.identity,
                functions: self.functions,
                values: self.values,
                result_types: self.result_types,
                result_constructors: self.result_constructors,
                result_accessors: self.result_accessors,
            })))
        }
    }
}

#[derive(Debug)]
struct Frozen {
    identity: Identity,
    functions: BTreeSet<JavaImportedCallable>,
    values: BTreeSet<JavaImportedValue>,
    result_types: BTreeSet<JavaImportedResultType>,
    result_constructors: BTreeSet<JavaImportedResultConstructor>,
    result_accessors: BTreeSet<JavaImportedResultAccessor>,
}

/// Immutable original-package registrations. Default is the legacy empty scope.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaDependencyBindings;
/// let forged = JavaDependencyBindings(None);
/// ```
#[derive(Clone, Debug, Default)]
pub struct JavaDependencyBindings(Option<Arc<Frozen>>);

impl PartialEq for JavaDependencyBindings {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }
}
impl Eq for JavaDependencyBindings {}

impl JavaDependencyBindings {
    pub(crate) fn verify_item(&self, item: &JavaFileItem) -> Vec<AstViolation> {
        item.symbols()
            .into_iter()
            .filter_map(|symbol| {
                let message = match symbol {
                    TargetSymbolRef::KnownConstructor(super::JavaReferencedConstructor::Dependency(value))
                        if !self.contains_result_constructor(&value) => {
                        "Java dependency constructor is absent from its file's frozen consumer scope"
                    }
                    TargetSymbolRef::KnownMethod(super::JavaReferencedMethod::Dependency(value))
                        if !self.contains_result_accessor(&value) => {
                        "Java dependency accessor is absent from its file's frozen consumer scope"
                    }
                    TargetSymbolRef::KnownType(super::JavaReferencedType::Dependency(ty))
                        if !self.contains_result_type(&ty) =>
                    {
                        "Java dependency type is absent from its file's frozen consumer scope"
                    }
                    TargetSymbolRef::DependencyCallable(callable) if !self.contains(&callable) => {
                        "Java dependency call is absent from its file's frozen consumer scope"
                    }
                    TargetSymbolRef::DependencyValue(value) if !self.contains_value(&value) => {
                        "Java dependency value is absent from its file's frozen consumer scope"
                    }
                    _ => return None,
                };
                Some(AstViolation::new(
                    DiagnosticCode::UnresolvedReference,
                    message,
                ))
            })
            .collect()
    }
    pub(crate) fn owners(&self) -> impl Iterator<Item = &super::JavaDependencyPackage> {
        self.functions()
            .map(|f| f.function().package_identity())
            .chain(self.values().map(|v| v.constant().package_identity()))
            .chain(
                self.result_types()
                    .map(|value| value.original().family().package_identity()),
            )
    }
    pub fn functions(&self) -> impl Iterator<Item = &JavaImportedCallable> {
        self.0.iter().flat_map(|frozen| &frozen.functions)
    }
    pub fn contains(&self, callable: &JavaImportedCallable) -> bool {
        self.0.as_ref().is_some_and(|frozen| {
            frozen.identity == callable.scope && frozen.functions.contains(callable)
        })
    }
}
