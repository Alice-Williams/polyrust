//! Language-owned linking hooks, independent of concrete symbol traversal.
use super::*;

pub trait LinkerDialect:
    TypedAstDialect + TargetDialect<Resolved = LinkedTargetPackage<Self>> + Clone + Ord
{
    type DependencyCallable: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type DependencyPackage: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type KnownField: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type KnownConstructor: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type KnownMethod: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type PreludeSymbol: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type StandardLibrary: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type ExternalPackage: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type PackageFeature: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type HelperId: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type HelperCapability: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type Identifier: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type QualifiedName: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type MemberName: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type Namespace: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type NameKey: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type ImportKind: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type ResolvedModule: Clone + std::fmt::Debug + Eq + Send + Sync;
    type ResolvedFileItem: Clone + std::fmt::Debug + Eq + Send + Sync;

    fn package_ecosystem(&self, package: &Self::ExternalPackage) -> PackageEcosystem;
    fn package_name(&self, package: &Self::ExternalPackage) -> &'static str;
    fn package_feature_name(&self, feature: &Self::PackageFeature) -> &'static str;
    fn helper_name(&self, helper: &Self::HelperId) -> &'static str;
    fn helper_capability_name(&self, capability: &Self::HelperCapability) -> &'static str;
    fn symbol_catalogue(&self) -> SymbolCatalogue<Self>;
    fn dependency_callable_spec(
        &self,
        callable: &Self::DependencyCallable,
    ) -> DependencyCallableSpec<Self>;
    /// Derive immutable dependency metadata from the checked original package.
    /// Post-link verification invokes this again; a linked catalogue is not
    /// independent authority. Existing static plugins need not override it.
    fn package_symbol_catalogue(
        &self,
        _package: &TargetAstPackage<Self>,
    ) -> Result<SymbolCatalogue<Self>, Vec<Diagnostic>> {
        Ok(self.symbol_catalogue())
    }
    fn identifier_from_candidate(
        &self,
        candidate: &str,
        namespace: &Self::Namespace,
    ) -> Result<Self::Identifier, AstViolation>;
    fn identifier_key(&self, identifier: &Self::Identifier) -> Self::NameKey;
    fn is_public(&self, visibility: &Self::Visibility) -> bool;
    fn type_namespace(&self, kind: &Self::DeclarationKind) -> Self::Namespace;
    fn type_namespace_from_known(&self, known: &Self::KnownType) -> Self::Namespace;
    fn callable_namespace(&self) -> Self::Namespace;
    fn member_namespace(&self) -> Self::Namespace;
    fn value_namespace(&self) -> Self::Namespace;
    fn known_call_expression(
        &self,
        callable: Self::KnownCallable,
        invocation: Self::InvocationKind,
        arguments: Vec<TargetExprId>,
    ) -> Self::Expression;
    fn known_constructor_expression(
        &self,
        constructor: Self::KnownConstructor,
        arguments: Vec<TargetExprId>,
    ) -> Self::Expression;
    fn known_method_expression(
        &self,
        method: Self::KnownMethod,
        receiver: TargetExprId,
        arguments: Vec<TargetExprId>,
    ) -> Self::Expression;
    fn expression_references(&self, expression: &Self::Expression) -> Vec<TargetSymbolRef<Self>>;
    fn statement_references(&self, statement: &Self::Statement) -> Vec<TargetSymbolRef<Self>>;
    fn file_item_roots(&self, item: &Self::FileItem) -> FileItemRoots<Self>;
    fn resolve_module(
        &self,
        module: &Self::ModuleDeclaration,
    ) -> Result<Self::ResolvedModule, AstViolation>;
    fn resolve_file_item(
        &self,
        package: &TargetAstPackage<Self>,
        item: &Self::FileItem,
        references: &ResolvedReferenceMap<Self>,
    ) -> Result<Self::ResolvedFileItem, AstViolation>;
    fn verify_resolved_file_item(&self, item: &Self::ResolvedFileItem) -> Vec<AstViolation>;

    /// Maps one checked cross-file dependency to a directive, if this dialect
    /// requires one. This does not allocate or alias generated symbol names.
    fn resolve_file_import(
        &self,
        _source: &TargetFile<Self>,
        _destination: &TargetFile<Self>,
    ) -> Result<Option<Self::ImportKind>, AstViolation> {
        Ok(None)
    }

    /// Verifies semantic constraints which are visible only after the linker
    /// has assembled a complete file, including injected runtime helpers.
    fn verify_resolved_file(
        &self,
        _file: &LinkedFile<Self>,
        _context: &crate::TargetAstContext<'_, Self>,
    ) -> Vec<AstViolation> {
        vec![]
    }

    fn permits_file_cycle(&self, _files: &[TargetFileId]) -> bool {
        false
    }

    fn forward_declarations(
        &self,
        _file: TargetFileId,
        _references: &[TargetSymbolRef<Self>],
    ) -> Vec<GeneratedSymbolId> {
        vec![]
    }
}
