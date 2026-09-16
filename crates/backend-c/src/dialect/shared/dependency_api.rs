//! Certificate-derived dependency evidence, separate from consumer registration.
mod constants;
mod foreign;
pub use foreign::CForeignConstantExport;
mod inventory;
use super::{CDialect, CGeneratedHeader, resources};
use crate::ast::{CFileRef, CFunctionRef, CFunctionType, CIdentifier};
pub use constants::CDependencyConstant;
use portable_codegen::{RenderReadyPackage, RustDeclarationId};
use std::{collections::BTreeMap, sync::Arc};

#[cfg(test)]
#[path = "../../tests/shared_dependency_api.rs"]
mod tests;

#[derive(Clone, Debug)]
struct Authority {
    package: RenderReadyPackage<CDialect>,
    root: RustDeclarationId,
    header: CGeneratedHeader,
    implementation: CFileRef,
    stack_bound_bytes: u64,
}

/// Opaque certificate identity retained inside imported callable references.
/// Address ordering authenticates only; it must never determine emitted names.
#[derive(Clone, Debug)]
pub(crate) struct CDependencyAuthority(Arc<Authority>);

impl PartialEq for CDependencyAuthority {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for CDependencyAuthority {}
impl PartialOrd for CDependencyAuthority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CDependencyAuthority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Arc::as_ptr(&self.0).cmp(&Arc::as_ptr(&other.0))
    }
}

/// Exact package certificate identity, separate from package-manager libraries.
/// Stable source identity orders valid distinct packages; ephemeral authority
/// distinguishes certificates only and is never rendered or serialized.
///
/// ```compile_fail
/// use portable_backend_c::dialect::CDependencyPackage;
/// fn forge() { let _ = CDependencyPackage {}; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CDependencyPackage {
    root: RustDeclarationId,
    header: CGeneratedHeader,
    authority: CDependencyAuthority,
}

impl CDependencyPackage {
    pub(super) fn certificate(&self) -> &RenderReadyPackage<CDialect> {
        &self.authority.0.package
    }

    pub(super) fn stack_bound_bytes(&self) -> u64 {
        self.authority.0.stack_bound_bytes
    }

    pub(super) fn public_symbols(&self) -> impl Iterator<Item = &CIdentifier> {
        super::c_defined_functions(&self.authority.0.package)
            .filter(|definition| definition.linkage() == crate::ast::CLinkage::External)
            .map(|definition| definition.name())
            .chain(
                super::c_defined_constants(&self.authority.0.package)
                    .map(|definition| definition.name()),
            )
    }

    pub fn root(&self) -> RustDeclarationId {
        self.root
    }
    pub fn public_header(&self) -> &CGeneratedHeader {
        &self.header
    }
}

/// An exact public scalar function from an independently certified C package.
/// This cannot be constructed from a name, signature, JSON or caller frame bound.
/// Consumer calls require explicit registration of this witness.
///
/// ```compile_fail
/// use portable_backend_c::dialect::CDependencyFunction;
/// fn forge() { let _ = CDependencyFunction {}; }
/// ```
#[derive(Clone, Debug)]
pub struct CDependencyFunction {
    authority: Arc<Authority>,
    declaration: RustDeclarationId,
    function: CFunctionRef,
    symbol: CIdentifier,
}

impl CDependencyFunction {
    pub fn package_identity(&self) -> CDependencyPackage {
        CDependencyPackage {
            root: self.authority.root,
            header: self.authority.header.clone(),
            authority: self.authority(),
        }
    }
    pub(crate) fn authority(&self) -> CDependencyAuthority {
        CDependencyAuthority(self.authority.clone())
    }

    #[cfg(test)]
    pub(crate) fn shares_certificate(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.authority, &other.authority)
    }

    pub fn declaration(&self) -> RustDeclarationId {
        self.declaration
    }

    pub fn function(&self) -> &CFunctionRef {
        &self.function
    }

    pub fn signature(&self) -> &CFunctionType {
        self.function.signature()
    }

    pub fn symbol(&self) -> &CIdentifier {
        &self.symbol
    }

    pub fn public_header(&self) -> &CGeneratedHeader {
        &self.authority.header
    }

    pub fn implementation(&self) -> &CFileRef {
        &self.authority.implementation
    }

    /// Conservative whole-package maximum, not a zero-cost leaf or exact frame.
    /// Includes the existing closed direct-call path and pinned platform policy.
    pub fn stack_bound_bytes(&self) -> u64 {
        self.authority.stack_bound_bytes
    }
}

// Certificate identity is ephemeral authority, not a generated name or hash.
// Two independently certified bodies may share a registry and declaration ID;
// comparing only their function references would silently conflate the proofs.
impl PartialEq for CDependencyFunction {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.authority, &other.authority)
            && self.declaration == other.declaration
            && self.function == other.function
            && self.symbol == other.symbol
    }
}
impl Eq for CDependencyFunction {}

impl PartialOrd for CDependencyFunction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CDependencyFunction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            &self.declaration,
            &self.function,
            &self.symbol,
            self.authority(),
        )
            .cmp(&(
                &other.declaration,
                &other.function,
                &other.symbol,
                other.authority(),
            ))
    }
}

/// Certificate-backed public lookup. Metadata consistency is NOT rustc analysis:
/// the compiler driver must separately authenticate source/metadata agreement.
///
/// ```compile_fail
/// use portable_backend_c::dialect::CDependencyApi;
/// fn forge() { let _ = CDependencyApi {}; }
/// ```
///
/// ```compile_fail
/// use portable_backend_c::dialect::{CDialect, CDependencyApi};
/// use portable_codegen::TargetAstPackage;
/// fn unchecked(package: TargetAstPackage<CDialect>) {
///     let _ = CDependencyApi::from_certificate(package);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct CDependencyApi {
    authority: Arc<Authority>,
    functions: BTreeMap<RustDeclarationId, CDependencyFunction>,
    constants: BTreeMap<RustDeclarationId, CDependencyConstant>,
    foreign_constants: Vec<CForeignConstantExport>,
}

impl CDependencyApi {
    pub fn from_certificate(package: RenderReadyPackage<CDialect>) -> Result<Self, String> {
        let inventory = inventory::collect(&package)?;
        let measured = resources::measure_package(package.ast())?;
        let authority = Arc::new(Authority {
            package,
            root: inventory.root,
            header: inventory.header,
            implementation: inventory.implementation,
            stack_bound_bytes: measured.total.frame_bound,
        });
        let functions = inventory
            .functions
            .into_iter()
            .map(|(declaration, (function, symbol))| {
                (
                    declaration,
                    CDependencyFunction {
                        authority: authority.clone(),
                        declaration,
                        function,
                        symbol,
                    },
                )
            })
            .collect();
        let constants = inventory
            .constants
            .into_iter()
            .map(|(declaration, constant)| {
                (
                    declaration,
                    CDependencyConstant::new(authority.clone(), declaration, constant),
                )
            })
            .collect();
        Ok(Self {
            authority,
            functions,
            constants,
            foreign_constants: inventory.foreign_constants,
        })
    }

    pub fn root(&self) -> RustDeclarationId {
        self.authority.root
    }

    pub fn public_header(&self) -> &CGeneratedHeader {
        &self.authority.header
    }

    /// The independently certified owning package, never a consumer projection.
    /// Reading it does not manufacture dependency witnesses for private functions.
    pub fn package(&self) -> &RenderReadyPackage<CDialect> {
        &self.authority.package
    }

    pub fn functions(&self) -> impl Iterator<Item = &CDependencyFunction> {
        self.functions.values()
    }

    pub fn function(&self, declaration: RustDeclarationId) -> Option<&CDependencyFunction> {
        self.functions.get(&declaration)
    }

    /// Public aliases retain original defining witnesses, separate from owned constants.
    pub fn foreign_constants(&self) -> impl Iterator<Item = &CForeignConstantExport> {
        self.foreign_constants.iter()
    }

    pub fn constants(&self) -> impl Iterator<Item = &CDependencyConstant> {
        self.constants.values()
    }

    pub fn constant(&self, declaration: RustDeclarationId) -> Option<&CDependencyConstant> {
        self.constants.get(&declaration)
    }
}
