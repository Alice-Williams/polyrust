use std::collections::{BTreeMap, BTreeSet};

use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef, sort_diagnostics};

use crate::{
    AstViolation, DeclaredDependency, Expr, GeneratedCallableId, GeneratedInterfaceMethodId,
    GeneratedTypeId, GeneratedValueId, InjectedHelper, PackageEcosystem, SourceRole,
    TargetArtifact, TargetAstBuilder, TargetAstPackage, TargetCallableSignature, TargetDialect,
    TargetExprId, TargetExpressionNode, TargetFile, TargetFileId, TargetFileItemNode,
    TargetResolver, TargetStatementNode, TargetStmtId, TargetTypeMarker, TargetTypeRef,
    TypedAstDialect, UnresolvedPackage, verify_target_ast,
};

macro_rules! linker_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            pub const fn index(self) -> usize {
                self.0 as usize
            }

            #[allow(dead_code)]
            fn from_index(index: usize) -> Self {
                Self(u32::try_from(index).expect("target linker arena exceeds u32"))
            }
        }
    };
}

linker_id!(TargetLocalId);
linker_id!(TargetPackageId);
linker_id!(TargetTestId);
linker_id!(ResolvedImportId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GeneratedSymbolId {
    Type(GeneratedTypeId),
    Callable(GeneratedCallableId),
    InterfaceMethod(GeneratedInterfaceMethodId),
    Value(GeneratedValueId),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TargetSymbolRef<D: LinkerDialect> {
    KnownType(D::KnownType),
    KnownCallable(D::KnownCallable),
    RuntimeCallable(D::RuntimeCallable),
    KnownField(D::KnownField),
    KnownConstructor(D::KnownConstructor),
    KnownMethod(D::KnownMethod),
    Generated(GeneratedSymbolId),
    RuntimeHelper(D::HelperId),
    TypeParameter(crate::TargetTypeParameterId),
    Local(TargetLocalId),
    Package(TargetPackageId),
    File(TargetFileId),
    Test(TargetTestId),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolOrigin<D: LinkerDialect> {
    Primitive,
    LanguagePrelude(D::PreludeSymbol),
    StandardLibrary(D::StandardLibrary),
    ExternalPackage(D::ExternalPackage),
    Generated(GeneratedSymbolId),
    Runtime(D::HelperId),
    TypeParameter(crate::TargetTypeParameterId),
    Local(TargetLocalId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FailureBehavior {
    Infallible,
    ReturnsSentinel,
    ReturnsResult,
    ThrowsChecked,
    ThrowsUnchecked,
    Aborts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TargetEffect {
    Allocation,
    Mutation,
    InputOutput,
    Nondeterminism,
    MayBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeParameterVariance {
    Invariant,
    Covariant,
    Contravariant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeParameterSpec<D: LinkerDialect> {
    pub name: D::Identifier,
    pub variance: TypeParameterVariance,
    pub upper_bound: Option<TargetTypeRef<D>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypePattern<D: LinkerDialect> {
    Exact(TargetTypeRef<D>),
    Parameter(u16),
    Constructed {
        constructor: D::ConstructedType,
        arguments: Vec<TypePattern<D>>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallablePattern<D: LinkerDialect> {
    pub invocation: D::InvocationKind,
    pub type_parameters: Vec<TypeParameterSpec<D>>,
    pub receiver: Option<TypePattern<D>>,
    pub parameters: Vec<TypePattern<D>>,
    pub result: TypePattern<D>,
    pub failure: FailureBehavior,
    pub effects: BTreeSet<TargetEffect>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageRequirement<D: LinkerDialect> {
    pub package: D::ExternalPackage,
    pub version_requirement: String,
    pub features: BTreeSet<D::PackageFeature>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DependencyPolicy<D: LinkerDialect> {
    Implicit,
    Import(D::ImportKind),
    Qualified,
    Member {
        owner: D::QualifiedName,
        member: D::MemberName,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownTypeSpec<D: LinkerDialect> {
    pub symbol: D::KnownType,
    pub name: D::Identifier,
    pub alias_stem: String,
    pub qualified_name: Option<D::QualifiedName>,
    pub origin: SymbolOrigin<D>,
    pub arity: u16,
    pub policy: DependencyPolicy<D>,
    pub dependency: Option<PackageRequirement<D>>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownCallableSpec<D: LinkerDialect> {
    pub symbol: D::KnownCallable,
    pub owner: Option<D::KnownType>,
    pub name: D::Identifier,
    pub alias_stem: String,
    pub qualified_name: Option<D::QualifiedName>,
    pub origin: SymbolOrigin<D>,
    pub signature: CallablePattern<D>,
    pub visibility: D::Visibility,
    pub policy: DependencyPolicy<D>,
    pub dependency: Option<PackageRequirement<D>>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeCallableSpec<D: LinkerDialect> {
    pub symbol: D::RuntimeCallable,
    pub name: D::Identifier,
    pub alias_stem: String,
    pub qualified_name: Option<D::QualifiedName>,
    pub origin: SymbolOrigin<D>,
    pub signature: CallablePattern<D>,
    pub policy: DependencyPolicy<D>,
    pub dependency: Option<PackageRequirement<D>>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownFieldSpec<D: LinkerDialect> {
    pub symbol: D::KnownField,
    pub owner: D::KnownType,
    pub name: D::Identifier,
    pub origin: SymbolOrigin<D>,
    pub ty: TypePattern<D>,
    pub policy: DependencyPolicy<D>,
    pub dependency: Option<PackageRequirement<D>>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownConstructorSpec<D: LinkerDialect> {
    pub symbol: D::KnownConstructor,
    pub owner: D::KnownType,
    pub name: D::Identifier,
    pub alias_stem: String,
    pub qualified_name: Option<D::QualifiedName>,
    pub origin: SymbolOrigin<D>,
    pub signature: CallablePattern<D>,
    pub visibility: D::Visibility,
    pub policy: DependencyPolicy<D>,
    pub dependency: Option<PackageRequirement<D>>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownMethodSpec<D: LinkerDialect> {
    pub symbol: D::KnownMethod,
    pub owner: D::KnownType,
    pub name: D::Identifier,
    pub origin: SymbolOrigin<D>,
    pub signature: CallablePattern<D>,
    pub visibility: D::Visibility,
    pub policy: DependencyPolicy<D>,
    pub dependency: Option<PackageRequirement<D>>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeHelperSpec<D: LinkerDialect> {
    pub id: D::HelperId,
    pub capability: D::HelperCapability,
    pub order: u32,
    pub name: D::Identifier,
    pub alias_stem: String,
    pub namespace: D::Namespace,
    pub items: Vec<D::FileItem>,
    pub placement: D::FilePlacement,
    pub visibility: D::Visibility,
    pub source: SourceRef,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SymbolCatalogue<D: LinkerDialect> {
    pub types: Vec<KnownTypeSpec<D>>,
    pub callables: Vec<KnownCallableSpec<D>>,
    pub runtime_callables: Vec<RuntimeCallableSpec<D>>,
    pub fields: Vec<KnownFieldSpec<D>>,
    pub constructors: Vec<KnownConstructorSpec<D>>,
    pub methods: Vec<KnownMethodSpec<D>>,
    pub helpers: Vec<RuntimeHelperSpec<D>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileItemRoots<D: LinkerDialect> {
    pub declarations: Vec<GeneratedSymbolId>,
    pub expressions: Vec<TargetExprId>,
    pub statements: Vec<TargetStmtId>,
    pub symbols: Vec<TargetSymbolRef<D>>,
}

impl<D: LinkerDialect> Default for FileItemRoots<D> {
    fn default() -> Self {
        Self {
            declarations: vec![],
            expressions: vec![],
            statements: vec![],
            symbols: vec![],
        }
    }
}

pub trait LinkerDialect:
    TypedAstDialect + TargetDialect<Resolved = LinkedTargetPackage<Self>> + Clone + Ord
{
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

impl<D: LinkerDialect> SymbolCatalogue<D> {
    pub fn verify(&self, dialect: &D) -> Result<(), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        check_unique(
            &mut diagnostics,
            self.types.iter().map(|spec| (&spec.symbol, &spec.source)),
            "known type",
        );
        check_unique(
            &mut diagnostics,
            self.callables
                .iter()
                .map(|spec| (&spec.symbol, &spec.source)),
            "known callable",
        );
        check_unique(
            &mut diagnostics,
            self.runtime_callables
                .iter()
                .map(|spec| (&spec.symbol, &spec.source)),
            "runtime callable",
        );
        check_unique(
            &mut diagnostics,
            self.fields.iter().map(|spec| (&spec.symbol, &spec.source)),
            "known field",
        );
        check_unique(
            &mut diagnostics,
            self.constructors
                .iter()
                .map(|spec| (&spec.symbol, &spec.source)),
            "known constructor",
        );
        check_unique(
            &mut diagnostics,
            self.methods.iter().map(|spec| (&spec.symbol, &spec.source)),
            "known method",
        );
        check_unique(
            &mut diagnostics,
            self.helpers.iter().map(|spec| (&spec.id, &spec.source)),
            "runtime helper",
        );
        check_unique(
            &mut diagnostics,
            self.helpers.iter().map(|spec| (&spec.order, &spec.source)),
            "runtime helper order",
        );
        for helper in &self.helpers {
            if dialect.is_public(&helper.visibility) {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidStructure,
                    "runtime helpers cannot be exposed as public declarations",
                    helper.source.clone(),
                ));
            }
            if helper.items.is_empty() {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidStructure,
                    "runtime helper must expand to structural target AST items",
                    helper.source.clone(),
                ));
            }
        }

        for spec in &self.callables {
            validate_callable_pattern(&mut diagnostics, &spec.signature, &spec.source);
            if let Some(concrete) = concrete_signature(&spec.signature) {
                let authoritative = dialect.known_callable_signature(&spec.symbol);
                if concrete != authoritative {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        "known-callable metadata disagrees with its typed signature",
                        spec.source.clone(),
                    ));
                }
            }
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                spec.qualified_name.as_ref(),
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        for spec in &self.runtime_callables {
            validate_callable_pattern(&mut diagnostics, &spec.signature, &spec.source);
            if let Some(concrete) = concrete_signature(&spec.signature) {
                let authoritative = dialect.runtime_callable_signature(&spec.symbol);
                if concrete != authoritative {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        "runtime-callable metadata disagrees with its typed signature",
                        spec.source.clone(),
                    ));
                }
            }
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                spec.qualified_name.as_ref(),
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        for spec in &self.constructors {
            validate_callable_pattern(&mut diagnostics, &spec.signature, &spec.source);
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                spec.qualified_name.as_ref(),
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        for spec in &self.methods {
            validate_callable_pattern(&mut diagnostics, &spec.signature, &spec.source);
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                None,
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        for spec in &self.types {
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                spec.qualified_name.as_ref(),
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        for spec in &self.fields {
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                None,
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        sort_diagnostics(&mut diagnostics);
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }

    fn helper(&self, id: &D::HelperId) -> Option<&RuntimeHelperSpec<D>> {
        self.helpers.iter().find(|spec| &spec.id == id)
    }
}

pub trait KnownNullaryCall<D: LinkerDialect> {
    type Result: TargetTypeMarker<D>;
    fn callable() -> D::KnownCallable;
}

pub trait KnownUnaryCall<D: LinkerDialect> {
    type Argument: TargetTypeMarker<D>;
    type Result: TargetTypeMarker<D>;
    fn callable() -> D::KnownCallable;
}

pub trait KnownBinaryCall<D: LinkerDialect> {
    type Left: TargetTypeMarker<D>;
    type Right: TargetTypeMarker<D>;
    type Result: TargetTypeMarker<D>;
    fn callable() -> D::KnownCallable;
}

pub trait KnownNullaryConstructor<D: LinkerDialect> {
    type Result: TargetTypeMarker<D>;
    fn constructor() -> D::KnownConstructor;
}

pub trait KnownInstanceUnaryCall<D: LinkerDialect> {
    type Receiver: TargetTypeMarker<D>;
    type Argument: TargetTypeMarker<D>;
    type Result: TargetTypeMarker<D>;
    fn method() -> D::KnownMethod;
}

pub fn known_nullary_call<D, C>(
    builder: &mut TargetAstBuilder<D>,
    source: SourceRef,
) -> Expr<D, C::Result>
where
    D: LinkerDialect,
    C: KnownNullaryCall<D>,
{
    known_call(builder, C::callable(), vec![], source)
}

/// Known unary calls enforce their declared argument marker at compile time.
///
/// ```compile_fail
/// use portable_codegen::{
///     Expr, KnownUnaryCall, LinkerDialect, TargetAstBuilder, TargetTypeMarker,
///     known_unary_call,
/// };
/// use portable_diagnostics::SourceRef;
///
/// fn wrong_argument<D, C, Wrong>(
///     builder: &mut TargetAstBuilder<D>,
///     value: Expr<D, Wrong>,
/// ) where
///     D: LinkerDialect,
///     C: KnownUnaryCall<D>,
///     Wrong: TargetTypeMarker<D>,
/// {
///     let _ = known_unary_call::<D, C>(builder, value, SourceRef::logical(["test"]));
/// }
/// ```
pub fn known_unary_call<D, C>(
    builder: &mut TargetAstBuilder<D>,
    argument: Expr<D, C::Argument>,
    source: SourceRef,
) -> Expr<D, C::Result>
where
    D: LinkerDialect,
    C: KnownUnaryCall<D>,
{
    known_call(builder, C::callable(), vec![argument.id()], source)
}

pub fn known_binary_call<D, C>(
    builder: &mut TargetAstBuilder<D>,
    left: Expr<D, C::Left>,
    right: Expr<D, C::Right>,
    source: SourceRef,
) -> Expr<D, C::Result>
where
    D: LinkerDialect,
    C: KnownBinaryCall<D>,
{
    known_call(builder, C::callable(), vec![left.id(), right.id()], source)
}

pub fn known_nullary_constructor<D, C>(
    builder: &mut TargetAstBuilder<D>,
    source: SourceRef,
) -> Expr<D, C::Result>
where
    D: LinkerDialect,
    C: KnownNullaryConstructor<D>,
{
    let node = builder
        .dialect()
        .known_constructor_expression(C::constructor(), vec![]);
    builder.expression::<C::Result>(node, source)
}

/// Instance-call receivers and arguments are separate phantom-typed inputs.
///
/// ```compile_fail
/// use portable_codegen::{
///     Expr, KnownInstanceUnaryCall, LinkerDialect, TargetAstBuilder,
///     TargetTypeMarker, known_instance_unary_call,
/// };
/// use portable_diagnostics::SourceRef;
///
/// fn wrong_receiver<D, C, Wrong>(
///     builder: &mut TargetAstBuilder<D>,
///     receiver: Expr<D, Wrong>,
///     argument: Expr<D, C::Argument>,
/// ) where
///     D: LinkerDialect,
///     C: KnownInstanceUnaryCall<D>,
///     Wrong: TargetTypeMarker<D>,
/// {
///     let _ = known_instance_unary_call::<D, C>(
///         builder,
///         receiver,
///         argument,
///         SourceRef::logical(["test"]),
///     );
/// }
/// ```
pub fn known_instance_unary_call<D, C>(
    builder: &mut TargetAstBuilder<D>,
    receiver: Expr<D, C::Receiver>,
    argument: Expr<D, C::Argument>,
    source: SourceRef,
) -> Expr<D, C::Result>
where
    D: LinkerDialect,
    C: KnownInstanceUnaryCall<D>,
{
    let node =
        builder
            .dialect()
            .known_method_expression(C::method(), receiver.id(), vec![argument.id()]);
    builder.expression::<C::Result>(node, source)
}

fn known_call<D, T>(
    builder: &mut TargetAstBuilder<D>,
    callable: D::KnownCallable,
    arguments: Vec<TargetExprId>,
    source: SourceRef,
) -> Expr<D, T>
where
    D: LinkerDialect,
    T: TargetTypeMarker<D>,
{
    let node = {
        let dialect = builder.dialect();
        let signature = dialect.known_callable_signature(&callable);
        dialect.known_call_expression(callable, signature.invocation, arguments)
    };
    builder.expression::<T>(node, source)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BindableSymbolId<D: LinkerDialect> {
    Generated(GeneratedSymbolId),
    Helper(D::HelperId),
    Local(TargetLocalId),
    Package(TargetPackageId),
    File(TargetFileId),
    Test(TargetTestId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BindingScope {
    Package,
    Type(GeneratedTypeId),
    Callable(GeneratedCallableId),
    File(TargetFileId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedBinding<D: LinkerDialect> {
    symbol: BindableSymbolId<D>,
    identifier: D::Identifier,
    namespace: D::Namespace,
    scope: BindingScope,
    public: bool,
    source: SourceRef,
}

impl<D: LinkerDialect> ResolvedBinding<D> {
    #[cfg(test)]
    fn symbol(&self) -> &BindableSymbolId<D> {
        &self.symbol
    }

    pub fn identifier(&self) -> &D::Identifier {
        &self.identifier
    }

    pub fn namespace(&self) -> &D::Namespace {
        &self.namespace
    }

    pub const fn scope(&self) -> BindingScope {
        self.scope
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedReference<D: LinkerDialect> {
    Local(D::Identifier),
    Imported {
        binding: D::Identifier,
        import: ResolvedImportId,
    },
    Qualified(D::QualifiedName),
    Member {
        owner: D::QualifiedName,
        member: D::MemberName,
    },
}

/// Read-only symbol spelling table available to a dialect's resolver.
///
/// Only the shared linker constructs this map. Renderers never receive it or
/// any unresolved target symbol reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedReferenceMap<D: LinkerDialect> {
    references: BTreeMap<TargetSymbolRef<D>, ResolvedReference<D>>,
}

impl<D: LinkerDialect> ResolvedReferenceMap<D> {
    pub fn get(&self, symbol: &TargetSymbolRef<D>) -> Option<&ResolvedReference<D>> {
        self.references.get(symbol)
    }

    fn from_linked(
        references: &[LinkedReference<D>],
        source: &SourceRef,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Self {
        let mut resolved = BTreeMap::new();
        for reference in references {
            if let Some(previous) =
                resolved.insert(reference.symbol.clone(), reference.resolved.clone())
                && previous != reference.resolved
            {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InterfaceNonconformance,
                    "one typed symbol resolved to inconsistent spellings in one file",
                    source.clone(),
                ));
            }
        }
        Self {
            references: resolved,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedImport<D: LinkerDialect> {
    id: ResolvedImportId,
    symbols: BTreeSet<TargetSymbolRef<D>>,
    original_binding: D::Identifier,
    binding: D::Identifier,
    kind: D::ImportKind,
    origin: SymbolOrigin<D>,
}

impl<D: LinkerDialect> ResolvedImport<D> {
    pub const fn id(&self) -> ResolvedImportId {
        self.id
    }

    #[cfg(test)]
    fn symbols(&self) -> &BTreeSet<TargetSymbolRef<D>> {
        &self.symbols
    }

    pub fn binding(&self) -> &D::Identifier {
        &self.binding
    }

    pub fn original_binding(&self) -> &D::Identifier {
        &self.original_binding
    }

    pub fn kind(&self) -> &D::ImportKind {
        &self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedPackageDependency<D: LinkerDialect> {
    requirement: PackageRequirement<D>,
}

impl<D: LinkerDialect> ResolvedPackageDependency<D> {
    pub fn requirement(&self) -> &PackageRequirement<D> {
        &self.requirement
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LinkedReference<D: LinkerDialect> {
    source: SourceRef,
    symbol: TargetSymbolRef<D>,
    resolved: ResolvedReference<D>,
}

impl<D: LinkerDialect> LinkedReference<D> {
    #[cfg(test)]
    fn resolved(&self) -> &ResolvedReference<D> {
        &self.resolved
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedFile<D: LinkerDialect> {
    file: TargetFileId,
    path: crate::RelativeOutputPath,
    role: SourceRole,
    module: D::ResolvedModule,
    placement: D::FilePlacement,
    template: D::TemplateId,
    items: Vec<D::ResolvedFileItem>,
    source: SourceRef,
    dependencies: Vec<TargetFileId>,
    references: Vec<LinkedReference<D>>,
    imports: Vec<ResolvedImport<D>>,
    forward_declarations: Vec<GeneratedSymbolId>,
    helpers: Vec<D::HelperId>,
}

impl<D: LinkerDialect> LinkedFile<D> {
    pub const fn file(&self) -> TargetFileId {
        self.file
    }

    pub fn path(&self) -> &crate::RelativeOutputPath {
        &self.path
    }

    pub const fn role(&self) -> SourceRole {
        self.role
    }

    pub fn module(&self) -> &D::ResolvedModule {
        &self.module
    }

    pub fn placement(&self) -> &D::FilePlacement {
        &self.placement
    }

    pub fn template(&self) -> &D::TemplateId {
        &self.template
    }

    pub fn items(&self) -> &[D::ResolvedFileItem] {
        &self.items
    }

    pub fn source(&self) -> &SourceRef {
        &self.source
    }

    #[cfg(test)]
    fn references(&self) -> &[LinkedReference<D>] {
        &self.references
    }

    pub fn dependencies(&self) -> &[TargetFileId] {
        &self.dependencies
    }

    pub fn imports(&self) -> &[ResolvedImport<D>] {
        &self.imports
    }

    #[cfg(test)]
    fn forward_declarations(&self) -> &[GeneratedSymbolId] {
        &self.forward_declarations
    }

    #[cfg(test)]
    fn helpers(&self) -> &[D::HelperId] {
        &self.helpers
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LinkedRuntimeHelper<D: LinkerDialect> {
    id: D::HelperId,
    capability: D::HelperCapability,
    order: u32,
    file: TargetFileId,
    items: Vec<D::FileItem>,
    source: SourceRef,
}

impl<D: LinkerDialect> LinkedRuntimeHelper<D> {
    #[cfg(test)]
    pub fn id(&self) -> &D::HelperId {
        &self.id
    }

    #[cfg(test)]
    pub const fn file(&self) -> TargetFileId {
        self.file
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedTargetPackage<D: LinkerDialect> {
    dialect: D,
    unresolved: TargetAstPackage<D>,
    bindings: Vec<ResolvedBinding<D>>,
    files: Vec<LinkedFile<D>>,
    dependencies: Vec<ResolvedPackageDependency<D>>,
    helpers: Vec<LinkedRuntimeHelper<D>>,
    catalogue: SymbolCatalogue<D>,
}

/// A resolved package does not expose its unresolved AST.
///
/// ```compile_fail
/// use portable_codegen::{LinkedTargetPackage, LinkerDialect};
///
/// fn cannot_recover_unresolved<D: LinkerDialect>(
///     package: &LinkedTargetPackage<D>,
/// ) {
///     let _ = package.unresolved();
/// }
/// ```
impl<D: LinkerDialect> LinkedTargetPackage<D> {
    #[cfg(test)]
    fn bindings(&self) -> &[ResolvedBinding<D>] {
        &self.bindings
    }

    pub fn files(&self) -> &[LinkedFile<D>] {
        &self.files
    }

    pub fn dependencies(&self) -> &[ResolvedPackageDependency<D>] {
        &self.dependencies
    }

    pub(crate) fn artifacts(&self) -> impl ExactSizeIterator<Item = &TargetArtifact> {
        self.unresolved.artifacts()
    }

    pub(crate) fn manifest_dependencies(&self) -> Vec<DeclaredDependency> {
        let mut dependencies = self
            .dependencies
            .iter()
            .map(|dependency| {
                let requirement = dependency.requirement();
                let mut features = requirement
                    .features
                    .iter()
                    .map(|feature| self.dialect.package_feature_name(feature).to_owned())
                    .collect::<Vec<_>>();
                features.sort();
                DeclaredDependency {
                    ecosystem: self
                        .dialect
                        .package_ecosystem(&requirement.package)
                        .as_str()
                        .to_owned(),
                    name: self.dialect.package_name(&requirement.package).to_owned(),
                    requirement: requirement.version_requirement.clone(),
                    features,
                }
            })
            .collect::<Vec<_>>();
        dependencies.sort();
        dependencies
    }

    pub(crate) fn manifest_helpers(&self) -> Vec<InjectedHelper> {
        let mut helpers = self
            .helpers
            .iter()
            .filter_map(|helper| {
                self.files
                    .iter()
                    .find(|file| file.file == helper.file)
                    .map(|file| InjectedHelper {
                        id: self.dialect.helper_name(&helper.id).to_owned(),
                        capability: self
                            .dialect
                            .helper_capability_name(&helper.capability)
                            .to_owned(),
                        files: vec![file.path().as_str().to_owned()],
                    })
            })
            .collect::<Vec<_>>();
        helpers.sort();
        helpers
    }

    #[cfg(test)]
    fn helpers(&self) -> &[LinkedRuntimeHelper<D>] {
        &self.helpers
    }

    #[cfg(test)]
    fn canonical_dump(&self) -> String {
        format!("{self:#?}")
    }

    #[cfg(test)]
    fn files_mut(&mut self) -> &mut [LinkedFile<D>] {
        &mut self.files
    }

    #[cfg(test)]
    fn dependencies_mut(&mut self) -> &mut Vec<ResolvedPackageDependency<D>> {
        &mut self.dependencies
    }

    #[cfg(test)]
    fn helpers_mut(&mut self) -> &mut Vec<LinkedRuntimeHelper<D>> {
        &mut self.helpers
    }
}

#[derive(Clone, Debug)]
pub struct TargetLinker<D: LinkerDialect> {
    dialect: D,
}

impl<D: LinkerDialect> TargetLinker<D> {
    pub const fn new(dialect: D) -> Self {
        Self { dialect }
    }

    pub fn link_ast(
        &self,
        unresolved: &TargetAstPackage<D>,
    ) -> Result<LinkedTargetPackage<D>, Vec<Diagnostic>> {
        verify_target_ast(unresolved)?;
        let catalogue = self.dialect.symbol_catalogue();
        catalogue.verify(&self.dialect)?;

        let mut diagnostics = Vec::new();
        let mut raw_files = collect_references(&self.dialect, unresolved, &mut diagnostics);
        let selected_helpers = expand_file_helpers(
            &self.dialect,
            unresolved,
            &catalogue,
            &mut raw_files,
            &mut diagnostics,
        );
        derive_and_validate_file_graph(&self.dialect, unresolved, &mut raw_files, &mut diagnostics);
        let selected_helper_ids = selected_helpers
            .iter()
            .map(|helper| helper.id.clone())
            .collect::<BTreeSet<_>>();
        let bindings = allocate_bindings(
            &self.dialect,
            unresolved,
            &catalogue,
            &selected_helper_ids,
            &mut diagnostics,
        );
        let binding_lookup = bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| (binding.symbol.clone(), index))
            .collect::<BTreeMap<_, _>>();
        let mut requirements = BTreeMap::new();
        let mut next_import = 0usize;
        let mut files = Vec::new();

        for raw_file in raw_files {
            let mut imports = Vec::new();
            let mut import_lookup = BTreeMap::new();
            let mut physical_import_lookup = BTreeMap::new();
            let mut references = Vec::new();
            let mut occupied = occupied_names(&self.dialect, &bindings, raw_file.file);
            for located in raw_file.references {
                let resolved = resolve_reference(
                    &self.dialect,
                    &catalogue,
                    &bindings,
                    &binding_lookup,
                    &mut imports,
                    &mut import_lookup,
                    &mut physical_import_lookup,
                    &mut occupied,
                    &mut requirements,
                    &mut next_import,
                    &located,
                    &mut diagnostics,
                );
                if let Some(resolved) = resolved {
                    references.push(LinkedReference {
                        source: located.source,
                        symbol: located.symbol,
                        resolved,
                    });
                }
            }
            let unresolved_refs = references
                .iter()
                .map(|reference| reference.symbol.clone())
                .collect::<Vec<_>>();
            let forward_declarations = self
                .dialect
                .forward_declarations(raw_file.file, &unresolved_refs);
            let source_file = unresolved
                .file(raw_file.file)
                .expect("raw linker file was collected from this package");
            let reference_map = ResolvedReferenceMap::from_linked(
                &references,
                source_file.source(),
                &mut diagnostics,
            );
            let module = match self.dialect.resolve_module(source_file.module()) {
                Ok(module) => module,
                Err(violation) => {
                    diagnostics.push(Diagnostic::error(
                        violation.code,
                        violation.message,
                        source_file.source().clone(),
                    ));
                    continue;
                }
            };
            let mut items = Vec::new();
            let unresolved_items = source_file
                .items()
                .iter()
                .map(|item| (item, source_file.source()))
                .chain(
                    selected_helpers
                        .iter()
                        .filter(|helper| helper.file == raw_file.file)
                        .flat_map(|helper| {
                            helper.items.iter().map(move |item| (item, &helper.source))
                        }),
                );
            for (item, source) in unresolved_items {
                match self
                    .dialect
                    .resolve_file_item(unresolved, item, &reference_map)
                {
                    Ok(item) => {
                        diagnostics.extend(
                            self.dialect
                                .verify_resolved_file_item(&item)
                                .into_iter()
                                .map(|violation| {
                                    Diagnostic::error(
                                        violation.code,
                                        violation.message,
                                        source.clone(),
                                    )
                                }),
                        );
                        items.push(item);
                    }
                    Err(violation) => diagnostics.push(Diagnostic::error(
                        violation.code,
                        violation.message,
                        source.clone(),
                    )),
                }
            }
            files.push(LinkedFile {
                file: raw_file.file,
                path: source_file.path().clone(),
                role: source_file.role(),
                module,
                placement: source_file.placement().clone(),
                template: source_file.template().clone(),
                items,
                source: source_file.source().clone(),
                dependencies: raw_file.dependencies,
                references,
                imports,
                forward_declarations,
                helpers: raw_file.helpers,
            });
        }

        sort_diagnostics(&mut diagnostics);
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        let linked = LinkedTargetPackage {
            dialect: self.dialect.clone(),
            unresolved: unresolved.clone(),
            bindings,
            files,
            dependencies: requirements
                .into_values()
                .map(|requirement| ResolvedPackageDependency { requirement })
                .collect(),
            helpers: selected_helpers,
            catalogue,
        };
        verify_linked_package(&linked)?;
        Ok(linked)
    }
}

impl<D: LinkerDialect> TargetResolver<D> for TargetLinker<D> {
    fn resolve_target(
        &self,
        package: &UnresolvedPackage<D>,
    ) -> Result<D::Resolved, Vec<Diagnostic>> {
        self.link_ast(package.ast())
    }
}

#[derive(Clone)]
struct LocatedSymbol<D: LinkerDialect> {
    symbol: TargetSymbolRef<D>,
    source: SourceRef,
}

struct RawFile<D: LinkerDialect> {
    file: TargetFileId,
    references: Vec<LocatedSymbol<D>>,
    helpers: Vec<D::HelperId>,
    dependencies: Vec<TargetFileId>,
}

fn collect_references<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<RawFile<D>> {
    let mut files = Vec::new();
    for (file_index, file) in package.files().enumerate() {
        let file_id = TargetFileId::from_index(file_index);
        let references =
            collect_item_references(dialect, package, file.items(), file.source(), diagnostics);
        files.push(RawFile {
            file: file_id,
            references,
            helpers: vec![],
            dependencies: vec![],
        });
    }
    files
}

fn collect_item_references<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    items: &[D::FileItem],
    item_source: &SourceRef,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<LocatedSymbol<D>> {
    let mut references = Vec::new();
    let mut visited_expressions = BTreeSet::new();
    let mut visited_statements = BTreeSet::new();
    for item in items {
        let roots = dialect.file_item_roots(item);
        references.extend(roots.symbols.into_iter().map(|symbol| LocatedSymbol {
            symbol,
            source: item_source.clone(),
        }));
        for statement in roots.statements {
            if !visited_statements.insert(statement) {
                continue;
            }
            let Some((node, source)) = package.statement(statement) else {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnresolvedReference,
                    "linker file item refers to a missing statement",
                    item_source.clone(),
                ));
                continue;
            };
            references.extend(
                dialect
                    .statement_references(node)
                    .into_iter()
                    .map(|symbol| LocatedSymbol {
                        symbol,
                        source: source.clone(),
                    }),
            );
            for expression in node.child_expressions() {
                collect_expression(
                    dialect,
                    package,
                    expression,
                    &mut visited_expressions,
                    &mut references,
                    diagnostics,
                );
            }
        }
        for expression in roots.expressions {
            collect_expression(
                dialect,
                package,
                expression,
                &mut visited_expressions,
                &mut references,
                diagnostics,
            );
        }
    }
    references
}

fn collect_expression<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    expression: TargetExprId,
    visited: &mut BTreeSet<TargetExprId>,
    references: &mut Vec<LocatedSymbol<D>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !visited.insert(expression) {
        return;
    }
    let Some((_, node, source)) = package.expression(expression) else {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnresolvedReference,
            "linker traversal reached a missing expression",
            SourceRef::logical(["target-linker", "expression"]),
        ));
        return;
    };
    for child in node.child_expressions() {
        collect_expression(dialect, package, child, visited, references, diagnostics);
    }
    references.extend(
        dialect
            .expression_references(node)
            .into_iter()
            .map(|symbol| LocatedSymbol {
                symbol,
                source: source.clone(),
            }),
    );
}

fn expand_file_helpers<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    catalogue: &SymbolCatalogue<D>,
    files: &mut [RawFile<D>],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<LinkedRuntimeHelper<D>> {
    let mut selected = BTreeSet::new();
    let roots = files
        .iter()
        .flat_map(|file| {
            file.references
                .iter()
                .filter_map(|reference| match &reference.symbol {
                    TargetSymbolRef::RuntimeHelper(helper) => {
                        Some((helper.clone(), reference.source.clone()))
                    }
                    _ => None,
                })
        })
        .collect::<Vec<_>>();
    let mut states = BTreeMap::new();
    let mut helper_references = BTreeMap::new();
    for (helper, source) in roots {
        expand_helper(
            dialect,
            package,
            catalogue,
            &helper,
            &source,
            &mut states,
            &mut selected,
            &mut helper_references,
            diagnostics,
        );
    }

    let mut selected_specs = selected
        .iter()
        .filter_map(|id| catalogue.helper(id))
        .collect::<Vec<_>>();
    selected_specs.sort_by_key(|spec| (spec.order, spec.id.clone()));
    let mut linked = Vec::new();
    for spec in selected_specs {
        let destinations = package
            .files()
            .enumerate()
            .filter(|(_, file)| {
                file.role() == SourceRole::Runtime && file.placement() == &spec.placement
            })
            .map(|(index, _)| TargetFileId::from_index(index))
            .collect::<Vec<_>>();
        if destinations.len() != 1 {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                "runtime helper placement must select exactly one typed runtime file",
                spec.source.clone(),
            ));
            continue;
        }
        let destination = destinations[0];
        if let Some(raw_file) = files.iter_mut().find(|file| file.file == destination) {
            raw_file.helpers.push(spec.id.clone());
            if let Some(references) = helper_references.remove(&spec.id) {
                raw_file.references.extend(references);
            }
        }
        linked.push(LinkedRuntimeHelper {
            id: spec.id.clone(),
            capability: spec.capability.clone(),
            order: spec.order,
            file: destination,
            items: spec.items.clone(),
            source: spec.source.clone(),
        });
    }
    linked
}

#[allow(clippy::too_many_arguments)]
fn expand_helper<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    catalogue: &SymbolCatalogue<D>,
    helper: &D::HelperId,
    requested_at: &SourceRef,
    states: &mut BTreeMap<D::HelperId, u8>,
    selected: &mut BTreeSet<D::HelperId>,
    helper_references: &mut BTreeMap<D::HelperId, Vec<LocatedSymbol<D>>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match states.get(helper) {
        Some(2) => return,
        Some(1) => {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::AliasCycle,
                "runtime-helper dependency cycle",
                requested_at.clone(),
            ));
            return;
        }
        _ => {}
    }
    let Some(spec) = catalogue.helper(helper) else {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnresolvedReference,
            "runtime-helper reference has no catalogue entry",
            requested_at.clone(),
        ));
        return;
    };
    let context = package.context();
    for item in &spec.items {
        diagnostics.extend(item.verify(&context).into_iter().map(|violation| {
            Diagnostic::error(violation.code, violation.message, spec.source.clone())
        }));
    }
    let references =
        collect_item_references(dialect, package, &spec.items, &spec.source, diagnostics);
    states.insert(helper.clone(), 1);
    for reference in &references {
        if let TargetSymbolRef::RuntimeHelper(child) = &reference.symbol {
            expand_helper(
                dialect,
                package,
                catalogue,
                child,
                &spec.source,
                states,
                selected,
                helper_references,
                diagnostics,
            );
        }
    }
    states.insert(helper.clone(), 2);
    selected.insert(helper.clone());
    helper_references.insert(helper.clone(), references);
}

fn derive_and_validate_file_graph<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    files: &mut [RawFile<D>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut declarations = BTreeMap::new();
    for (file_index, file) in package.files().enumerate() {
        let file_id = TargetFileId::from_index(file_index);
        for declaration in file
            .items()
            .iter()
            .flat_map(|item| dialect.file_item_roots(item).declarations)
        {
            if generated_symbol_source(package, declaration).is_none() {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnresolvedReference,
                    "file item declares a missing generated symbol",
                    file.source().clone(),
                ));
            } else if declarations.insert(declaration, file_id).is_some() {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::DuplicateDeclaration,
                    "generated symbol is declared in more than one source file",
                    file.source().clone(),
                ));
            }
        }
    }
    for symbol in (0..package.generated_types().len())
        .map(|index| GeneratedSymbolId::Type(GeneratedTypeId::from_index(index)))
        .chain(
            (0..package.callables().len())
                .map(|index| GeneratedSymbolId::Callable(GeneratedCallableId::from_index(index))),
        )
    {
        if !declarations.contains_key(&symbol) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                "generated top-level symbol is not placed in a source file",
                generated_symbol_source(package, symbol)
                    .cloned()
                    .unwrap_or_else(|| SourceRef::logical(["target-linker", "file-graph"])),
            ));
        }
    }

    let mut graph = BTreeMap::new();
    for raw_file in files.iter_mut() {
        let from_role = package.file(raw_file.file).map(TargetFile::role);
        let mut edges = BTreeSet::new();
        for reference in &raw_file.references {
            let TargetSymbolRef::Generated(symbol) = reference.symbol else {
                continue;
            };
            let Some(destination) = declarations.get(&symbol).copied() else {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnresolvedReference,
                    "generated symbol reference has no structural source-file declaration",
                    reference.source.clone(),
                ));
                continue;
            };
            if destination == raw_file.file {
                continue;
            }
            edges.insert(destination);
            let to_role = package.file(destination).map(TargetFile::role);
            if violates_file_visibility(dialect, package, from_role, to_role, symbol) {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidStructure,
                    "cross-file reference violates source-role or public-API visibility",
                    reference.source.clone(),
                ));
            }
        }
        raw_file.dependencies = edges.iter().copied().collect();
        graph.insert(raw_file.file, edges);
    }

    if let Some(cycle) = find_file_cycle(&graph)
        && !dialect.permits_file_cycle(&cycle)
    {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            "target source-file dependency graph contains a forbidden cycle",
            SourceRef::logical(["target-linker", "file-cycle"]),
        ));
    }
}

fn generated_symbol_source<D: LinkerDialect>(
    package: &TargetAstPackage<D>,
    symbol: GeneratedSymbolId,
) -> Option<&SourceRef> {
    match symbol {
        GeneratedSymbolId::Type(id) => package.generated_type(id).map(|value| &value.source),
        GeneratedSymbolId::Callable(id) => package.callable(id).map(|value| &value.source),
        GeneratedSymbolId::InterfaceMethod(id) => {
            package.interface_method(id).map(|value| &value.source)
        }
        GeneratedSymbolId::Value(id) => package.value(id).map(|value| &value.source),
    }
}

fn generated_symbol_is_public<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    symbol: GeneratedSymbolId,
) -> bool {
    match symbol {
        GeneratedSymbolId::Type(id) => package
            .generated_type(id)
            .is_some_and(|value| dialect.is_public(&value.visibility)),
        GeneratedSymbolId::Callable(id) => package
            .callable(id)
            .is_some_and(|value| dialect.is_public(&value.visibility)),
        GeneratedSymbolId::InterfaceMethod(id) => package
            .interface_method(id)
            .and_then(|value| package.generated_type(value.owner))
            .is_some_and(|owner| dialect.is_public(&owner.visibility)),
        GeneratedSymbolId::Value(_) => false,
    }
}

fn violates_file_visibility<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    from: Option<SourceRole>,
    to: Option<SourceRole>,
    symbol: GeneratedSymbolId,
) -> bool {
    let (Some(from), Some(to)) = (from, to) else {
        return true;
    };
    let from_is_test = matches!(
        from,
        SourceRole::NativeTest | SourceRole::Conformance | SourceRole::NegativeTest
    );
    let to_is_test = matches!(
        to,
        SourceRole::NativeTest | SourceRole::Conformance | SourceRole::NegativeTest
    );
    (from == SourceRole::Runtime && to != SourceRole::Runtime)
        || (!from_is_test && to_is_test)
        || (from == SourceRole::PublicApi
            && (to == SourceRole::Implementation
                || !generated_symbol_is_public(dialect, package, symbol)))
}

fn find_file_cycle(
    graph: &BTreeMap<TargetFileId, BTreeSet<TargetFileId>>,
) -> Option<Vec<TargetFileId>> {
    fn visit(
        node: TargetFileId,
        graph: &BTreeMap<TargetFileId, BTreeSet<TargetFileId>>,
        states: &mut BTreeMap<TargetFileId, u8>,
        stack: &mut Vec<TargetFileId>,
    ) -> Option<Vec<TargetFileId>> {
        match states.get(&node) {
            Some(2) => return None,
            Some(1) => {
                let start = stack.iter().position(|candidate| candidate == &node)?;
                return Some(stack[start..].to_vec());
            }
            _ => {}
        }
        states.insert(node, 1);
        stack.push(node);
        if let Some(edges) = graph.get(&node) {
            for edge in edges {
                if let Some(cycle) = visit(*edge, graph, states, stack) {
                    return Some(cycle);
                }
            }
        }
        stack.pop();
        states.insert(node, 2);
        None
    }

    let mut states = BTreeMap::new();
    for node in graph.keys() {
        if let Some(cycle) = visit(*node, graph, &mut states, &mut Vec::new()) {
            return Some(cycle);
        }
    }
    None
}

#[derive(Clone)]
struct BindingCandidate<D: LinkerDialect> {
    symbol: BindableSymbolId<D>,
    requested: String,
    namespace: D::Namespace,
    scope: BindingScope,
    public: bool,
    source: SourceRef,
}

fn allocate_bindings<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    catalogue: &SymbolCatalogue<D>,
    helpers: &BTreeSet<D::HelperId>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ResolvedBinding<D>> {
    let mut candidates: Vec<BindingCandidate<D>> = Vec::new();
    for (index, value) in package.generated_types().enumerate() {
        candidates.push(BindingCandidate {
            symbol: BindableSymbolId::Generated(GeneratedSymbolId::Type(
                GeneratedTypeId::from_index(index),
            )),
            requested: value.name.clone(),
            namespace: dialect.type_namespace(&value.kind),
            scope: BindingScope::Package,
            public: dialect.is_public(&value.visibility),
            source: value.source.clone(),
        });
    }
    for (index, value) in package.callables().enumerate() {
        candidates.push(BindingCandidate {
            symbol: BindableSymbolId::Generated(GeneratedSymbolId::Callable(
                GeneratedCallableId::from_index(index),
            )),
            requested: value.name.clone(),
            namespace: dialect.callable_namespace(),
            scope: BindingScope::Package,
            public: dialect.is_public(&value.visibility),
            source: value.source.clone(),
        });
    }
    for (index, value) in package.interface_methods().enumerate() {
        candidates.push(BindingCandidate {
            symbol: BindableSymbolId::Generated(GeneratedSymbolId::InterfaceMethod(
                GeneratedInterfaceMethodId::from_index(index),
            )),
            requested: value.name.clone(),
            namespace: dialect.member_namespace(),
            scope: BindingScope::Type(value.owner),
            public: true,
            source: value.source.clone(),
        });
    }
    for (index, value) in package.values().enumerate() {
        candidates.push(BindingCandidate {
            symbol: BindableSymbolId::Generated(GeneratedSymbolId::Value(
                GeneratedValueId::from_index(index),
            )),
            requested: value.name.clone(),
            namespace: dialect.value_namespace(),
            scope: BindingScope::Package,
            public: false,
            source: value.source.clone(),
        });
    }
    for helper in helpers {
        if let Some(spec) = catalogue.helper(helper) {
            candidates.push(BindingCandidate {
                symbol: BindableSymbolId::Helper(helper.clone()),
                requested: spec.alias_stem.clone(),
                namespace: spec.namespace.clone(),
                scope: BindingScope::Package,
                public: false,
                source: spec.source.clone(),
            });
        }
    }
    candidates.sort_by_key(|candidate| (!candidate.public, candidate.symbol.clone()));

    let mut occupied = BTreeSet::new();
    let mut bindings = Vec::new();
    for candidate in candidates {
        let base =
            match dialect.identifier_from_candidate(&candidate.requested, &candidate.namespace) {
                Ok(identifier) => identifier,
                Err(violation) => {
                    diagnostics.push(Diagnostic::error(
                        violation.code,
                        violation.message,
                        candidate.source,
                    ));
                    continue;
                }
            };
        let base_key = (
            candidate.scope,
            candidate.namespace.clone(),
            dialect.identifier_key(&base),
        );
        let identifier = if occupied.insert(base_key) {
            base
        } else if candidate.public {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateDeclaration,
                "public target name collides in its namespace",
                candidate.source.clone(),
            ));
            continue;
        } else {
            let mut suffix = 2u32;
            loop {
                let renamed = format!("{}_{}", candidate.requested, suffix);
                match dialect.identifier_from_candidate(&renamed, &candidate.namespace) {
                    Ok(identifier) => {
                        let key = (
                            candidate.scope,
                            candidate.namespace.clone(),
                            dialect.identifier_key(&identifier),
                        );
                        if occupied.insert(key) {
                            break identifier;
                        }
                    }
                    Err(violation) => {
                        diagnostics.push(Diagnostic::error(
                            violation.code,
                            violation.message,
                            candidate.source.clone(),
                        ));
                        break base;
                    }
                }
                suffix += 1;
            }
        };
        bindings.push(ResolvedBinding {
            symbol: candidate.symbol,
            identifier,
            namespace: candidate.namespace,
            scope: candidate.scope,
            public: candidate.public,
            source: candidate.source,
        });
    }
    bindings
}

fn occupied_names<D: LinkerDialect>(
    dialect: &D,
    bindings: &[ResolvedBinding<D>],
    file: TargetFileId,
) -> BTreeSet<(D::Namespace, D::NameKey)> {
    bindings
        .iter()
        .filter(|binding| {
            matches!(binding.scope, BindingScope::Package)
                || binding.scope == BindingScope::File(file)
        })
        .map(|binding| {
            (
                binding.namespace.clone(),
                dialect.identifier_key(&binding.identifier),
            )
        })
        .collect()
}

struct ReferencePlan<D: LinkerDialect> {
    name: D::Identifier,
    alias_stem: String,
    namespace: D::Namespace,
    qualified_name: Option<D::QualifiedName>,
    origin: SymbolOrigin<D>,
    policy: DependencyPolicy<D>,
    dependency: Option<PackageRequirement<D>>,
}

#[allow(clippy::too_many_arguments)]
fn resolve_reference<D: LinkerDialect>(
    dialect: &D,
    catalogue: &SymbolCatalogue<D>,
    bindings: &[ResolvedBinding<D>],
    binding_lookup: &BTreeMap<BindableSymbolId<D>, usize>,
    imports: &mut Vec<ResolvedImport<D>>,
    import_lookup: &mut BTreeMap<TargetSymbolRef<D>, ResolvedImportId>,
    physical_import_lookup: &mut BTreeMap<(D::ImportKind, D::Identifier), ResolvedImportId>,
    occupied: &mut BTreeSet<(D::Namespace, D::NameKey)>,
    requirements: &mut BTreeMap<D::ExternalPackage, PackageRequirement<D>>,
    next_import: &mut usize,
    located: &LocatedSymbol<D>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ResolvedReference<D>> {
    let bindable = match &located.symbol {
        TargetSymbolRef::Generated(id) => Some(BindableSymbolId::Generated(*id)),
        TargetSymbolRef::RuntimeHelper(id) => Some(BindableSymbolId::Helper(id.clone())),
        TargetSymbolRef::Local(id) => Some(BindableSymbolId::Local(*id)),
        TargetSymbolRef::Package(id) => Some(BindableSymbolId::Package(*id)),
        TargetSymbolRef::File(id) => Some(BindableSymbolId::File(*id)),
        TargetSymbolRef::Test(id) => Some(BindableSymbolId::Test(*id)),
        _ => None,
    };
    if let Some(symbol) = bindable {
        return binding_lookup.get(&symbol).map_or_else(
            || {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnresolvedReference,
                    "generated/local/package/file/test/helper symbol has no binding",
                    located.source.clone(),
                ));
                None
            },
            |index| {
                Some(ResolvedReference::Local(
                    bindings[*index].identifier.clone(),
                ))
            },
        );
    }
    if matches!(located.symbol, TargetSymbolRef::TypeParameter(_)) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnresolvedReference,
            "target type parameter has no allocated binding",
            located.source.clone(),
        ));
        return None;
    }

    let Some(plan) = reference_plan(dialect, catalogue, &located.symbol) else {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::UnresolvedReference,
            "known symbol has no authoritative catalogue entry",
            located.source.clone(),
        ));
        return None;
    };
    if let Some(requirement) = &plan.dependency {
        match requirements.get(&requirement.package) {
            Some(existing) if existing != requirement => diagnostics.push(Diagnostic::error(
                DiagnosticCode::InterfaceNonconformance,
                "selected symbols require conflicting package versions or features",
                located.source.clone(),
            )),
            Some(_) => {}
            None => {
                requirements.insert(requirement.package.clone(), requirement.clone());
            }
        }
    }
    match plan.policy {
        DependencyPolicy::Implicit => Some(ResolvedReference::Local(plan.name)),
        DependencyPolicy::Qualified => plan
            .qualified_name
            .map(ResolvedReference::Qualified)
            .or_else(|| {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidStructure,
                    "qualified symbol has no typed qualified name",
                    located.source.clone(),
                ));
                None
            }),
        DependencyPolicy::Member { owner, member } => {
            Some(ResolvedReference::Member { owner, member })
        }
        DependencyPolicy::Import(kind) => {
            if let Some(id) = import_lookup.get(&located.symbol) {
                let import = imports.iter().find(|import| import.id == *id)?;
                return Some(ResolvedReference::Imported {
                    binding: import.binding.clone(),
                    import: *id,
                });
            }
            let physical_key = (kind.clone(), plan.name.clone());
            if let Some(id) = physical_import_lookup.get(&physical_key).copied() {
                let import = imports.iter_mut().find(|import| import.id == id)?;
                if import.origin != plan.origin {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::InterfaceNonconformance,
                        "one physical import was assigned conflicting typed origins",
                        located.source.clone(),
                    ));
                    return None;
                }
                import.symbols.insert(located.symbol.clone());
                import_lookup.insert(located.symbol.clone(), id);
                return Some(ResolvedReference::Imported {
                    binding: import.binding.clone(),
                    import: id,
                });
            }
            let key = (plan.namespace.clone(), dialect.identifier_key(&plan.name));
            let binding = if occupied.insert(key) {
                plan.name.clone()
            } else {
                let mut suffix = 2u32;
                loop {
                    let candidate = format!("{}_import_{}", plan.alias_stem, suffix);
                    match dialect.identifier_from_candidate(&candidate, &plan.namespace) {
                        Ok(identifier) => {
                            let key = (plan.namespace.clone(), dialect.identifier_key(&identifier));
                            if occupied.insert(key) {
                                break identifier;
                            }
                        }
                        Err(violation) => {
                            diagnostics.push(Diagnostic::error(
                                violation.code,
                                violation.message,
                                located.source.clone(),
                            ));
                            return None;
                        }
                    }
                    suffix += 1;
                }
            };
            let id = ResolvedImportId::from_index(*next_import);
            *next_import += 1;
            imports.push(ResolvedImport {
                id,
                symbols: BTreeSet::from([located.symbol.clone()]),
                original_binding: plan.name,
                binding: binding.clone(),
                kind,
                origin: plan.origin,
            });
            import_lookup.insert(located.symbol.clone(), id);
            physical_import_lookup.insert(physical_key, id);
            Some(ResolvedReference::Imported {
                binding,
                import: id,
            })
        }
    }
}

fn reference_plan<D: LinkerDialect>(
    dialect: &D,
    catalogue: &SymbolCatalogue<D>,
    symbol: &TargetSymbolRef<D>,
) -> Option<ReferencePlan<D>> {
    match symbol {
        TargetSymbolRef::KnownType(id) => catalogue
            .types
            .iter()
            .find(|spec| &spec.symbol == id)
            .map(|spec| ReferencePlan {
                name: spec.name.clone(),
                alias_stem: spec.alias_stem.clone(),
                namespace: dialect.type_namespace_from_known(&spec.symbol),
                qualified_name: spec.qualified_name.clone(),
                origin: spec.origin.clone(),
                policy: spec.policy.clone(),
                dependency: spec.dependency.clone(),
            }),
        TargetSymbolRef::KnownCallable(id) => catalogue
            .callables
            .iter()
            .find(|spec| &spec.symbol == id)
            .map(|spec| ReferencePlan {
                name: spec.name.clone(),
                alias_stem: spec.alias_stem.clone(),
                namespace: dialect.callable_namespace(),
                qualified_name: spec.qualified_name.clone(),
                origin: spec.origin.clone(),
                policy: spec.policy.clone(),
                dependency: spec.dependency.clone(),
            }),
        TargetSymbolRef::RuntimeCallable(id) => catalogue
            .runtime_callables
            .iter()
            .find(|spec| &spec.symbol == id)
            .map(|spec| ReferencePlan {
                name: spec.name.clone(),
                alias_stem: spec.alias_stem.clone(),
                namespace: dialect.callable_namespace(),
                qualified_name: spec.qualified_name.clone(),
                origin: spec.origin.clone(),
                policy: spec.policy.clone(),
                dependency: spec.dependency.clone(),
            }),
        TargetSymbolRef::KnownField(id) => catalogue
            .fields
            .iter()
            .find(|spec| &spec.symbol == id)
            .map(|spec| ReferencePlan {
                name: spec.name.clone(),
                alias_stem: "field".to_owned(),
                namespace: dialect.member_namespace(),
                qualified_name: None,
                origin: spec.origin.clone(),
                policy: spec.policy.clone(),
                dependency: spec.dependency.clone(),
            }),
        TargetSymbolRef::KnownConstructor(id) => catalogue
            .constructors
            .iter()
            .find(|spec| &spec.symbol == id)
            .map(|spec| ReferencePlan {
                name: spec.name.clone(),
                alias_stem: spec.alias_stem.clone(),
                namespace: dialect.callable_namespace(),
                qualified_name: spec.qualified_name.clone(),
                origin: spec.origin.clone(),
                policy: spec.policy.clone(),
                dependency: spec.dependency.clone(),
            }),
        TargetSymbolRef::KnownMethod(id) => catalogue
            .methods
            .iter()
            .find(|spec| &spec.symbol == id)
            .map(|spec| ReferencePlan {
                name: spec.name.clone(),
                alias_stem: "method".to_owned(),
                namespace: dialect.member_namespace(),
                qualified_name: None,
                origin: spec.origin.clone(),
                policy: spec.policy.clone(),
                dependency: spec.dependency.clone(),
            }),
        _ => None,
    }
}

pub fn verify_linked_package<D: LinkerDialect>(
    package: &LinkedTargetPackage<D>,
) -> Result<(), Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    if let Err(mut errors) = verify_target_ast(&package.unresolved) {
        diagnostics.append(&mut errors);
    }
    if let Err(mut errors) = package.catalogue.verify(&package.dialect) {
        diagnostics.append(&mut errors);
    }
    let mut names = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    for binding in &package.bindings {
        if !symbols.insert(binding.symbol.clone()) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateDeclaration,
                "resolved symbol has more than one binding",
                binding.source.clone(),
            ));
        }
        let key = (
            binding.scope,
            binding.namespace.clone(),
            package.dialect.identifier_key(&binding.identifier),
        );
        if !names.insert(key) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateDeclaration,
                "resolved bindings collide in a target namespace",
                binding.source.clone(),
            ));
        }
    }
    let mut files = BTreeSet::new();
    let mut referenced_imports = BTreeSet::new();
    let mut declared_imports = BTreeSet::new();
    let mut expected_dependencies = BTreeMap::new();
    let mut helper_roots = BTreeSet::new();
    let declaration_files = package
        .unresolved
        .files()
        .enumerate()
        .flat_map(|(file_index, file)| {
            file.items()
                .iter()
                .flat_map(|item| package.dialect.file_item_roots(item).declarations)
                .map(move |symbol| (symbol, TargetFileId::from_index(file_index)))
        })
        .collect::<BTreeMap<_, _>>();
    for file in &package.files {
        if !files.insert(file.file) {
            diagnostics.push(link_error(
                DiagnosticCode::DuplicateDeclaration,
                "resolved file appears more than once",
                "files",
            ));
        }
        let mut file_imports = BTreeMap::new();
        let mut physical_imports = BTreeSet::new();
        let mut expected_import_symbols =
            BTreeMap::<ResolvedImportId, BTreeSet<TargetSymbolRef<D>>>::new();
        let mut expected_file_dependencies = BTreeSet::new();
        for import in &file.imports {
            if !declared_imports.insert(import.id) {
                diagnostics.push(link_error(
                    DiagnosticCode::DuplicateDeclaration,
                    "resolved import ID appears more than once",
                    "imports",
                ));
            }
            if !physical_imports.insert((import.kind.clone(), import.original_binding.clone())) {
                diagnostics.push(link_error(
                    DiagnosticCode::DuplicateDeclaration,
                    "one physical import appears more than once in a file",
                    "imports",
                ));
            }
            file_imports.insert(import.id, import);
        }
        for reference in &file.references {
            if let ResolvedReference::Imported { import, .. } = reference.resolved {
                referenced_imports.insert(import);
                expected_import_symbols
                    .entry(import)
                    .or_default()
                    .insert(reference.symbol.clone());
            }
            if !resolved_reference_matches(package, reference, &file_imports) {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InterfaceNonconformance,
                    "resolved reference does not match its typed symbol or import",
                    reference.source.clone(),
                ));
            }
            if let TargetSymbolRef::RuntimeHelper(helper) = &reference.symbol {
                helper_roots.insert(helper.clone());
            }
            if let TargetSymbolRef::Generated(symbol) = &reference.symbol
                && let Some(destination) = declaration_files.get(symbol)
                && destination != &file.file
            {
                expected_file_dependencies.insert(*destination);
            }
            if let Some(plan) =
                reference_plan(&package.dialect, &package.catalogue, &reference.symbol)
                && let Some(requirement) = plan.dependency
            {
                match expected_dependencies.get(&requirement.package) {
                    Some(existing) if existing != &requirement => {
                        diagnostics.push(Diagnostic::error(
                            DiagnosticCode::InterfaceNonconformance,
                            "resolved references retain conflicting dependency requirements",
                            reference.source.clone(),
                        ));
                    }
                    Some(_) => {}
                    None => {
                        expected_dependencies.insert(requirement.package.clone(), requirement);
                    }
                }
            }
        }
        for import in &file.imports {
            if expected_import_symbols.get(&import.id) != Some(&import.symbols) {
                diagnostics.push(link_error(
                    DiagnosticCode::InterfaceNonconformance,
                    "resolved import symbol membership is not exactly reference-derived",
                    "imports",
                ));
            }
        }
        let symbols = file
            .references
            .iter()
            .map(|reference| reference.symbol.clone())
            .collect::<Vec<_>>();
        if file.forward_declarations != package.dialect.forward_declarations(file.file, &symbols) {
            diagnostics.push(link_error(
                DiagnosticCode::InterfaceNonconformance,
                "forward declarations are not resolver-derived",
                "forward-declarations",
            ));
        }
        if file.dependencies != expected_file_dependencies.into_iter().collect::<Vec<_>>() {
            diagnostics.push(link_error(
                DiagnosticCode::InterfaceNonconformance,
                "cross-file dependency edges are not exactly reference-derived",
                "file-dependencies",
            ));
        }
        let Some(source_file) = package.unresolved.file(file.file) else {
            diagnostics.push(link_error(
                DiagnosticCode::UnresolvedReference,
                "resolved file has no unresolved source-file identity",
                "files",
            ));
            continue;
        };
        if file.path != *source_file.path()
            || file.role != source_file.role()
            || file.placement != *source_file.placement()
            || file.template != *source_file.template()
            || file.source != *source_file.source()
        {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InterfaceNonconformance,
                "resolved file metadata does not match structural source-file metadata",
                file.source.clone(),
            ));
        }
        match package.dialect.resolve_module(source_file.module()) {
            Ok(expected) if expected == file.module => {}
            _ => diagnostics.push(Diagnostic::error(
                DiagnosticCode::InterfaceNonconformance,
                "resolved module does not match dialect resolution",
                file.source.clone(),
            )),
        }
        let reference_map =
            ResolvedReferenceMap::from_linked(&file.references, &file.source, &mut diagnostics);
        let mut expected_items = Vec::new();
        let unresolved_items = source_file
            .items()
            .iter()
            .map(|item| (item, source_file.source()))
            .chain(
                package
                    .helpers
                    .iter()
                    .filter(|helper| helper.file == file.file)
                    .flat_map(|helper| helper.items.iter().map(move |item| (item, &helper.source))),
            );
        for (item, source) in unresolved_items {
            match package
                .dialect
                .resolve_file_item(&package.unresolved, item, &reference_map)
            {
                Ok(item) => {
                    diagnostics.extend(
                        package
                            .dialect
                            .verify_resolved_file_item(&item)
                            .into_iter()
                            .map(|violation| {
                                Diagnostic::error(violation.code, violation.message, source.clone())
                            }),
                    );
                    expected_items.push(item);
                }
                Err(violation) => diagnostics.push(Diagnostic::error(
                    violation.code,
                    violation.message,
                    source.clone(),
                )),
            }
        }
        if file.items != expected_items {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InterfaceNonconformance,
                "resolved file items are not the exact dialect rewrite of source/helper AST",
                file.source.clone(),
            ));
        }
    }
    if files.len() != package.unresolved.files().len() {
        diagnostics.push(link_error(
            DiagnosticCode::InvalidStructure,
            "resolved package does not contain every unresolved file exactly once",
            "files",
        ));
    }
    if referenced_imports != declared_imports {
        diagnostics.push(link_error(
            DiagnosticCode::InterfaceNonconformance,
            "resolved imports are not exactly reference-derived",
            "imports",
        ));
    }
    let actual_dependencies = package
        .dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.requirement.package.clone(),
                dependency.requirement.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if actual_dependencies != expected_dependencies {
        diagnostics.push(link_error(
            DiagnosticCode::InterfaceNonconformance,
            "resolved dependencies are not exactly reference-derived",
            "dependencies",
        ));
    }
    let actual_helpers = package
        .helpers
        .iter()
        .map(|helper| helper.id.clone())
        .collect::<BTreeSet<_>>();
    if actual_helpers.len() != package.helpers.len() || helper_roots != actual_helpers {
        diagnostics.push(link_error(
            DiagnosticCode::InterfaceNonconformance,
            "resolved helper set is missing roots or contains duplicates",
            "helpers",
        ));
    }
    if package
        .helpers
        .windows(2)
        .any(|pair| (pair[0].order, &pair[0].id) >= (pair[1].order, &pair[1].id))
    {
        diagnostics.push(link_error(
            DiagnosticCode::InterfaceNonconformance,
            "resolved helpers are not in deterministic catalogue order",
            "helpers",
        ));
    }
    for helper in &package.helpers {
        let Some(spec) = package.catalogue.helper(&helper.id) else {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnresolvedReference,
                "resolved helper has no typed catalogue declaration",
                helper.source.clone(),
            ));
            continue;
        };
        if helper.capability != spec.capability
            || helper.order != spec.order
            || helper.items != spec.items
            || helper.source != spec.source
        {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InterfaceNonconformance,
                "resolved helper does not match its structural catalogue declaration",
                helper.source.clone(),
            ));
        }
        match package.unresolved.file(helper.file) {
            Some(file)
                if file.role() == SourceRole::Runtime && file.placement() == &spec.placement => {}
            _ => diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                "resolved helper is not placed in its typed runtime file",
                helper.source.clone(),
            )),
        }
    }
    for file in &package.files {
        let expected = package
            .helpers
            .iter()
            .filter(|helper| helper.file == file.file)
            .map(|helper| helper.id.clone())
            .collect::<Vec<_>>();
        if file.helpers != expected {
            diagnostics.push(link_error(
                DiagnosticCode::InterfaceNonconformance,
                "runtime helper declarations are not exactly file-placement-derived",
                "helpers",
            ));
        }
    }
    sort_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn resolved_reference_matches<D: LinkerDialect>(
    package: &LinkedTargetPackage<D>,
    reference: &LinkedReference<D>,
    imports: &BTreeMap<ResolvedImportId, &ResolvedImport<D>>,
) -> bool {
    let bindable = match &reference.symbol {
        TargetSymbolRef::Generated(id) => Some(BindableSymbolId::Generated(*id)),
        TargetSymbolRef::RuntimeHelper(id) => Some(BindableSymbolId::Helper(id.clone())),
        TargetSymbolRef::Local(id) => Some(BindableSymbolId::Local(*id)),
        TargetSymbolRef::Package(id) => Some(BindableSymbolId::Package(*id)),
        TargetSymbolRef::File(id) => Some(BindableSymbolId::File(*id)),
        TargetSymbolRef::Test(id) => Some(BindableSymbolId::Test(*id)),
        _ => None,
    };
    if let Some(symbol) = bindable {
        let expected = package
            .bindings
            .iter()
            .find(|binding| binding.symbol == symbol)
            .map(|binding| &binding.identifier);
        return matches!((&reference.resolved, expected), (ResolvedReference::Local(actual), Some(expected)) if actual == expected);
    }
    let Some(plan) = reference_plan(&package.dialect, &package.catalogue, &reference.symbol) else {
        return false;
    };
    match (&reference.resolved, plan.policy) {
        (ResolvedReference::Local(actual), DependencyPolicy::Implicit) => actual == &plan.name,
        (ResolvedReference::Qualified(actual), DependencyPolicy::Qualified) => {
            Some(actual) == plan.qualified_name.as_ref()
        }
        (
            ResolvedReference::Member {
                owner: actual_owner,
                member: actual_member,
            },
            DependencyPolicy::Member { owner, member },
        ) => actual_owner == &owner && actual_member == &member,
        (ResolvedReference::Imported { binding, import }, DependencyPolicy::Import(kind)) => {
            imports.get(import).is_some_and(|record| {
                record.symbols.contains(&reference.symbol)
                    && record.original_binding == plan.name
                    && record.kind == kind
                    && record.origin == plan.origin
                    && &record.binding == binding
            })
        }
        _ => false,
    }
}

fn link_error(code: DiagnosticCode, message: &str, category: &str) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        SourceRef::logical(["target-linker", category]),
    )
}

fn check_unique<'a, K: Ord + 'a>(
    diagnostics: &mut Vec<Diagnostic>,
    values: impl Iterator<Item = (&'a K, &'a SourceRef)>,
    category: &str,
) {
    let mut seen = BTreeSet::new();
    for (value, source) in values {
        if !seen.insert(value) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateDeclaration,
                format!("duplicate {category} catalogue entry"),
                source.clone(),
            ));
        }
    }
}

fn validate_callable_pattern<D: LinkerDialect>(
    diagnostics: &mut Vec<Diagnostic>,
    pattern: &CallablePattern<D>,
    source: &SourceRef,
) {
    let count = pattern.type_parameters.len();
    let mut names = BTreeSet::new();
    for parameter in &pattern.type_parameters {
        if !names.insert(parameter.name.clone()) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DuplicateDeclaration,
                "callable type-parameter names are not unique",
                source.clone(),
            ));
        }
    }
    for ty in pattern
        .receiver
        .iter()
        .chain(&pattern.parameters)
        .chain(std::iter::once(&pattern.result))
    {
        validate_pattern_parameter(diagnostics, ty, count, source);
    }
}

fn validate_pattern_parameter<D: LinkerDialect>(
    diagnostics: &mut Vec<Diagnostic>,
    pattern: &TypePattern<D>,
    count: usize,
    source: &SourceRef,
) {
    match pattern {
        TypePattern::Parameter(index) if usize::from(*index) >= count => {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnresolvedReference,
                "type pattern refers to a missing catalogue type parameter",
                source.clone(),
            ));
        }
        TypePattern::Constructed { arguments, .. } => {
            for argument in arguments {
                validate_pattern_parameter(diagnostics, argument, count, source);
            }
        }
        TypePattern::Exact(_) | TypePattern::Parameter(_) => {}
    }
}

fn concrete_signature<D: LinkerDialect>(
    pattern: &CallablePattern<D>,
) -> Option<TargetCallableSignature<D>> {
    if !pattern.type_parameters.is_empty() {
        return None;
    }
    fn exact<D: LinkerDialect>(pattern: &TypePattern<D>) -> Option<TargetTypeRef<D>> {
        match pattern {
            TypePattern::Exact(ty) => Some(ty.clone()),
            TypePattern::Parameter(_) | TypePattern::Constructed { .. } => None,
        }
    }
    Some(TargetCallableSignature {
        invocation: pattern.invocation.clone(),
        receiver: match &pattern.receiver {
            Some(receiver) => Some(exact(receiver)?),
            None => None,
        },
        parameters: pattern
            .parameters
            .iter()
            .map(exact)
            .collect::<Option<Vec<_>>>()?,
        return_type: exact(&pattern.result)?,
    })
}

fn validate_dependency<D: LinkerDialect>(
    diagnostics: &mut Vec<Diagnostic>,
    origin: &SymbolOrigin<D>,
    policy: &DependencyPolicy<D>,
    qualified_name: Option<&D::QualifiedName>,
    dependency: Option<&PackageRequirement<D>>,
    source: &SourceRef,
) {
    if matches!(policy, DependencyPolicy::Qualified) && qualified_name.is_none() {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            "qualified dependency policy requires a typed qualified name",
            source.clone(),
        ));
    }
    match (origin, dependency) {
        (SymbolOrigin::ExternalPackage(package), Some(requirement))
            if package == &requirement.package => {}
        (SymbolOrigin::ExternalPackage(_), _) => diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            "external symbol must carry its matching package requirement",
            source.clone(),
        )),
        (_, Some(_)) => diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            "non-external symbol cannot add a package dependency",
            source.clone(),
        )),
        (_, None) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CertifiedTemplateEngine, CertifiedTemplateId, EmbeddedTemplate, FileGroupRole,
        GeneratedCallable, GeneratedOrigin, GeneratedType, GeneratedValue, RelativeOutputPath,
        ResolvedTemplateRenderer, SourceRole, SynthesisReason, TargetExpressionNode, TargetFile,
        TargetFileGroup, TargetFileItemNode, TargetFileMember, TargetStatementNode,
        render_linked_package,
    };
    use serde::Serialize;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum CatalogueMode {
        Normal,
        Duplicate,
        BadSignature,
        MissingHelper,
        HelperCycle,
        HelperIllegalPlacement,
        PublicHelper,
        DuplicateHelperOrder,
        DuplicateHelper,
        PermittedFileCycle,
        DependencyConflict,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct TestDialect(CatalogueMode);

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Primitive {
        Bool,
        I64,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum KnownType {
        Clock,
        Uncatalogued,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum RuntimeType {
        Error,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum ConstructedType {
        List,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum KnownCallable {
        Zero,
        Negate,
        Maximum,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum RuntimeCallable {
        IsPositive,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Invocation {
        Static,
        Instance,
        Constructor,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Visibility {
        Private,
        Public,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum DeclarationKind {
        Record,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum AstOrigin {
        Runtime,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Template {
        Source,
        Declaration,
    }

    impl CertifiedTemplateId for Template {
        fn all() -> &'static [Self] {
            const ALL: &[Template] = &[Template::Source, Template::Declaration];
            ALL
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Module {
        Generated,
        Runtime,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Placement {
        Implementation,
        Runtime,
        MissingRuntime,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum KnownField {
        Epoch,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum KnownConstructor {
        NewClock,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum KnownMethod {
        Elapsed,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Prelude {
        Integer,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum StandardLibrary {
        Time,
        Runtime,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum ExternalPackage {
        Math,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum PackageFeature {
        Fast,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Helper {
        Root,
        Leaf,
        Cycle,
        Missing,
        Unused,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum HelperCapability {
        Arithmetic,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Identifier(String);

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum QualifiedName {
        RuntimeIsPositive,
        StdNegate,
        StdClock,
        MathClock,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum MemberName {
        Epoch,
        Elapsed,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum Namespace {
        Type,
        Value,
        Member,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum ImportKind {
        Type,
        Value,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Expression {
        I64(i64),
        Call {
            callable: KnownCallable,
            invocation: Invocation,
            arguments: Vec<TargetExprId>,
        },
        Construct {
            constructor: KnownConstructor,
            arguments: Vec<TargetExprId>,
        },
        Method {
            method: KnownMethod,
            receiver: TargetExprId,
            arguments: Vec<TargetExprId>,
        },
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Statement {
        expression: TargetExprId,
        symbols: Vec<TargetSymbolRef<TestDialect>>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum FileItem {
        Root(TargetStmtId),
        Declaration(GeneratedSymbolId),
        RuntimeDeclaration {
            helper: Helper,
            symbols: Vec<TargetSymbolRef<TestDialect>>,
        },
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum ResolvedItemKind {
        Root,
        Declaration(GeneratedSymbolId),
        RuntimeDeclaration(Helper),
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct ResolvedItem {
        kind: ResolvedItemKind,
        references: Vec<ResolvedReference<TestDialect>>,
    }

    #[derive(Serialize)]
    struct RenderedSourceView {
        declarations: Vec<String>,
    }

    #[derive(Serialize)]
    struct RenderedDeclarationView {
        keyword: &'static str,
        name: &'static str,
    }

    struct TestRenderer;

    impl ResolvedTemplateRenderer<TestDialect> for TestRenderer {
        type FileView = RenderedSourceView;

        fn target_name(&self) -> &'static str {
            "linker-test"
        }

        fn templates(&self) -> Vec<EmbeddedTemplate<Template>> {
            vec![
                EmbeddedTemplate::new(
                    Template::Source,
                    "{{#each declarations}}{{this}}\n{{/each}}",
                    &["declarations"],
                ),
                EmbeddedTemplate::new(
                    Template::Declaration,
                    "{{keyword}} {{name}};",
                    &["keyword", "name"],
                ),
            ]
        }

        fn build_file_view(
            &self,
            _package: &LinkedTargetPackage<TestDialect>,
            file: &LinkedFile<TestDialect>,
            templates: &mut CertifiedTemplateEngine<Template>,
        ) -> Result<Self::FileView, Vec<Diagnostic>> {
            let mut declarations = Vec::new();
            for item in file.items() {
                let name = match &item.kind {
                    ResolvedItemKind::Root => "root",
                    ResolvedItemKind::Declaration(_) => "user",
                    ResolvedItemKind::RuntimeDeclaration(_) => "runtime",
                };
                declarations.push(templates.render(
                    &Template::Declaration,
                    &RenderedDeclarationView {
                        keyword: "declaration",
                        name,
                    },
                    self.target_name(),
                    file,
                )?);
            }
            Ok(RenderedSourceView { declarations })
        }
    }

    struct I64Marker;
    struct ClockMarker;
    struct ListMarker;
    struct ZeroCall;
    struct NegateCall;
    struct MaximumCall;
    struct ClockConstructor;
    struct ElapsedCall;

    impl TargetTypeMarker<TestDialect> for I64Marker {
        fn target_type() -> TargetTypeRef<TestDialect> {
            i64_type()
        }
    }

    impl TargetTypeMarker<TestDialect> for ClockMarker {
        fn target_type() -> TargetTypeRef<TestDialect> {
            TargetTypeRef::Known(KnownType::Clock)
        }
    }

    impl TargetTypeMarker<TestDialect> for ListMarker {
        fn target_type() -> TargetTypeRef<TestDialect> {
            TargetTypeRef::Constructed(ConstructedType::List)
        }
    }

    impl KnownNullaryCall<TestDialect> for ZeroCall {
        type Result = I64Marker;

        fn callable() -> KnownCallable {
            KnownCallable::Zero
        }
    }

    impl KnownUnaryCall<TestDialect> for NegateCall {
        type Argument = I64Marker;
        type Result = I64Marker;

        fn callable() -> KnownCallable {
            KnownCallable::Negate
        }
    }

    impl KnownBinaryCall<TestDialect> for MaximumCall {
        type Left = I64Marker;
        type Right = I64Marker;
        type Result = I64Marker;

        fn callable() -> KnownCallable {
            KnownCallable::Maximum
        }
    }

    impl KnownNullaryConstructor<TestDialect> for ClockConstructor {
        type Result = ClockMarker;

        fn constructor() -> KnownConstructor {
            KnownConstructor::NewClock
        }
    }

    impl KnownInstanceUnaryCall<TestDialect> for ElapsedCall {
        type Receiver = ClockMarker;
        type Argument = I64Marker;
        type Result = ListMarker;

        fn method() -> KnownMethod {
            KnownMethod::Elapsed
        }
    }

    impl TargetDialect for TestDialect {
        type Unresolved = TargetAstPackage<Self>;
        type Resolved = LinkedTargetPackage<Self>;

        fn verify_unresolved(&self, ast: &Self::Unresolved) -> Result<(), Vec<Diagnostic>> {
            verify_target_ast(ast)
        }

        fn verify_resolved(&self, ast: &Self::Resolved) -> Result<(), Vec<Diagnostic>> {
            verify_linked_package(ast)
        }
    }

    impl TypedAstDialect for TestDialect {
        type PrimitiveType = Primitive;
        type KnownType = KnownType;
        type RuntimeType = RuntimeType;
        type ConstructedType = ConstructedType;
        type KnownCallable = KnownCallable;
        type RuntimeCallable = RuntimeCallable;
        type InvocationKind = Invocation;
        type Visibility = Visibility;
        type DeclarationKind = DeclarationKind;
        type SymbolOrigin = AstOrigin;
        type TemplateId = Template;
        type ModuleDeclaration = Module;
        type FilePlacement = Placement;
        type Expression = Expression;
        type Statement = Statement;
        type FileItem = FileItem;

        fn known_callable_signature(
            &self,
            callable: &Self::KnownCallable,
        ) -> TargetCallableSignature<Self> {
            match callable {
                KnownCallable::Zero => signature(vec![], i64_type()),
                KnownCallable::Negate => signature(vec![i64_type()], i64_type()),
                KnownCallable::Maximum => signature(vec![i64_type(), i64_type()], i64_type()),
            }
        }

        fn runtime_callable_signature(
            &self,
            callable: &Self::RuntimeCallable,
        ) -> TargetCallableSignature<Self> {
            match callable {
                RuntimeCallable::IsPositive => signature(vec![i64_type()], bool_type()),
            }
        }

        fn verify_signature(&self, signature: &TargetCallableSignature<Self>) -> Vec<AstViolation> {
            let valid = match signature.invocation {
                Invocation::Instance => signature.receiver.is_some(),
                Invocation::Static | Invocation::Constructor => signature.receiver.is_none(),
            };
            if valid {
                vec![]
            } else {
                vec![AstViolation::new(
                    DiagnosticCode::InvalidInvocation,
                    "receiver/invocation mismatch",
                )]
            }
        }

        fn verify_source_file(
            &self,
            _file: &TargetFile<Self>,
            _context: &crate::TargetAstContext<'_, Self>,
        ) -> Vec<AstViolation> {
            vec![]
        }
    }

    impl TargetExpressionNode<TestDialect> for Expression {
        fn child_expressions(&self) -> Vec<TargetExprId> {
            match self {
                Self::I64(_) => vec![],
                Self::Call { arguments, .. } | Self::Construct { arguments, .. } => {
                    arguments.clone()
                }
                Self::Method {
                    receiver,
                    arguments,
                    ..
                } => std::iter::once(*receiver)
                    .chain(arguments.iter().copied())
                    .collect(),
            }
        }

        fn verify(
            &self,
            stored_type: &TargetTypeRef<TestDialect>,
            context: &crate::TargetAstContext<'_, TestDialect>,
        ) -> Vec<AstViolation> {
            match self {
                Self::I64(value) => {
                    let _observed = *value;
                    type_errors(stored_type, &i64_type())
                }
                Self::Call {
                    callable,
                    invocation,
                    arguments,
                } => {
                    let signature = context.dialect().known_callable_signature(callable);
                    let mut errors = type_errors(stored_type, &signature.return_type);
                    if invocation != &signature.invocation
                        || arguments.len() != signature.parameters.len()
                    {
                        errors.push(AstViolation::new(
                            DiagnosticCode::InvalidInvocation,
                            "known call shape mismatch",
                        ));
                    }
                    for (argument, expected) in arguments.iter().zip(signature.parameters) {
                        if context.expression_type(*argument) != Some(&expected) {
                            errors.push(AstViolation::new(
                                DiagnosticCode::TypeMismatch,
                                "known call argument mismatch",
                            ));
                        }
                    }
                    errors
                }
                Self::Construct {
                    constructor,
                    arguments,
                } => {
                    let mut errors =
                        type_errors(stored_type, &TargetTypeRef::Known(KnownType::Clock));
                    if constructor != &KnownConstructor::NewClock || !arguments.is_empty() {
                        errors.push(AstViolation::new(
                            DiagnosticCode::InvalidInvocation,
                            "constructor shape mismatch",
                        ));
                    }
                    errors
                }
                Self::Method {
                    method,
                    receiver,
                    arguments,
                } => {
                    let mut errors = type_errors(
                        stored_type,
                        &TargetTypeRef::Constructed(ConstructedType::List),
                    );
                    if method != &KnownMethod::Elapsed
                        || context.expression_type(*receiver)
                            != Some(&TargetTypeRef::Known(KnownType::Clock))
                        || arguments.len() != 1
                        || context.expression_type(arguments[0]) != Some(&i64_type())
                    {
                        errors.push(AstViolation::new(
                            DiagnosticCode::InvalidInvocation,
                            "instance method shape mismatch",
                        ));
                    }
                    errors
                }
            }
        }
    }

    impl TargetStatementNode<TestDialect> for Statement {
        fn child_expressions(&self) -> Vec<TargetExprId> {
            vec![self.expression]
        }

        fn verify(&self, _context: &crate::TargetAstContext<'_, TestDialect>) -> Vec<AstViolation> {
            vec![]
        }
    }

    impl TargetFileItemNode<TestDialect> for FileItem {
        fn verify(&self, context: &crate::TargetAstContext<'_, TestDialect>) -> Vec<AstViolation> {
            match self {
                Self::Root(statement) if !context.contains_statement(*statement) => {
                    vec![AstViolation::new(
                        DiagnosticCode::UnresolvedReference,
                        "file root statement is missing",
                    )]
                }
                Self::Root(_) | Self::Declaration(_) | Self::RuntimeDeclaration { .. } => vec![],
            }
        }
    }

    impl LinkerDialect for TestDialect {
        type KnownField = KnownField;
        type KnownConstructor = KnownConstructor;
        type KnownMethod = KnownMethod;
        type PreludeSymbol = Prelude;
        type StandardLibrary = StandardLibrary;
        type ExternalPackage = ExternalPackage;
        type PackageFeature = PackageFeature;
        type HelperId = Helper;
        type HelperCapability = HelperCapability;
        type Identifier = Identifier;
        type QualifiedName = QualifiedName;
        type MemberName = MemberName;
        type Namespace = Namespace;
        type NameKey = String;
        type ImportKind = ImportKind;
        type ResolvedModule = Module;
        type ResolvedFileItem = ResolvedItem;

        fn package_ecosystem(&self, package: &Self::ExternalPackage) -> PackageEcosystem {
            match package {
                ExternalPackage::Math => PackageEcosystem::Cargo,
            }
        }

        fn package_name(&self, package: &Self::ExternalPackage) -> &'static str {
            match package {
                ExternalPackage::Math => "math",
            }
        }

        fn package_feature_name(&self, feature: &Self::PackageFeature) -> &'static str {
            match feature {
                PackageFeature::Fast => "fast",
            }
        }

        fn helper_name(&self, helper: &Self::HelperId) -> &'static str {
            match helper {
                Helper::Root => "root",
                Helper::Leaf => "leaf",
                Helper::Cycle => "cycle",
                Helper::Missing => "missing",
                Helper::Unused => "unused",
            }
        }

        fn helper_capability_name(&self, capability: &Self::HelperCapability) -> &'static str {
            match capability {
                HelperCapability::Arithmetic => "arithmetic",
            }
        }

        fn symbol_catalogue(&self) -> SymbolCatalogue<Self> {
            catalogue(self.0)
        }

        fn identifier_from_candidate(
            &self,
            candidate: &str,
            _namespace: &Self::Namespace,
        ) -> Result<Self::Identifier, AstViolation> {
            let valid = !candidate.is_empty()
                && candidate
                    .chars()
                    .all(|character| character == '_' || character.is_ascii_alphanumeric())
                && !candidate.as_bytes()[0].is_ascii_digit();
            if valid {
                Ok(Identifier(candidate.to_owned()))
            } else {
                Err(AstViolation::new(
                    DiagnosticCode::InvalidIdentifier,
                    "invalid test-dialect identifier",
                ))
            }
        }

        fn identifier_key(&self, identifier: &Self::Identifier) -> Self::NameKey {
            identifier.0.to_ascii_lowercase()
        }

        fn is_public(&self, visibility: &Self::Visibility) -> bool {
            matches!(visibility, Visibility::Public)
        }

        fn type_namespace(&self, _kind: &Self::DeclarationKind) -> Self::Namespace {
            Namespace::Type
        }

        fn type_namespace_from_known(&self, _known: &Self::KnownType) -> Self::Namespace {
            Namespace::Type
        }

        fn callable_namespace(&self) -> Self::Namespace {
            Namespace::Value
        }

        fn member_namespace(&self) -> Self::Namespace {
            Namespace::Member
        }

        fn value_namespace(&self) -> Self::Namespace {
            Namespace::Value
        }

        fn known_call_expression(
            &self,
            callable: Self::KnownCallable,
            invocation: Self::InvocationKind,
            arguments: Vec<TargetExprId>,
        ) -> Self::Expression {
            Expression::Call {
                callable,
                invocation,
                arguments,
            }
        }

        fn known_constructor_expression(
            &self,
            constructor: Self::KnownConstructor,
            arguments: Vec<TargetExprId>,
        ) -> Self::Expression {
            Expression::Construct {
                constructor,
                arguments,
            }
        }

        fn known_method_expression(
            &self,
            method: Self::KnownMethod,
            receiver: TargetExprId,
            arguments: Vec<TargetExprId>,
        ) -> Self::Expression {
            Expression::Method {
                method,
                receiver,
                arguments,
            }
        }

        fn expression_references(
            &self,
            expression: &Self::Expression,
        ) -> Vec<TargetSymbolRef<Self>> {
            match expression {
                Expression::Call { callable, .. } => {
                    vec![TargetSymbolRef::KnownCallable(callable.clone())]
                }
                Expression::Construct { constructor, .. } => {
                    vec![TargetSymbolRef::KnownConstructor(constructor.clone())]
                }
                Expression::Method { method, .. } => {
                    vec![TargetSymbolRef::KnownMethod(method.clone())]
                }
                Expression::I64(_) => vec![],
            }
        }

        fn statement_references(&self, statement: &Self::Statement) -> Vec<TargetSymbolRef<Self>> {
            statement.symbols.clone()
        }

        fn file_item_roots(&self, item: &Self::FileItem) -> FileItemRoots<Self> {
            match item {
                FileItem::Root(statement) => FileItemRoots {
                    statements: vec![*statement],
                    ..FileItemRoots::default()
                },
                FileItem::Declaration(symbol) => FileItemRoots {
                    declarations: vec![*symbol],
                    ..FileItemRoots::default()
                },
                FileItem::RuntimeDeclaration { helper, symbols } => {
                    let _typed_identity = helper;
                    FileItemRoots {
                        symbols: symbols.clone(),
                        ..FileItemRoots::default()
                    }
                }
            }
        }

        fn resolve_module(
            &self,
            module: &Self::ModuleDeclaration,
        ) -> Result<Self::ResolvedModule, AstViolation> {
            Ok(module.clone())
        }

        fn resolve_file_item(
            &self,
            package: &TargetAstPackage<Self>,
            item: &Self::FileItem,
            references: &ResolvedReferenceMap<Self>,
        ) -> Result<Self::ResolvedFileItem, AstViolation> {
            fn resolve_symbols(
                symbols: impl IntoIterator<Item = TargetSymbolRef<TestDialect>>,
                references: &ResolvedReferenceMap<TestDialect>,
            ) -> Result<Vec<ResolvedReference<TestDialect>>, AstViolation> {
                symbols
                    .into_iter()
                    .map(|symbol| {
                        references.get(&symbol).cloned().ok_or_else(|| {
                            AstViolation::new(
                                DiagnosticCode::UnresolvedReference,
                                "resolved item cannot retain an unresolved symbol",
                            )
                        })
                    })
                    .collect()
            }

            match item {
                FileItem::Declaration(symbol) => Ok(ResolvedItem {
                    kind: ResolvedItemKind::Declaration(*symbol),
                    references: vec![],
                }),
                FileItem::RuntimeDeclaration { helper, symbols } => Ok(ResolvedItem {
                    kind: ResolvedItemKind::RuntimeDeclaration(helper.clone()),
                    references: resolve_symbols(symbols.clone(), references)?,
                }),
                FileItem::Root(statement) => {
                    let Some((statement, _)) = package.statement(*statement) else {
                        return Err(AstViolation::new(
                            DiagnosticCode::UnresolvedReference,
                            "resolved root refers to a missing statement",
                        ));
                    };
                    let mut symbols = statement.symbols.clone();
                    let mut expressions = vec![statement.expression];
                    let mut visited = BTreeSet::new();
                    while let Some(expression) = expressions.pop() {
                        if !visited.insert(expression) {
                            continue;
                        }
                        let Some((_, node, _)) = package.expression(expression) else {
                            return Err(AstViolation::new(
                                DiagnosticCode::UnresolvedReference,
                                "resolved root reached a missing expression",
                            ));
                        };
                        symbols.extend(self.expression_references(node));
                        expressions.extend(node.child_expressions());
                    }
                    Ok(ResolvedItem {
                        kind: ResolvedItemKind::Root,
                        references: resolve_symbols(symbols, references)?,
                    })
                }
            }
        }

        fn verify_resolved_file_item(&self, _item: &Self::ResolvedFileItem) -> Vec<AstViolation> {
            vec![]
        }

        fn permits_file_cycle(&self, _files: &[TargetFileId]) -> bool {
            self.0 == CatalogueMode::PermittedFileCycle
        }

        fn forward_declarations(
            &self,
            _file: TargetFileId,
            references: &[TargetSymbolRef<Self>],
        ) -> Vec<GeneratedSymbolId> {
            references
                .iter()
                .filter_map(|reference| match reference {
                    TargetSymbolRef::Generated(symbol) => Some(*symbol),
                    _ => None,
                })
                .collect()
        }
    }

    fn catalogue(mode: CatalogueMode) -> SymbolCatalogue<TestDialect> {
        let math_v1 = requirement("1", [PackageFeature::Fast]);
        let mut result = SymbolCatalogue {
            types: vec![KnownTypeSpec {
                symbol: KnownType::Clock,
                name: id("Clock"),
                alias_stem: "Clock".to_owned(),
                qualified_name: Some(QualifiedName::StdClock),
                origin: SymbolOrigin::StandardLibrary(StandardLibrary::Time),
                arity: 0,
                policy: DependencyPolicy::Import(ImportKind::Type),
                dependency: None,
                source: source("known-type"),
            }],
            callables: vec![
                known_callable(
                    KnownCallable::Zero,
                    "zero",
                    callable_pattern(vec![], i64_type()),
                    SymbolOrigin::LanguagePrelude(Prelude::Integer),
                    DependencyPolicy::Implicit,
                    None,
                    None,
                ),
                known_callable(
                    KnownCallable::Negate,
                    "negate",
                    callable_pattern(vec![i64_type()], i64_type()),
                    SymbolOrigin::StandardLibrary(StandardLibrary::Runtime),
                    DependencyPolicy::Qualified,
                    Some(QualifiedName::StdNegate),
                    None,
                ),
                known_callable(
                    KnownCallable::Maximum,
                    "maximum",
                    callable_pattern(vec![i64_type(), i64_type()], i64_type()),
                    SymbolOrigin::ExternalPackage(ExternalPackage::Math),
                    DependencyPolicy::Import(ImportKind::Value),
                    None,
                    Some(math_v1.clone()),
                ),
            ],
            runtime_callables: vec![RuntimeCallableSpec {
                symbol: RuntimeCallable::IsPositive,
                name: id("is_positive"),
                alias_stem: "is_positive".to_owned(),
                qualified_name: Some(QualifiedName::RuntimeIsPositive),
                origin: SymbolOrigin::StandardLibrary(StandardLibrary::Runtime),
                signature: callable_pattern(vec![i64_type()], bool_type()),
                policy: DependencyPolicy::Qualified,
                dependency: None,
                source: source("runtime-callable"),
            }],
            fields: vec![KnownFieldSpec {
                symbol: KnownField::Epoch,
                owner: KnownType::Clock,
                name: id("epoch"),
                origin: SymbolOrigin::StandardLibrary(StandardLibrary::Time),
                ty: TypePattern::Exact(i64_type()),
                policy: DependencyPolicy::Member {
                    owner: QualifiedName::StdClock,
                    member: MemberName::Epoch,
                },
                dependency: None,
                source: source("field"),
            }],
            constructors: vec![KnownConstructorSpec {
                symbol: KnownConstructor::NewClock,
                owner: KnownType::Clock,
                name: id("maximum"),
                alias_stem: "new_clock".to_owned(),
                qualified_name: Some(QualifiedName::MathClock),
                origin: SymbolOrigin::ExternalPackage(ExternalPackage::Math),
                signature: CallablePattern {
                    invocation: Invocation::Constructor,
                    type_parameters: vec![],
                    receiver: None,
                    parameters: vec![],
                    result: TypePattern::Exact(TargetTypeRef::Known(KnownType::Clock)),
                    failure: FailureBehavior::ReturnsResult,
                    effects: BTreeSet::from([TargetEffect::Allocation]),
                },
                visibility: Visibility::Public,
                policy: DependencyPolicy::Import(ImportKind::Value),
                dependency: Some(math_v1.clone()),
                source: source("constructor"),
            }],
            methods: vec![KnownMethodSpec {
                symbol: KnownMethod::Elapsed,
                owner: KnownType::Clock,
                name: id("elapsed"),
                origin: SymbolOrigin::ExternalPackage(ExternalPackage::Math),
                signature: CallablePattern {
                    invocation: Invocation::Instance,
                    type_parameters: vec![TypeParameterSpec {
                        name: id("T"),
                        variance: TypeParameterVariance::Covariant,
                        upper_bound: Some(i64_type()),
                    }],
                    receiver: Some(TypePattern::Exact(TargetTypeRef::Known(KnownType::Clock))),
                    parameters: vec![TypePattern::Parameter(0)],
                    result: TypePattern::Constructed {
                        constructor: ConstructedType::List,
                        arguments: vec![TypePattern::Parameter(0)],
                    },
                    failure: FailureBehavior::Infallible,
                    effects: BTreeSet::from([TargetEffect::Allocation]),
                },
                visibility: Visibility::Public,
                policy: DependencyPolicy::Member {
                    owner: QualifiedName::MathClock,
                    member: MemberName::Elapsed,
                },
                dependency: Some(if mode == CatalogueMode::DependencyConflict {
                    requirement("2", [])
                } else {
                    math_v1
                }),
                source: source("method"),
            }],
            helpers: vec![
                RuntimeHelperSpec {
                    id: Helper::Root,
                    capability: HelperCapability::Arithmetic,
                    order: 20,
                    name: id("runtime_root"),
                    alias_stem: "runtime_root".to_owned(),
                    namespace: Namespace::Value,
                    items: vec![FileItem::RuntimeDeclaration {
                        helper: Helper::Root,
                        symbols: vec![
                            TargetSymbolRef::RuntimeHelper(Helper::Leaf),
                            TargetSymbolRef::KnownType(KnownType::Clock),
                        ],
                    }],
                    placement: Placement::Runtime,
                    visibility: Visibility::Private,
                    source: source("helper-root"),
                },
                RuntimeHelperSpec {
                    id: Helper::Leaf,
                    capability: HelperCapability::Arithmetic,
                    order: 10,
                    name: id("runtime_leaf"),
                    alias_stem: "runtime_leaf".to_owned(),
                    namespace: Namespace::Value,
                    items: vec![FileItem::RuntimeDeclaration {
                        helper: Helper::Leaf,
                        symbols: vec![TargetSymbolRef::KnownCallable(KnownCallable::Maximum)],
                    }],
                    placement: Placement::Runtime,
                    visibility: Visibility::Private,
                    source: source("helper-leaf"),
                },
                RuntimeHelperSpec {
                    id: Helper::Unused,
                    capability: HelperCapability::Arithmetic,
                    order: 30,
                    name: id("runtime_unused"),
                    alias_stem: "runtime_unused".to_owned(),
                    namespace: Namespace::Value,
                    items: vec![FileItem::RuntimeDeclaration {
                        helper: Helper::Unused,
                        symbols: vec![],
                    }],
                    placement: Placement::Runtime,
                    visibility: Visibility::Private,
                    source: source("helper-unused"),
                },
            ],
        };
        match mode {
            CatalogueMode::Duplicate => result.types.push(result.types[0].clone()),
            CatalogueMode::BadSignature => {
                result.callables[0].signature.result = TypePattern::Exact(bool_type());
            }
            CatalogueMode::MissingHelper => {
                result.helpers[0].items = vec![FileItem::RuntimeDeclaration {
                    helper: Helper::Root,
                    symbols: vec![TargetSymbolRef::RuntimeHelper(Helper::Missing)],
                }];
            }
            CatalogueMode::HelperCycle => {
                result.helpers[0].items = vec![FileItem::RuntimeDeclaration {
                    helper: Helper::Root,
                    symbols: vec![TargetSymbolRef::RuntimeHelper(Helper::Cycle)],
                }];
                result.helpers.push(RuntimeHelperSpec {
                    id: Helper::Cycle,
                    capability: HelperCapability::Arithmetic,
                    order: 40,
                    name: id("runtime_cycle"),
                    alias_stem: "runtime_cycle".to_owned(),
                    namespace: Namespace::Value,
                    items: vec![FileItem::RuntimeDeclaration {
                        helper: Helper::Cycle,
                        symbols: vec![TargetSymbolRef::RuntimeHelper(Helper::Root)],
                    }],
                    placement: Placement::Runtime,
                    visibility: Visibility::Private,
                    source: source("helper-cycle"),
                });
            }
            CatalogueMode::HelperIllegalPlacement => {
                result.helpers[0].placement = Placement::MissingRuntime;
            }
            CatalogueMode::PublicHelper => {
                result.helpers[0].visibility = Visibility::Public;
            }
            CatalogueMode::DuplicateHelperOrder => {
                result.helpers[1].order = result.helpers[0].order;
            }
            CatalogueMode::DuplicateHelper => {
                result.helpers.push(result.helpers[0].clone());
            }
            CatalogueMode::Normal
            | CatalogueMode::PermittedFileCycle
            | CatalogueMode::DependencyConflict => {}
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn known_callable(
        symbol: KnownCallable,
        name: &str,
        signature: CallablePattern<TestDialect>,
        origin: SymbolOrigin<TestDialect>,
        policy: DependencyPolicy<TestDialect>,
        qualified_name: Option<QualifiedName>,
        dependency: Option<PackageRequirement<TestDialect>>,
    ) -> KnownCallableSpec<TestDialect> {
        KnownCallableSpec {
            symbol,
            owner: None,
            name: id(name),
            alias_stem: name.to_owned(),
            qualified_name,
            origin,
            signature,
            visibility: Visibility::Public,
            policy,
            dependency,
            source: source(name),
        }
    }

    fn callable_pattern(
        parameters: Vec<TargetTypeRef<TestDialect>>,
        result: TargetTypeRef<TestDialect>,
    ) -> CallablePattern<TestDialect> {
        CallablePattern {
            invocation: Invocation::Static,
            type_parameters: vec![],
            receiver: None,
            parameters: parameters.into_iter().map(TypePattern::Exact).collect(),
            result: TypePattern::Exact(result),
            failure: FailureBehavior::Infallible,
            effects: BTreeSet::new(),
        }
    }

    fn requirement(
        version: &str,
        features: impl IntoIterator<Item = PackageFeature>,
    ) -> PackageRequirement<TestDialect> {
        PackageRequirement {
            package: ExternalPackage::Math,
            version_requirement: version.to_owned(),
            features: features.into_iter().collect(),
        }
    }

    fn package(
        mode: CatalogueMode,
        symbols: Vec<TargetSymbolRef<TestDialect>>,
    ) -> TargetAstPackage<TestDialect> {
        let dialect = TestDialect(mode);
        let mut builder = TargetAstBuilder::new(dialect);
        let generated_type = builder.generated_type(GeneratedType {
            name: "Clock".to_owned(),
            kind: DeclarationKind::Record,
            visibility: Visibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
            source: source("generated-type"),
        });
        let _private_type = builder.generated_type(GeneratedType {
            name: "ClockState".to_owned(),
            kind: DeclarationKind::Record,
            visibility: Visibility::Private,
            origin: GeneratedOrigin::Runtime(AstOrigin::Runtime),
            source: source("private-generated-type"),
        });
        let callable = builder.callable(GeneratedCallable {
            name: "entry".to_owned(),
            signature: signature(vec![], i64_type()),
            visibility: Visibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
            source: source("generated-callable"),
        });
        let _first_value = builder.value(GeneratedValue {
            name: "temporary".to_owned(),
            ty: TargetTypeRef::Known(KnownType::Clock),
            origin: GeneratedOrigin::Synthesized(SynthesisReason::EvaluationTemporary),
            source: source("first-value"),
        });
        let _second_value = builder.value(GeneratedValue {
            name: "temporary".to_owned(),
            ty: TargetTypeRef::Runtime(RuntimeType::Error),
            origin: GeneratedOrigin::Runtime(AstOrigin::Runtime),
            source: source("second-value"),
        });
        let one = builder.expression::<I64Marker>(Expression::I64(1), source("one"));
        let zero = known_nullary_call::<_, ZeroCall>(&mut builder, source("zero-call"));
        let negative = known_unary_call::<_, NegateCall>(&mut builder, one, source("negate-call"));
        let maximum = known_binary_call::<_, MaximumCall>(
            &mut builder,
            negative,
            zero,
            source("maximum-call"),
        );
        let clock = known_nullary_constructor::<_, ClockConstructor>(
            &mut builder,
            source("clock-constructor"),
        );
        let elapsed = known_instance_unary_call::<_, ElapsedCall>(
            &mut builder,
            clock,
            maximum,
            source("elapsed-call"),
        );
        let mut symbols = symbols;
        symbols.push(TargetSymbolRef::Generated(GeneratedSymbolId::Type(
            generated_type,
        )));
        symbols.push(TargetSymbolRef::Generated(GeneratedSymbolId::Callable(
            callable,
        )));
        let statement = builder.statement(
            Statement {
                expression: elapsed.id(),
                symbols,
            },
            source("statement"),
        );
        let file = builder.file(TargetFile::new(
            RelativeOutputPath::new("src/generated.test").unwrap(),
            SourceRole::Implementation,
            Module::Generated,
            Placement::Implementation,
            vec![
                FileItem::Declaration(GeneratedSymbolId::Type(generated_type)),
                FileItem::Declaration(GeneratedSymbolId::Type(_private_type)),
                FileItem::Declaration(GeneratedSymbolId::Callable(callable)),
                FileItem::Root(statement),
            ],
            Template::Source,
            source("file"),
        ));
        let runtime = builder.file(TargetFile::new(
            RelativeOutputPath::new("src/runtime.test").unwrap(),
            SourceRole::Runtime,
            Module::Runtime,
            Placement::Runtime,
            vec![],
            Template::Source,
            source("runtime-file"),
        ));
        let documentation = builder.artifact(TargetArtifact::Documentation {
            path: RelativeOutputPath::new("README.md").unwrap(),
            contents: "# Linked fixture\n".to_owned(),
            source: source("documentation"),
        });
        let asset = builder.artifact(TargetArtifact::Asset {
            path: RelativeOutputPath::new("assets/proof.bin").unwrap(),
            contents: vec![0, 255],
            source: source("asset"),
        });
        builder.group(TargetFileGroup::new(
            FileGroupRole::Implementation,
            vec![TargetFileMember::Source(file)],
            source("implementation-group"),
        ));
        builder.group(TargetFileGroup::new(
            FileGroupRole::Runtime,
            vec![TargetFileMember::Source(runtime)],
            source("runtime-group"),
        ));
        builder.group(TargetFileGroup::new(
            FileGroupRole::Documentation,
            vec![TargetFileMember::Artifact(documentation)],
            source("documentation-group"),
        ));
        builder.group(TargetFileGroup::new(
            FileGroupRole::Assets,
            vec![TargetFileMember::Artifact(asset)],
            source("asset-group"),
        ));
        builder.build()
    }

    fn file_graph_package(
        mode: CatalogueMode,
        left_role: SourceRole,
        right_role: SourceRole,
        right_visibility: Visibility,
        cycle: bool,
    ) -> TargetAstPackage<TestDialect> {
        let mut builder = TargetAstBuilder::new(TestDialect(mode));
        let left = builder.generated_type(GeneratedType {
            name: "Left".to_owned(),
            kind: DeclarationKind::Record,
            visibility: Visibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
            source: source("left"),
        });
        let right = builder.generated_type(GeneratedType {
            name: "Right".to_owned(),
            kind: DeclarationKind::Record,
            visibility: right_visibility,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
            source: source("right"),
        });
        let literal = builder.expression::<I64Marker>(Expression::I64(1), source("literal"));
        let left_statement = builder.statement(
            Statement {
                expression: literal.id(),
                symbols: vec![TargetSymbolRef::Generated(GeneratedSymbolId::Type(right))],
            },
            source("left-reference"),
        );
        let right_statement = builder.statement(
            Statement {
                expression: literal.id(),
                symbols: if cycle {
                    vec![TargetSymbolRef::Generated(GeneratedSymbolId::Type(left))]
                } else {
                    vec![]
                },
            },
            source("right-reference"),
        );
        let left_file = builder.file(TargetFile::new(
            RelativeOutputPath::new("src/a.test").unwrap(),
            left_role,
            Module::Generated,
            Placement::Implementation,
            vec![
                FileItem::Declaration(GeneratedSymbolId::Type(left)),
                FileItem::Root(left_statement),
            ],
            Template::Source,
            source("left-file"),
        ));
        let right_file = builder.file(TargetFile::new(
            RelativeOutputPath::new("src/b.test").unwrap(),
            right_role,
            Module::Generated,
            Placement::Implementation,
            vec![
                FileItem::Declaration(GeneratedSymbolId::Type(right)),
                FileItem::Root(right_statement),
            ],
            Template::Source,
            source("right-file"),
        ));
        let group_role = |role| match role {
            SourceRole::PublicApi => FileGroupRole::PublicApi,
            SourceRole::Implementation => FileGroupRole::Implementation,
            SourceRole::Runtime => FileGroupRole::Runtime,
            SourceRole::NativeTest => FileGroupRole::NativeTests,
            SourceRole::Conformance => FileGroupRole::Conformance,
            SourceRole::NegativeTest => FileGroupRole::NegativeTests,
        };
        let mut groups = BTreeMap::new();
        groups
            .entry(group_role(left_role))
            .or_insert_with(Vec::new)
            .push(TargetFileMember::Source(left_file));
        groups
            .entry(group_role(right_role))
            .or_insert_with(Vec::new)
            .push(TargetFileMember::Source(right_file));
        for (role, members) in groups {
            builder.group(TargetFileGroup::new(role, members, source("graph-group")));
        }
        builder.build()
    }

    fn full_symbols() -> Vec<TargetSymbolRef<TestDialect>> {
        vec![
            TargetSymbolRef::KnownType(KnownType::Clock),
            TargetSymbolRef::RuntimeCallable(RuntimeCallable::IsPositive),
            TargetSymbolRef::KnownField(KnownField::Epoch),
            TargetSymbolRef::RuntimeHelper(Helper::Root),
            TargetSymbolRef::RuntimeHelper(Helper::Root),
        ]
    }

    fn signature(
        parameters: Vec<TargetTypeRef<TestDialect>>,
        return_type: TargetTypeRef<TestDialect>,
    ) -> TargetCallableSignature<TestDialect> {
        TargetCallableSignature {
            invocation: Invocation::Static,
            receiver: None,
            parameters,
            return_type,
        }
    }

    fn i64_type() -> TargetTypeRef<TestDialect> {
        TargetTypeRef::Primitive(Primitive::I64)
    }

    fn bool_type() -> TargetTypeRef<TestDialect> {
        TargetTypeRef::Primitive(Primitive::Bool)
    }

    fn type_errors(
        actual: &TargetTypeRef<TestDialect>,
        expected: &TargetTypeRef<TestDialect>,
    ) -> Vec<AstViolation> {
        if actual == expected {
            vec![]
        } else {
            vec![AstViolation::new(
                DiagnosticCode::TypeMismatch,
                "stored test expression type mismatch",
            )]
        }
    }

    fn id(value: &str) -> Identifier {
        Identifier(value.to_owned())
    }

    fn source(value: &str) -> SourceRef {
        SourceRef::logical(["linker-test", value])
    }

    fn codes(diagnostics: Vec<Diagnostic>) -> BTreeSet<DiagnosticCode> {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect()
    }

    #[test]
    fn catalogue_is_typed_complete_and_rejects_duplicate_or_inconsistent_entries() {
        let normal = TestDialect(CatalogueMode::Normal);
        let catalogue = normal.symbol_catalogue();
        assert_eq!(catalogue.types.len(), 1);
        assert_eq!(catalogue.callables.len(), 3);
        assert_eq!(catalogue.runtime_callables.len(), 1);
        assert_eq!(catalogue.fields.len(), 1);
        assert_eq!(catalogue.constructors.len(), 1);
        assert_eq!(catalogue.methods.len(), 1);
        assert_eq!(catalogue.helpers.len(), 3);
        assert_eq!(catalogue.verify(&normal), Ok(()));

        let duplicate = TestDialect(CatalogueMode::Duplicate);
        assert!(
            codes(duplicate.symbol_catalogue().verify(&duplicate).unwrap_err())
                .contains(&DiagnosticCode::DuplicateDeclaration)
        );
        let bad = TestDialect(CatalogueMode::BadSignature);
        assert!(
            codes(bad.symbol_catalogue().verify(&bad).unwrap_err())
                .contains(&DiagnosticCode::TypeMismatch)
        );
        for mode in [
            CatalogueMode::PublicHelper,
            CatalogueMode::DuplicateHelperOrder,
            CatalogueMode::DuplicateHelper,
        ] {
            let errors = codes(
                TestDialect(mode)
                    .symbol_catalogue()
                    .verify(&TestDialect(mode))
                    .unwrap_err(),
            );
            assert!(
                errors.contains(&DiagnosticCode::InvalidStructure)
                    || errors.contains(&DiagnosticCode::DuplicateDeclaration)
            );
        }
    }

    #[test]
    fn linking_derives_exact_import_dependency_helper_and_alias_sets() {
        let dialect = TestDialect(CatalogueMode::Normal);
        let linked = TargetLinker::new(dialect.clone())
            .link_ast(&package(CatalogueMode::Normal, full_symbols()))
            .unwrap();
        assert_eq!(verify_linked_package(&linked), Ok(()));
        assert_eq!(linked.files().len(), 2);
        assert_eq!(linked.dependencies().len(), 1);
        assert_eq!(
            linked.dependencies()[0].requirement().version_requirement,
            "1"
        );
        assert_eq!(
            linked
                .helpers()
                .iter()
                .map(LinkedRuntimeHelper::id)
                .cloned()
                .collect::<Vec<_>>(),
            vec![Helper::Leaf, Helper::Root]
        );
        assert!(
            !linked
                .helpers()
                .iter()
                .any(|helper| helper.id() == &Helper::Unused)
        );
        assert_eq!(linked.files()[1].helpers(), &[Helper::Leaf, Helper::Root]);
        assert!(linked.files()[1].items().iter().any(|item| matches!(
            item.kind,
            ResolvedItemKind::RuntimeDeclaration(Helper::Leaf)
        )));
        assert!(
            linked
                .files()
                .iter()
                .flat_map(LinkedFile::items)
                .flat_map(|item| &item.references)
                .all(|reference| matches!(
                    reference,
                    ResolvedReference::Local(_)
                        | ResolvedReference::Imported { .. }
                        | ResolvedReference::Qualified(_)
                        | ResolvedReference::Member { .. }
                ))
        );
        assert!(
            linked
                .helpers()
                .iter()
                .all(|helper| helper.file().index() == 1)
        );
        assert!(
            linked
                .helpers()
                .iter()
                .all(|helper| !helper.items.is_empty())
        );

        let file = &linked.files()[0];
        let imports = file.imports();
        assert_eq!(imports.len(), 2);
        let clock = imports
            .iter()
            .find(|import| {
                import
                    .symbols()
                    .contains(&TargetSymbolRef::KnownType(KnownType::Clock))
            })
            .unwrap();
        assert_eq!(clock.original_binding(), &id("Clock"));
        assert_eq!(clock.binding(), &id("Clock_import_2"));
        assert!(file.references().iter().any(|reference| {
            matches!(
                reference.resolved(),
                ResolvedReference::Qualified(QualifiedName::StdNegate)
            )
        }));
        assert!(file.references().iter().any(|reference| {
            matches!(
                reference.resolved(),
                ResolvedReference::Member {
                    owner: QualifiedName::StdClock,
                    member: MemberName::Epoch,
                }
            )
        }));
        assert_eq!(file.forward_declarations().len(), 2);
        let private_names = linked
            .bindings()
            .iter()
            .filter_map(|binding| match binding.symbol() {
                BindableSymbolId::Generated(GeneratedSymbolId::Value(_)) => {
                    Some(binding.identifier().clone())
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            private_names,
            BTreeSet::from([id("temporary"), id("temporary_2")])
        );
    }

    #[test]
    fn minimal_references_do_not_select_optional_catalogue_entries() {
        let linked = TargetLinker::new(TestDialect(CatalogueMode::Normal))
            .link_ast(&package(CatalogueMode::Normal, vec![]))
            .unwrap();
        let symbols = linked.files()[0]
            .imports()
            .iter()
            .flat_map(|import| import.symbols().iter().cloned())
            .collect::<BTreeSet<_>>();
        assert_eq!(linked.files()[0].imports().len(), 1);
        assert_eq!(
            symbols,
            BTreeSet::from([
                TargetSymbolRef::KnownCallable(KnownCallable::Maximum),
                TargetSymbolRef::KnownConstructor(KnownConstructor::NewClock),
            ])
        );
        assert_eq!(linked.dependencies().len(), 1);
        assert!(linked.helpers().is_empty());
        assert!(!symbols.contains(&TargetSymbolRef::KnownType(KnownType::Clock)));
    }

    #[test]
    fn missing_unknown_cycle_and_dependency_conflict_are_targeted() {
        let missing_symbol = TargetLinker::new(TestDialect(CatalogueMode::Normal))
            .link_ast(&package(
                CatalogueMode::Normal,
                vec![TargetSymbolRef::KnownType(KnownType::Uncatalogued)],
            ))
            .unwrap_err();
        assert!(codes(missing_symbol).contains(&DiagnosticCode::UnresolvedReference));

        for (mode, expected) in [
            (
                CatalogueMode::MissingHelper,
                DiagnosticCode::UnresolvedReference,
            ),
            (CatalogueMode::HelperCycle, DiagnosticCode::AliasCycle),
            (
                CatalogueMode::DependencyConflict,
                DiagnosticCode::InterfaceNonconformance,
            ),
            (
                CatalogueMode::HelperIllegalPlacement,
                DiagnosticCode::InvalidStructure,
            ),
        ] {
            let diagnostics = TargetLinker::new(TestDialect(mode))
                .link_ast(&package(mode, full_symbols()))
                .unwrap_err();
            assert!(codes(diagnostics).contains(&expected), "mode {mode:?}");
        }
    }

    #[test]
    fn file_graph_rejects_private_api_runtime_user_and_test_only_edges() {
        for (left_role, right_role, visibility) in [
            (
                SourceRole::PublicApi,
                SourceRole::PublicApi,
                Visibility::Private,
            ),
            (
                SourceRole::PublicApi,
                SourceRole::Implementation,
                Visibility::Public,
            ),
            (
                SourceRole::Runtime,
                SourceRole::Implementation,
                Visibility::Public,
            ),
            (
                SourceRole::Implementation,
                SourceRole::NativeTest,
                Visibility::Public,
            ),
        ] {
            let diagnostics = TargetLinker::new(TestDialect(CatalogueMode::Normal))
                .link_ast(&file_graph_package(
                    CatalogueMode::Normal,
                    left_role,
                    right_role,
                    visibility,
                    false,
                ))
                .unwrap_err();
            assert!(
                diagnostics.iter().any(|diagnostic| diagnostic
                    .message
                    .contains("source-role or public-API visibility")),
                "{left_role:?} -> {right_role:?}: {diagnostics:#?}"
            );
        }
    }

    #[test]
    fn file_graph_cycles_require_an_explicit_dialect_policy() {
        let forbidden = TargetLinker::new(TestDialect(CatalogueMode::Normal))
            .link_ast(&file_graph_package(
                CatalogueMode::Normal,
                SourceRole::Implementation,
                SourceRole::Implementation,
                Visibility::Public,
                true,
            ))
            .unwrap_err();
        assert!(
            forbidden
                .iter()
                .any(|diagnostic| diagnostic.message.contains("forbidden cycle"))
        );

        let permitted = TargetLinker::new(TestDialect(CatalogueMode::PermittedFileCycle))
            .link_ast(&file_graph_package(
                CatalogueMode::PermittedFileCycle,
                SourceRole::Implementation,
                SourceRole::Implementation,
                Visibility::Public,
                true,
            ))
            .unwrap();
        assert_eq!(
            permitted.files()[0].dependencies(),
            &[TargetFileId::from_index(1)]
        );
        assert_eq!(
            permitted.files()[1].dependencies(),
            &[TargetFileId::from_index(0)]
        );
        assert_eq!(verify_linked_package(&permitted), Ok(()));
    }

    #[test]
    fn public_collision_fails_while_private_collision_is_stably_renamed() {
        let mode = CatalogueMode::Normal;
        let mut package = package(mode, vec![]);
        package.types_mut().push(GeneratedType {
            name: "clock".to_owned(),
            kind: DeclarationKind::Record,
            visibility: Visibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
            source: source("colliding-public"),
        });
        let diagnostics = TargetLinker::new(TestDialect(mode))
            .link_ast(&package)
            .unwrap_err();
        assert!(codes(diagnostics).contains(&DiagnosticCode::DuplicateDeclaration));
    }

    #[test]
    fn forged_resolved_sets_are_rejected() {
        let mode = CatalogueMode::Normal;
        let base = TargetLinker::new(TestDialect(mode))
            .link_ast(&package(mode, full_symbols()))
            .unwrap();

        let mut missing_import = base.clone();
        missing_import.files_mut()[0].imports.pop();
        assert!(verify_linked_package(&missing_import).is_err());

        let mut forged_import_membership = base.clone();
        let shared_import = forged_import_membership.files_mut()[0]
            .imports
            .iter_mut()
            .find(|import| import.symbols.len() > 1)
            .expect("fixture has one physical import shared by typed symbols");
        let removed = shared_import
            .symbols
            .iter()
            .next()
            .cloned()
            .expect("shared import has symbols");
        shared_import.symbols.remove(&removed);
        assert!(verify_linked_package(&forged_import_membership).is_err());

        let mut missing_dependency = base.clone();
        missing_dependency.dependencies_mut().clear();
        assert!(verify_linked_package(&missing_dependency).is_err());

        let mut forged_file_edge = base.clone();
        forged_file_edge.files_mut()[0]
            .dependencies
            .push(TargetFileId::from_index(1));
        assert!(verify_linked_package(&forged_file_edge).is_err());

        let mut missing_resolved_item = base.clone();
        missing_resolved_item.files_mut()[0].items.clear();
        assert!(verify_linked_package(&missing_resolved_item).is_err());

        let mut extra_helper = base;
        let spec = extra_helper
            .catalogue
            .helper(&Helper::Unused)
            .unwrap()
            .clone();
        extra_helper.helpers_mut().push(LinkedRuntimeHelper {
            id: spec.id,
            capability: spec.capability,
            order: spec.order,
            file: TargetFileId::from_index(1),
            items: spec.items,
            source: spec.source,
        });
        assert!(verify_linked_package(&extra_helper).is_err());
    }

    #[test]
    fn three_resolutions_have_identical_non_source_dumps() {
        let resolve = || {
            TargetLinker::new(TestDialect(CatalogueMode::Normal))
                .link_ast(&package(CatalogueMode::Normal, full_symbols()))
                .unwrap()
                .canonical_dump()
        };
        let first = resolve();
        assert_eq!(first, resolve());
        assert_eq!(first, resolve());
        assert!(first.contains("ResolvedImport"));
        assert!(!first.contains("import Clock"));
    }

    #[test]
    fn resolved_user_and_runtime_items_share_certified_templates() {
        let render = || {
            let linked = TargetLinker::new(TestDialect(CatalogueMode::Normal))
                .link_ast(&package(CatalogueMode::Normal, full_symbols()))
                .unwrap();
            render_linked_package(&TestRenderer, &linked).unwrap()
        };
        let rendered = render();
        let first = format!("{rendered:#?}");
        assert_eq!(first, format!("{:#?}", render()));
        assert_eq!(first, format!("{:#?}", render()));
        assert_eq!(
            rendered
                .files()
                .iter()
                .map(|file| (file.path(), file.role()))
                .collect::<Vec<_>>(),
            vec![
                ("README.md", crate::OutputFileRole::Documentation),
                ("assets/proof.bin", crate::OutputFileRole::Asset),
                (
                    "src/generated.test",
                    crate::OutputFileRole::ImplementationSource,
                ),
                ("src/runtime.test", crate::OutputFileRole::RuntimeSource),
            ]
        );
        assert_eq!(
            rendered.files()[0].contents(),
            &crate::OutputContents::Text("# Linked fixture\n".to_owned())
        );
        assert_eq!(
            rendered.files()[1].contents(),
            &crate::OutputContents::Bytes(vec![0, 255])
        );
        assert_eq!(
            rendered.dependencies(),
            &[crate::DeclaredDependency {
                ecosystem: "cargo".to_owned(),
                name: "math".to_owned(),
                requirement: "1".to_owned(),
                features: vec!["fast".to_owned()],
            }]
        );
        assert_eq!(
            rendered.helpers(),
            &[
                crate::InjectedHelper {
                    id: "leaf".to_owned(),
                    capability: "arithmetic".to_owned(),
                    files: vec!["src/runtime.test".to_owned()],
                },
                crate::InjectedHelper {
                    id: "root".to_owned(),
                    capability: "arithmetic".to_owned(),
                    files: vec!["src/runtime.test".to_owned()],
                },
            ]
        );
        assert!(first.contains("declaration user;"));
        assert!(first.contains("declaration runtime;"));
        assert!(first.contains("src/generated.test"));
        assert!(first.contains("src/runtime.test"));
    }

    #[test]
    fn closed_metadata_variants_remain_constructible_without_string_switches() {
        let failures = [
            FailureBehavior::Infallible,
            FailureBehavior::ReturnsSentinel,
            FailureBehavior::ReturnsResult,
            FailureBehavior::ThrowsChecked,
            FailureBehavior::ThrowsUnchecked,
            FailureBehavior::Aborts,
        ];
        let effects = [
            TargetEffect::Allocation,
            TargetEffect::Mutation,
            TargetEffect::InputOutput,
            TargetEffect::Nondeterminism,
            TargetEffect::MayBlock,
        ];
        let variances = [
            TypeParameterVariance::Invariant,
            TypeParameterVariance::Covariant,
            TypeParameterVariance::Contravariant,
        ];
        assert_eq!(failures.len(), 6);
        assert_eq!(effects.len(), 5);
        assert_eq!(variances.len(), 3);
        assert_eq!(
            Namespace::Value,
            TestDialect(CatalogueMode::Normal).callable_namespace()
        );
        assert_eq!(ImportKind::Type, ImportKind::Type);
        assert_eq!(Invocation::Constructor, Invocation::Constructor);
        assert_eq!(QualifiedName::MathClock, QualifiedName::MathClock);
    }
}
