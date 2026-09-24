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
mod result_layout;
mod result_profile;
mod result_types;
pub use result_types::{
    JavaDependencyResultAccessor, JavaDependencyResultConstructor, JavaDependencyResultFamily,
    JavaDependencyResultType, JavaResultTypeRole,
};
mod signatures;
mod source_bound;
mod source_constant_values;
mod source_types;
pub use descriptions::{JavaSourceDescription, JavaSourceDescriptionKind, JavaSourceTarget};
pub use signatures::{JavaDependencySignature, JavaDependencyType};

use super::JavaDialect;
use crate::ast::{JavaDeclaredPath, JavaMethodSignature, JavaPackage};
use portable_codegen::{RenderReadyPackage, RustDeclarationId, RustSourceOrigin};
use std::{collections::BTreeMap, sync::Arc};

struct Authority {
    package: RenderReadyPackage<JavaDialect>,
    root: RustDeclarationId,
    namespace: JavaPackage,
    dependencies: BTreeMap<u64, JavaDependencyPackage>,
    source_types: Option<Arc<portable_codegen::RustSourceTypes>>,
    result_families: BTreeMap<portable_codegen::GeneratedTypeId, Arc<result_layout::ResultLayout>>,
    result_types: BTreeMap<
        portable_codegen::GeneratedTypeId,
        (Arc<result_layout::ResultLayout>, JavaResultTypeRole),
    >,
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
    pub(crate) fn is_result_type(&self, id: portable_codegen::GeneratedTypeId) -> bool {
        self.0.result_types.contains_key(&id)
    }
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
    exported_signature: JavaDependencySignature,
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
    /// Descriptive producer-local signature; do not insert it into a consumer AST.
    pub fn declaration_signature(&self) -> &JavaMethodSignature {
        &self.signature
    }
    pub fn exported_signature(&self) -> &JavaDependencySignature {
        &self.exported_signature
    }
    pub fn call_height(&self) -> usize {
        self.call_height
    }
    pub fn source_signature(&self) -> Option<&portable_codegen::RustFunctionTypes> {
        self.owner
            .0
            .source_types
            .as_deref()?
            .functions()
            .get(&self.declaration())
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
        Self::from_certificate_with_results(package, &[])
    }
    /// Select complete original result families; publication still verifies the
    /// canonical owner inventory, admitted bodies and exact exported signatures.
    pub fn from_certificate_with_results(
        package: RenderReadyPackage<JavaDialect>,
        selections: &[super::JavaScalarResultTypes],
    ) -> Result<Self, String> {
        let inventory = inventory::collect_with_results(&package, selections)?;
        let source_types = source_types::read(&package);
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
            result_types: inventory
                .result_families
                .values()
                .flat_map(|layout| {
                    [
                        (
                            layout.types.interface,
                            (layout.clone(), JavaResultTypeRole::Interface),
                        ),
                        (
                            layout.types.success,
                            (layout.clone(), JavaResultTypeRole::Success),
                        ),
                        (
                            layout.types.error,
                            (layout.clone(), JavaResultTypeRole::Error),
                        ),
                    ]
                })
                .collect(),
            package,
            root: inventory.root,
            namespace: inventory.namespace,
            dependencies,
            source_types,
            result_families: inventory.result_families,
        }));
        let functions = inventory
            .functions
            .into_iter()
            .map(|(id, function)| {
                Ok((
                    id,
                    JavaDependencyFunction {
                        exported_signature: JavaDependencySignature::project(
                            &owner,
                            &function.signature,
                        )?,
                        owner: owner.clone(),
                        source: function.source,
                        path: function.path,
                        signature: function.signature,
                        call_height: function.call_height,
                    },
                ))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        let constants = inventory
            .constants
            .into_iter()
            .map(|(id, value)| (id, JavaDependencyConstant::new(owner.clone(), value)))
            .collect();
        let api = Self {
            owner,
            functions,
            constants,
            foreign_constants: inventory.foreign_constants,
        };
        source_types::verify(&api)?;
        if !selections.is_empty()
            || api
                .package()
                .ast()
                .files()
                .iter()
                .flat_map(|file| file.items())
                .any(|item| {
                    matches!(&item.item, crate::ast::JavaFileItem::Type { dependencies, .. }
                if dependencies.result_types().next().is_some())
                })
        {
            api.source_byte_bound()?;
        }
        Ok(api)
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
    pub fn source_types(&self) -> Option<&portable_codegen::RustSourceTypes> {
        self.owner.0.source_types.as_deref()
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
