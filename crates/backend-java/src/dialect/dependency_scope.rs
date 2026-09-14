//! A consuming registration scope binds certified functions to one consumer.
#[cfg(test)]
#[path = "../tests/dependency_scope.rs"]
mod tests;
mod verification;
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct JavaImportedCallable {
    scope: Identity,
    function: Arc<JavaDependencyFunction>,
}

impl JavaImportedCallable {
    pub fn function(&self) -> &JavaDependencyFunction {
        &self.function
    }
    pub fn signature(&self) -> &JavaMethodSignature {
        self.function.signature()
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
    functions: BTreeSet<JavaImportedCallable>,
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
            functions: BTreeSet::new(),
        }
    }
    pub fn import(mut self, function: JavaDependencyFunction) -> (Self, JavaImportedCallable) {
        let imported = JavaImportedCallable {
            scope: self.identity.clone(),
            function: Arc::new(function),
        };
        self.functions.insert(imported.clone());
        (self, imported)
    }
    pub fn finish(self) -> JavaDependencyBindings {
        if self.functions.is_empty() {
            JavaDependencyBindings::default()
        } else {
            JavaDependencyBindings(Some(Arc::new(Frozen {
                identity: self.identity,
                functions: self.functions,
            })))
        }
    }
}

#[derive(Debug)]
struct Frozen {
    identity: Identity,
    functions: BTreeSet<JavaImportedCallable>,
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
                if let TargetSymbolRef::DependencyCallable(callable) = symbol
                    && !self.contains(&callable)
                {
                    Some(AstViolation::new(
                        DiagnosticCode::UnresolvedReference,
                        "Java dependency call is absent from its file's frozen consumer scope",
                    ))
                } else {
                    None
                }
            })
            .collect()
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
