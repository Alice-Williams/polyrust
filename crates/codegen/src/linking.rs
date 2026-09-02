use std::collections::{BTreeMap, BTreeSet};

use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef, sort_diagnostics};

use crate::{
    AstViolation, Expr, GeneratedCallableId, GeneratedInterfaceMethodId, GeneratedTypeId,
    GeneratedValueId, TargetAstBuilder, TargetAstPackage, TargetCallableSignature, TargetDialect,
    TargetExprId, TargetExpressionNode, TargetFileId, TargetResolver, TargetStatementNode,
    TargetStmtId, TargetTypeMarker, TargetTypeRef, TypedAstDialect, UnresolvedPackage,
    verify_target_ast,
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
    pub name: D::Identifier,
    pub alias_stem: String,
    pub namespace: D::Namespace,
    pub references: Vec<TargetSymbolRef<D>>,
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
    pub expressions: Vec<TargetExprId>,
    pub statements: Vec<TargetStmtId>,
    pub symbols: Vec<TargetSymbolRef<D>>,
}

impl<D: LinkerDialect> Default for FileItemRoots<D> {
    fn default() -> Self {
        Self {
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
    type Identifier: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type QualifiedName: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type MemberName: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type Namespace: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type NameKey: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;
    type ImportKind: Clone + std::fmt::Debug + Eq + Ord + Send + Sync;

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
    pub fn symbol(&self) -> &BindableSymbolId<D> {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedImport<D: LinkerDialect> {
    id: ResolvedImportId,
    symbol: TargetSymbolRef<D>,
    original_binding: D::Identifier,
    binding: D::Identifier,
    kind: D::ImportKind,
    origin: SymbolOrigin<D>,
}

impl<D: LinkerDialect> ResolvedImport<D> {
    pub const fn id(&self) -> ResolvedImportId {
        self.id
    }

    pub fn symbol(&self) -> &TargetSymbolRef<D> {
        &self.symbol
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
pub struct LinkedReference<D: LinkerDialect> {
    source: SourceRef,
    symbol: TargetSymbolRef<D>,
    resolved: ResolvedReference<D>,
}

impl<D: LinkerDialect> LinkedReference<D> {
    pub fn source(&self) -> &SourceRef {
        &self.source
    }

    pub fn symbol(&self) -> &TargetSymbolRef<D> {
        &self.symbol
    }

    pub fn resolved(&self) -> &ResolvedReference<D> {
        &self.resolved
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedFile<D: LinkerDialect> {
    file: TargetFileId,
    references: Vec<LinkedReference<D>>,
    imports: Vec<ResolvedImport<D>>,
    forward_declarations: Vec<GeneratedSymbolId>,
}

impl<D: LinkerDialect> LinkedFile<D> {
    pub const fn file(&self) -> TargetFileId {
        self.file
    }

    pub fn references(&self) -> &[LinkedReference<D>] {
        &self.references
    }

    pub fn imports(&self) -> &[ResolvedImport<D>] {
        &self.imports
    }

    pub fn forward_declarations(&self) -> &[GeneratedSymbolId] {
        &self.forward_declarations
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkedTargetPackage<D: LinkerDialect> {
    dialect: D,
    unresolved: TargetAstPackage<D>,
    bindings: Vec<ResolvedBinding<D>>,
    files: Vec<LinkedFile<D>>,
    dependencies: Vec<ResolvedPackageDependency<D>>,
    helpers: Vec<D::HelperId>,
    catalogue: SymbolCatalogue<D>,
}

impl<D: LinkerDialect> LinkedTargetPackage<D> {
    pub fn unresolved(&self) -> &TargetAstPackage<D> {
        &self.unresolved
    }

    pub fn bindings(&self) -> &[ResolvedBinding<D>] {
        &self.bindings
    }

    pub fn files(&self) -> &[LinkedFile<D>] {
        &self.files
    }

    pub fn dependencies(&self) -> &[ResolvedPackageDependency<D>] {
        &self.dependencies
    }

    pub fn helpers(&self) -> &[D::HelperId] {
        &self.helpers
    }

    pub fn canonical_dump(&self) -> String {
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
    fn helpers_mut(&mut self) -> &mut Vec<D::HelperId> {
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
        let selected_helpers = expand_file_helpers(&catalogue, &mut raw_files, &mut diagnostics);
        let bindings = allocate_bindings(
            &self.dialect,
            unresolved,
            &catalogue,
            &selected_helpers,
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
            files.push(LinkedFile {
                file: raw_file.file,
                references,
                imports,
                forward_declarations,
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
            helpers: selected_helpers.into_iter().collect(),
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
}

fn collect_references<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<RawFile<D>> {
    let mut files = Vec::new();
    for (file_index, file) in package.files().enumerate() {
        let file_id = TargetFileId::from_index(file_index);
        let mut references = Vec::new();
        let mut visited_expressions = BTreeSet::new();
        let mut visited_statements = BTreeSet::new();
        for item in &file.items {
            let roots = dialect.file_item_roots(item);
            references.extend(roots.symbols.into_iter().map(|symbol| LocatedSymbol {
                symbol,
                source: file.source.clone(),
            }));
            for statement in roots.statements {
                if !visited_statements.insert(statement) {
                    continue;
                }
                let Some((node, source)) = package.statement(statement) else {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::UnresolvedReference,
                        "linker file item refers to a missing statement",
                        file.source.clone(),
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
        files.push(RawFile {
            file: file_id,
            references,
        });
    }
    files
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
    catalogue: &SymbolCatalogue<D>,
    files: &mut [RawFile<D>],
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeSet<D::HelperId> {
    let mut selected = BTreeSet::new();
    for file in files {
        let roots = file
            .references
            .iter()
            .filter_map(|reference| match &reference.symbol {
                TargetSymbolRef::RuntimeHelper(helper) => {
                    Some((helper.clone(), reference.source.clone()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut states = BTreeMap::new();
        let mut expanded = Vec::new();
        for (helper, source) in roots {
            expand_helper(
                catalogue,
                &helper,
                &source,
                &mut states,
                &mut selected,
                &mut expanded,
                diagnostics,
            );
        }
        file.references.extend(expanded);
    }
    selected
}

#[allow(clippy::too_many_arguments)]
fn expand_helper<D: LinkerDialect>(
    catalogue: &SymbolCatalogue<D>,
    helper: &D::HelperId,
    requested_at: &SourceRef,
    states: &mut BTreeMap<D::HelperId, u8>,
    selected: &mut BTreeSet<D::HelperId>,
    expanded: &mut Vec<LocatedSymbol<D>>,
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
    states.insert(helper.clone(), 1);
    for reference in &spec.references {
        if let TargetSymbolRef::RuntimeHelper(child) = reference {
            expand_helper(
                catalogue,
                child,
                &spec.source,
                states,
                selected,
                expanded,
                diagnostics,
            );
        }
        expanded.push(LocatedSymbol {
            symbol: reference.clone(),
            source: spec.source.clone(),
        });
    }
    states.insert(helper.clone(), 2);
    selected.insert(helper.clone());
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
                symbol: located.symbol.clone(),
                original_binding: plan.name,
                binding: binding.clone(),
                kind,
                origin: plan.origin,
            });
            import_lookup.insert(located.symbol.clone(), id);
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
    for file in &package.files {
        if !files.insert(file.file) {
            diagnostics.push(link_error(
                DiagnosticCode::DuplicateDeclaration,
                "resolved file appears more than once",
                "files",
            ));
        }
        let mut file_imports = BTreeMap::new();
        for import in &file.imports {
            if !declared_imports.insert(import.id) {
                diagnostics.push(link_error(
                    DiagnosticCode::DuplicateDeclaration,
                    "resolved import ID appears more than once",
                    "imports",
                ));
            }
            file_imports.insert(import.id, import);
        }
        for reference in &file.references {
            if let ResolvedReference::Imported { import, .. } = reference.resolved {
                referenced_imports.insert(import);
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
    let actual_helpers = package.helpers.iter().cloned().collect::<BTreeSet<_>>();
    if actual_helpers.len() != package.helpers.len() || helper_roots != actual_helpers {
        diagnostics.push(link_error(
            DiagnosticCode::InterfaceNonconformance,
            "resolved helper set is missing roots or contains duplicates",
            "helpers",
        ));
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
        (ResolvedReference::Imported { binding, import }, DependencyPolicy::Import(_)) => imports
            .get(import)
            .is_some_and(|record| record.symbol == reference.symbol && &record.binding == binding),
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
        GeneratedCallable, GeneratedOrigin, GeneratedType, GeneratedValue, SynthesisReason,
        TargetExpressionNode, TargetFile, TargetFileGroup, TargetFileItemNode, TargetFileRole,
        TargetStatementNode,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum CatalogueMode {
        Normal,
        Duplicate,
        BadSignature,
        MissingHelper,
        HelperCycle,
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
        Declaration,
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
                Self::Root(_) | Self::Declaration => vec![],
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
        type Identifier = Identifier;
        type QualifiedName = QualifiedName;
        type MemberName = MemberName;
        type Namespace = Namespace;
        type NameKey = String;
        type ImportKind = ImportKind;

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
                FileItem::Declaration => FileItemRoots::default(),
            }
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
                name: id("new_clock"),
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
                    name: id("runtime_root"),
                    alias_stem: "runtime_root".to_owned(),
                    namespace: Namespace::Value,
                    references: vec![
                        TargetSymbolRef::RuntimeHelper(Helper::Leaf),
                        TargetSymbolRef::KnownType(KnownType::Clock),
                    ],
                    source: source("helper-root"),
                },
                RuntimeHelperSpec {
                    id: Helper::Leaf,
                    name: id("runtime_leaf"),
                    alias_stem: "runtime_leaf".to_owned(),
                    namespace: Namespace::Value,
                    references: vec![TargetSymbolRef::KnownCallable(KnownCallable::Maximum)],
                    source: source("helper-leaf"),
                },
                RuntimeHelperSpec {
                    id: Helper::Unused,
                    name: id("runtime_unused"),
                    alias_stem: "runtime_unused".to_owned(),
                    namespace: Namespace::Value,
                    references: vec![],
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
                result.helpers[0].references =
                    vec![TargetSymbolRef::RuntimeHelper(Helper::Missing)];
            }
            CatalogueMode::HelperCycle => {
                result.helpers[0].references = vec![TargetSymbolRef::RuntimeHelper(Helper::Cycle)];
                result.helpers.push(RuntimeHelperSpec {
                    id: Helper::Cycle,
                    name: id("runtime_cycle"),
                    alias_stem: "runtime_cycle".to_owned(),
                    namespace: Namespace::Value,
                    references: vec![TargetSymbolRef::RuntimeHelper(Helper::Root)],
                    source: source("helper-cycle"),
                });
            }
            CatalogueMode::Normal | CatalogueMode::DependencyConflict => {}
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
        let file = builder.file(TargetFile {
            path: "src/generated.test".to_owned(),
            role: TargetFileRole::Source,
            items: vec![FileItem::Declaration, FileItem::Root(statement)],
            template: Template::Source,
            source: source("file"),
        });
        builder.group(TargetFileGroup {
            files: vec![file],
            source: source("group"),
        });
        builder.build()
    }

    fn full_symbols() -> Vec<TargetSymbolRef<TestDialect>> {
        vec![
            TargetSymbolRef::KnownType(KnownType::Clock),
            TargetSymbolRef::RuntimeCallable(RuntimeCallable::IsPositive),
            TargetSymbolRef::KnownField(KnownField::Epoch),
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
    }

    #[test]
    fn linking_derives_exact_import_dependency_helper_and_alias_sets() {
        let dialect = TestDialect(CatalogueMode::Normal);
        let linked = TargetLinker::new(dialect.clone())
            .link_ast(&package(CatalogueMode::Normal, full_symbols()))
            .unwrap();
        assert_eq!(verify_linked_package(&linked), Ok(()));
        assert_eq!(linked.files().len(), 1);
        assert_eq!(linked.dependencies().len(), 1);
        assert_eq!(
            linked.dependencies()[0].requirement().version_requirement,
            "1"
        );
        assert_eq!(linked.helpers(), &[Helper::Root, Helper::Leaf]);
        assert!(!linked.helpers().contains(&Helper::Unused));

        let file = &linked.files()[0];
        let imports = file.imports();
        assert_eq!(imports.len(), 3);
        let clock = imports
            .iter()
            .find(|import| import.symbol() == &TargetSymbolRef::KnownType(KnownType::Clock))
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
            .map(|import| import.symbol().clone())
            .collect::<BTreeSet<_>>();
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
        ] {
            let diagnostics = TargetLinker::new(TestDialect(mode))
                .link_ast(&package(mode, full_symbols()))
                .unwrap_err();
            assert!(codes(diagnostics).contains(&expected), "mode {mode:?}");
        }
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

        let mut missing_dependency = base.clone();
        missing_dependency.dependencies_mut().clear();
        assert!(verify_linked_package(&missing_dependency).is_err());

        let mut extra_helper = base;
        extra_helper.helpers_mut().push(Helper::Unused);
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
