//! Java owner authority derives only from an immutable render-ready package.
mod bodies;
mod constants;
pub use constants::JavaDependencyConstant;
mod descriptions;
mod exports;
mod foreign;
mod inventory;
pub use foreign::JavaForeignConstantExport;
mod records;
mod source_bound;
pub use descriptions::{JavaSourceDescription, JavaSourceDescriptionKind, JavaSourceTarget};

use super::JavaDialect;
use crate::ast::{JavaDeclaredPath, JavaMethodSignature, JavaPackage};
use portable_codegen::{RenderReadyPackage, RustDeclarationId, RustSourceOrigin};
use std::{collections::BTreeMap, sync::Arc};

struct Authority {
    package: RenderReadyPackage<JavaDialect>,
    root: RustDeclarationId,
    namespace: JavaPackage,
    dependencies: BTreeMap<u64, JavaDependencyPackage>,
}

/// Opaque owner identity. Address order authenticates only; never render it.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaDependencyPackage;
/// let forged = JavaDependencyPackage {};
/// ```
#[derive(Clone)]
pub struct JavaDependencyPackage(Arc<Authority>);

impl std::fmt::Debug for JavaDependencyPackage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JavaDependencyPackage")
            .field("root", &self.0.root)
            .field("namespace", &self.0.namespace)
            .field("direct_dependencies", &self.0.dependencies.len())
            .finish_non_exhaustive()
    }
}

impl JavaDependencyPackage {
    pub fn root(&self) -> RustDeclarationId {
        self.0.root
    }
    pub fn namespace(&self) -> JavaPackage {
        self.0.namespace
    }
    pub fn package(&self) -> &RenderReadyPackage<JavaDialect> {
        &self.0.package
    }
    pub fn dependencies(&self) -> impl ExactSizeIterator<Item = &JavaDependencyPackage> {
        self.0.dependencies.values()
    }
}
impl PartialEq for JavaDependencyPackage {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for JavaDependencyPackage {}
impl PartialOrd for JavaDependencyPackage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for JavaDependencyPackage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0.root, Arc::as_ptr(&self.0)).cmp(&(other.0.root, Arc::as_ptr(&other.0)))
    }
}

/// One public scalar function of an exact certified owner, not a name lookup.
/// Clones share authority; independent certifications are distinct proofs.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaDependencyFunction;
/// let forged = JavaDependencyFunction {};
/// ```
#[derive(Clone)]
pub struct JavaDependencyFunction {
    owner: JavaDependencyPackage,
    source: Arc<RustSourceOrigin>,
    path: JavaDeclaredPath,
    signature: JavaMethodSignature,
    call_height: usize,
}

impl std::fmt::Debug for JavaDependencyFunction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("JavaDependencyFunction")
            .field("declaration", &self.declaration())
            .field("path", &self.path)
            .field("signature", &self.signature)
            .field("call_height", &self.call_height)
            .finish_non_exhaustive()
    }
}

impl JavaDependencyFunction {
    pub fn package_identity(&self) -> &JavaDependencyPackage {
        &self.owner
    }
    pub fn declaration(&self) -> RustDeclarationId {
        self.source.declaration
    }
    pub fn source(&self) -> &RustSourceOrigin {
        &self.source
    }
    pub fn path(&self) -> &JavaDeclaredPath {
        &self.path
    }
    pub fn signature(&self) -> &JavaMethodSignature {
        &self.signature
    }
    pub fn call_height(&self) -> usize {
        self.call_height
    }
}
impl PartialEq for JavaDependencyFunction {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner && self.declaration() == other.declaration()
    }
}
impl Eq for JavaDependencyFunction {}
impl PartialOrd for JavaDependencyFunction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for JavaDependencyFunction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.declaration(), &self.owner).cmp(&(other.declaration(), &other.owner))
    }
}

/// An immutable API backed by its exact Java certificate. Source metadata is
/// descriptive: rustc source/metadata agreement remains the compiler's job.
///
/// ```compile_fail
/// use portable_backend_java::dialect::JavaDependencyApi;
/// let forged = JavaDependencyApi {};
/// ```
///
/// ```compile_fail
/// use portable_backend_java::dialect::{JavaDependencyApi, JavaDialect};
/// use portable_codegen::TargetAstPackage;
/// fn unchecked(package: TargetAstPackage<JavaDialect>) {
///     let _ = JavaDependencyApi::from_certificate(package);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct JavaDependencyApi {
    owner: JavaDependencyPackage,
    functions: BTreeMap<RustDeclarationId, JavaDependencyFunction>,
    constants: BTreeMap<RustDeclarationId, JavaDependencyConstant>,
    foreign_constants: Vec<JavaForeignConstantExport>,
}

impl JavaDependencyApi {
    /// Borrow descriptive source facts, including private declarations.
    /// These values are not callable handles and grant no construction authority.
    pub fn source_descriptions(&self) -> Result<Vec<JavaSourceDescription<'_>>, String> {
        descriptions::collect(self)
    }

    /// Conservative final UTF-8 source reservation for this closed owner only.
    /// Does not render, reserve temporary renderer memory, or measure bytecode.
    pub fn source_byte_bound(&self) -> Result<u64, String> {
        source_bound::measure(self)
    }

    pub fn from_certificate(package: RenderReadyPackage<JavaDialect>) -> Result<Self, String> {
        let inventory = inventory::collect(&package)?;
        let dependencies = package
            .ast()
            .files()
            .iter()
            .flat_map(|file| file.items())
            .filter_map(|item| match &item.item {
                crate::ast::JavaFileItem::Type { dependencies, .. } => Some(dependencies),
                _ => None,
            })
            .flat_map(|dependencies| dependencies.owners())
            .map(|owner| (owner.root().crate_id, owner.clone()))
            .collect();
        let owner = JavaDependencyPackage(Arc::new(Authority {
            package,
            root: inventory.root,
            namespace: inventory.namespace,
            dependencies,
        }));
        let functions = inventory
            .functions
            .into_iter()
            .map(|(id, function)| {
                (
                    id,
                    JavaDependencyFunction {
                        owner: owner.clone(),
                        source: function.source,
                        path: function.path,
                        signature: function.signature,
                        call_height: function.call_height,
                    },
                )
            })
            .collect();
        let constants = inventory
            .constants
            .into_iter()
            .map(|(id, value)| (id, JavaDependencyConstant::new(owner.clone(), value)))
            .collect();
        Ok(Self {
            owner,
            functions,
            constants,
            foreign_constants: inventory.foreign_constants,
        })
    }
    pub fn root(&self) -> RustDeclarationId {
        self.owner.root()
    }
    pub fn package_identity(&self) -> &JavaDependencyPackage {
        &self.owner
    }
    pub fn package(&self) -> &RenderReadyPackage<JavaDialect> {
        &self.owner.0.package
    }
    pub fn foreign_constants(&self) -> impl ExactSizeIterator<Item = &JavaForeignConstantExport> {
        self.foreign_constants.iter()
    }
    pub fn constants(&self) -> impl ExactSizeIterator<Item = &JavaDependencyConstant> {
        self.constants.values()
    }
    pub fn constant(&self, id: RustDeclarationId) -> Option<&JavaDependencyConstant> {
        self.constants.get(&id)
    }
    pub fn functions(&self) -> impl ExactSizeIterator<Item = &JavaDependencyFunction> {
        self.functions.values()
    }
    pub fn dependencies(&self) -> impl ExactSizeIterator<Item = &JavaDependencyPackage> {
        self.owner.dependencies()
    }
    pub fn function(&self, id: RustDeclarationId) -> Option<&JavaDependencyFunction> {
        self.functions.get(&id)
    }
}
