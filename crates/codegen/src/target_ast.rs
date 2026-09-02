use std::{fmt::Debug, marker::PhantomData};

use portable_core_ir::{CoreDeclaration, CoreExprId};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef, sort_diagnostics};

use crate::typed_pipeline::TargetDialect;

macro_rules! ast_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            pub const fn index(self) -> usize {
                self.0 as usize
            }

            fn from_index(index: usize) -> Self {
                Self(u32::try_from(index).expect("target AST arena exceeds u32"))
            }
        }
    };
}

ast_id!(TargetExprId);
ast_id!(TargetStmtId);
ast_id!(GeneratedTypeId);
ast_id!(GeneratedCallableId);
ast_id!(GeneratedInterfaceMethodId);
ast_id!(GeneratedValueId);
ast_id!(TargetTypeParameterId);
ast_id!(TargetFileId);
ast_id!(TargetFileGroupId);

/// Target-language grammar and catalogue contract used by the typed AST.
///
/// This extends the phase-level [`TargetDialect`] and fixes its unresolved
/// associated type to this dialect's [`TargetAstPackage`].
pub trait TypedAstDialect:
    TargetDialect<Unresolved = TargetAstPackage<Self>> + Clone + Debug + Eq
{
    type PrimitiveType: Clone + Debug + Eq + Ord + Send + Sync;
    type KnownType: Clone + Debug + Eq + Ord + Send + Sync;
    type RuntimeType: Clone + Debug + Eq + Ord + Send + Sync;
    type ConstructedType: Clone + Debug + Eq + Ord + Send + Sync;
    type KnownCallable: Clone + Debug + Eq + Ord + Send + Sync;
    type RuntimeCallable: Clone + Debug + Eq + Ord + Send + Sync;
    type InvocationKind: Clone + Debug + Eq + Ord + Send + Sync;
    type Visibility: Clone + Debug + Eq + Ord + Send + Sync;
    type DeclarationKind: Clone + Debug + Eq + Ord + Send + Sync;
    type SymbolOrigin: Clone + Debug + Eq + Ord + Send + Sync;
    type TemplateId: Clone + Debug + Eq + Ord + Send + Sync;
    type Expression: TargetExpressionNode<Self>;
    type Statement: TargetStatementNode<Self>;
    type FileItem: TargetFileItemNode<Self>;

    fn known_callable_signature(
        &self,
        callable: &Self::KnownCallable,
    ) -> TargetCallableSignature<Self>;

    fn runtime_callable_signature(
        &self,
        callable: &Self::RuntimeCallable,
    ) -> TargetCallableSignature<Self>;

    fn verify_signature(&self, signature: &TargetCallableSignature<Self>) -> Vec<AstViolation>;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TargetTypeRef<D: TypedAstDialect> {
    Primitive(D::PrimitiveType),
    Known(D::KnownType),
    Generated(GeneratedTypeId),
    Runtime(D::RuntimeType),
    TypeParameter(TargetTypeParameterId),
    Constructed(D::ConstructedType),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TargetCallableRef<D: TypedAstDialect> {
    Known(D::KnownCallable),
    Generated(GeneratedCallableId),
    Interface(GeneratedInterfaceMethodId),
    Runtime(D::RuntimeCallable),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GeneratedOrigin<D: TypedAstDialect> {
    CoreDeclaration(CoreDeclaration),
    CoreExpression(CoreExprId),
    Runtime(D::SymbolOrigin),
    Synthesized(SynthesisReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SynthesisReason {
    EvaluationTemporary,
    OwnershipAdapter,
    InterfaceAdapter,
    TestHarness,
    PackageEntryPoint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetCallableSignature<D: TypedAstDialect> {
    pub invocation: D::InvocationKind,
    pub receiver: Option<TargetTypeRef<D>>,
    pub parameters: Vec<TargetTypeRef<D>>,
    pub return_type: TargetTypeRef<D>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedType<D: TypedAstDialect> {
    pub name: String,
    pub kind: D::DeclarationKind,
    pub visibility: D::Visibility,
    pub origin: GeneratedOrigin<D>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedCallable<D: TypedAstDialect> {
    pub name: String,
    pub signature: TargetCallableSignature<D>,
    pub visibility: D::Visibility,
    pub origin: GeneratedOrigin<D>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedInterfaceMethod<D: TypedAstDialect> {
    pub owner: GeneratedTypeId,
    pub name: String,
    pub signature: TargetCallableSignature<D>,
    pub origin: GeneratedOrigin<D>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedValue<D: TypedAstDialect> {
    pub name: String,
    pub ty: TargetTypeRef<D>,
    pub origin: GeneratedOrigin<D>,
    pub source: SourceRef,
}

pub trait TargetTypeMarker<D: TypedAstDialect>: Send + Sync + 'static {
    fn target_type() -> TargetTypeRef<D>;
}

/// A handle whose marker makes a known target type part of its Rust type.
///
/// ```compile_fail
/// use portable_codegen::{Expr, TypedAstDialect, TargetTypeMarker};
///
/// fn require_same<D, T>(_: Expr<D, T>, _: Expr<D, T>)
/// where
///     D: TypedAstDialect,
///     T: TargetTypeMarker<D>,
/// {}
///
/// fn cannot_mix<D, Left, Right>(left: Expr<D, Left>, right: Expr<D, Right>)
/// where
///     D: TypedAstDialect,
///     Left: TargetTypeMarker<D>,
///     Right: TargetTypeMarker<D>,
/// {
///     require_same(left, right);
/// }
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Expr<D: TypedAstDialect, T: TargetTypeMarker<D>> {
    id: TargetExprId,
    marker: PhantomData<fn() -> (D, T)>,
}

impl<D: TypedAstDialect, T: TargetTypeMarker<D>> Copy for Expr<D, T> {}

impl<D: TypedAstDialect, T: TargetTypeMarker<D>> Clone for Expr<D, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D: TypedAstDialect, T: TargetTypeMarker<D>> Expr<D, T> {
    pub const fn id(self) -> TargetExprId {
        self.id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynamicExpr<D: TypedAstDialect> {
    id: TargetExprId,
    marker: PhantomData<fn() -> D>,
}

impl<D: TypedAstDialect> DynamicExpr<D> {
    pub const fn id(self) -> TargetExprId {
        self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstViolation {
    pub code: DiagnosticCode,
    pub message: String,
}

impl AstViolation {
    pub fn new(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub trait TargetExpressionNode<D: TypedAstDialect>:
    Clone + Debug + Eq + Send + Sync + 'static
{
    fn child_expressions(&self) -> Vec<TargetExprId>;
    fn verify(
        &self,
        stored_type: &TargetTypeRef<D>,
        context: &TargetAstContext<'_, D>,
    ) -> Vec<AstViolation>;
}

pub trait TargetStatementNode<D: TypedAstDialect>:
    Clone + Debug + Eq + Send + Sync + 'static
{
    fn child_expressions(&self) -> Vec<TargetExprId>;
    fn verify(&self, context: &TargetAstContext<'_, D>) -> Vec<AstViolation>;
}

pub trait TargetFileItemNode<D: TypedAstDialect>:
    Clone + Debug + Eq + Send + Sync + 'static
{
    fn verify(&self, context: &TargetAstContext<'_, D>) -> Vec<AstViolation>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TargetExpression<D: TypedAstDialect> {
    ty: TargetTypeRef<D>,
    node: D::Expression,
    source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TargetStatement<D: TypedAstDialect> {
    node: D::Statement,
    source: SourceRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TargetFileRole {
    Source,
    Runtime,
    Test,
    Conformance,
    NegativeTest,
    Metadata,
    Documentation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetFile<D: TypedAstDialect> {
    pub path: String,
    pub role: TargetFileRole,
    pub items: Vec<D::FileItem>,
    pub template: D::TemplateId,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetFileGroup {
    pub files: Vec<TargetFileId>,
    pub source: SourceRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetAstPackage<D: TypedAstDialect> {
    dialect: D,
    types: Vec<GeneratedType<D>>,
    callables: Vec<GeneratedCallable<D>>,
    interface_methods: Vec<GeneratedInterfaceMethod<D>>,
    values: Vec<GeneratedValue<D>>,
    type_parameters: Vec<SourceRef>,
    expressions: Vec<TargetExpression<D>>,
    statements: Vec<TargetStatement<D>>,
    files: Vec<TargetFile<D>>,
    groups: Vec<TargetFileGroup>,
}

impl<D: TypedAstDialect> TargetAstPackage<D> {
    pub fn dialect(&self) -> &D {
        &self.dialect
    }

    pub fn generated_type(&self, id: GeneratedTypeId) -> Option<&GeneratedType<D>> {
        self.types.get(id.index())
    }

    pub fn callable(&self, id: GeneratedCallableId) -> Option<&GeneratedCallable<D>> {
        self.callables.get(id.index())
    }

    pub fn interface_method(
        &self,
        id: GeneratedInterfaceMethodId,
    ) -> Option<&GeneratedInterfaceMethod<D>> {
        self.interface_methods.get(id.index())
    }

    pub fn value(&self, id: GeneratedValueId) -> Option<&GeneratedValue<D>> {
        self.values.get(id.index())
    }

    pub fn type_parameter(&self, id: TargetTypeParameterId) -> Option<&SourceRef> {
        self.type_parameters.get(id.index())
    }

    pub fn expression_type(&self, id: TargetExprId) -> Option<&TargetTypeRef<D>> {
        self.expressions.get(id.index()).map(|value| &value.ty)
    }

    pub fn file(&self, id: TargetFileId) -> Option<&TargetFile<D>> {
        self.files.get(id.index())
    }

    pub fn group(&self, id: TargetFileGroupId) -> Option<&TargetFileGroup> {
        self.groups.get(id.index())
    }

    pub fn canonical_dump(&self) -> String {
        format!("{self:#?}")
    }

    #[cfg(test)]
    fn expressions_mut(&mut self) -> &mut [TargetExpression<D>] {
        &mut self.expressions
    }

    #[cfg(test)]
    fn groups_mut(&mut self) -> &mut [TargetFileGroup] {
        &mut self.groups
    }
}

pub struct TargetAstContext<'a, D: TypedAstDialect> {
    package: &'a TargetAstPackage<D>,
}

impl<'a, D: TypedAstDialect> TargetAstContext<'a, D> {
    pub fn dialect(&self) -> &'a D {
        self.package.dialect()
    }

    pub fn expression_type(&self, id: TargetExprId) -> Option<&'a TargetTypeRef<D>> {
        self.package.expression_type(id)
    }

    pub fn generated_type(&self, id: GeneratedTypeId) -> Option<&'a GeneratedType<D>> {
        self.package.generated_type(id)
    }

    pub fn callable(&self, id: GeneratedCallableId) -> Option<&'a GeneratedCallable<D>> {
        self.package.callable(id)
    }

    pub fn interface_method(
        &self,
        id: GeneratedInterfaceMethodId,
    ) -> Option<&'a GeneratedInterfaceMethod<D>> {
        self.package.interface_method(id)
    }

    pub fn value(&self, id: GeneratedValueId) -> Option<&'a GeneratedValue<D>> {
        self.package.value(id)
    }

    pub fn type_parameter(&self, id: TargetTypeParameterId) -> Option<&'a SourceRef> {
        self.package.type_parameter(id)
    }

    pub fn contains_statement(&self, id: TargetStmtId) -> bool {
        self.package.statements.get(id.index()).is_some()
    }

    pub fn callable_signature(
        &self,
        callable: &TargetCallableRef<D>,
    ) -> Option<TargetCallableSignature<D>> {
        match callable {
            TargetCallableRef::Known(callable) => {
                Some(self.dialect().known_callable_signature(callable))
            }
            TargetCallableRef::Generated(id) => {
                self.callable(*id).map(|value| value.signature.clone())
            }
            TargetCallableRef::Interface(id) => self
                .interface_method(*id)
                .map(|value| value.signature.clone()),
            TargetCallableRef::Runtime(callable) => {
                Some(self.dialect().runtime_callable_signature(callable))
            }
        }
    }
}

/// Typed AST construction keeps expression and statement grammar categories
/// separate. It accepts dialect nodes, never source text or a document.
///
/// ```compile_fail
/// use portable_codegen::{TargetAstBuilder, TypedAstDialect};
///
/// fn wrong_category<D: TypedAstDialect>(
///     builder: &mut TargetAstBuilder<D>,
///     statement: D::Statement,
/// ) {
///     // Expression construction cannot accept a statement node.
///     builder.dynamic_expression(statement, unreachable!(), unreachable!());
/// }
/// ```
#[derive(Clone, Debug)]
pub struct TargetAstBuilder<D: TypedAstDialect> {
    package: TargetAstPackage<D>,
}

impl<D: TypedAstDialect> TargetAstBuilder<D> {
    pub fn new(dialect: D) -> Self {
        Self {
            package: TargetAstPackage {
                dialect,
                types: vec![],
                callables: vec![],
                interface_methods: vec![],
                values: vec![],
                type_parameters: vec![],
                expressions: vec![],
                statements: vec![],
                files: vec![],
                groups: vec![],
            },
        }
    }

    pub fn generated_type(&mut self, value: GeneratedType<D>) -> GeneratedTypeId {
        let id = GeneratedTypeId::from_index(self.package.types.len());
        self.package.types.push(value);
        id
    }

    pub fn callable(&mut self, value: GeneratedCallable<D>) -> GeneratedCallableId {
        let id = GeneratedCallableId::from_index(self.package.callables.len());
        self.package.callables.push(value);
        id
    }

    pub fn interface_method(
        &mut self,
        value: GeneratedInterfaceMethod<D>,
    ) -> GeneratedInterfaceMethodId {
        let id = GeneratedInterfaceMethodId::from_index(self.package.interface_methods.len());
        self.package.interface_methods.push(value);
        id
    }

    pub fn value(&mut self, value: GeneratedValue<D>) -> GeneratedValueId {
        let id = GeneratedValueId::from_index(self.package.values.len());
        self.package.values.push(value);
        id
    }

    pub fn type_parameter(&mut self, source: SourceRef) -> TargetTypeParameterId {
        let id = TargetTypeParameterId::from_index(self.package.type_parameters.len());
        self.package.type_parameters.push(source);
        id
    }

    pub fn expression<T: TargetTypeMarker<D>>(
        &mut self,
        node: D::Expression,
        source: SourceRef,
    ) -> Expr<D, T> {
        let id = self.dynamic_expression(node, T::target_type(), source).id;
        Expr {
            id,
            marker: PhantomData,
        }
    }

    pub fn dynamic_expression(
        &mut self,
        node: D::Expression,
        ty: TargetTypeRef<D>,
        source: SourceRef,
    ) -> DynamicExpr<D> {
        let id = TargetExprId::from_index(self.package.expressions.len());
        self.package
            .expressions
            .push(TargetExpression { ty, node, source });
        DynamicExpr {
            id,
            marker: PhantomData,
        }
    }

    pub fn statement(&mut self, node: D::Statement, source: SourceRef) -> TargetStmtId {
        let id = TargetStmtId::from_index(self.package.statements.len());
        self.package
            .statements
            .push(TargetStatement { node, source });
        id
    }

    pub fn file(&mut self, file: TargetFile<D>) -> TargetFileId {
        let id = TargetFileId::from_index(self.package.files.len());
        self.package.files.push(file);
        id
    }

    pub fn group(&mut self, group: TargetFileGroup) -> TargetFileGroupId {
        let id = TargetFileGroupId::from_index(self.package.groups.len());
        self.package.groups.push(group);
        id
    }

    pub fn build(self) -> TargetAstPackage<D> {
        self.package
    }
}

pub fn verify_target_ast<D: TypedAstDialect>(
    package: &TargetAstPackage<D>,
) -> Result<(), Vec<Diagnostic>> {
    let context = TargetAstContext { package };
    let mut diagnostics = Vec::new();
    for value in &package.types {
        check_source(&mut diagnostics, &value.source, "generated type");
        check_name(
            &mut diagnostics,
            &value.name,
            &value.source,
            "generated type",
        );
    }
    for value in &package.callables {
        check_source(&mut diagnostics, &value.source, "generated callable");
        check_name(
            &mut diagnostics,
            &value.name,
            &value.source,
            "generated callable",
        );
        check_signature(&mut diagnostics, &value.signature, &context, &value.source);
    }
    for value in &package.interface_methods {
        check_source(
            &mut diagnostics,
            &value.source,
            "generated interface method",
        );
        check_name(
            &mut diagnostics,
            &value.name,
            &value.source,
            "generated interface method",
        );
        if package.generated_type(value.owner).is_none() {
            missing(&mut diagnostics, "interface owner", &value.source);
        }
        check_signature(&mut diagnostics, &value.signature, &context, &value.source);
    }
    for value in &package.values {
        check_source(&mut diagnostics, &value.source, "generated value");
        check_name(
            &mut diagnostics,
            &value.name,
            &value.source,
            "generated value",
        );
        check_type(&mut diagnostics, &value.ty, &context, &value.source);
    }
    for source in &package.type_parameters {
        check_source(&mut diagnostics, source, "target type parameter");
    }
    for (index, expression) in package.expressions.iter().enumerate() {
        check_source(&mut diagnostics, &expression.source, "target expression");
        check_type(
            &mut diagnostics,
            &expression.ty,
            &context,
            &expression.source,
        );
        for child in expression.node.child_expressions() {
            if package.expression_type(child).is_none() {
                missing(&mut diagnostics, "expression child", &expression.source);
            } else if child.index() >= index {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidControlFlow,
                    "target expression child is not defined before use",
                    expression.source.clone(),
                ));
            }
        }
        add_violations(
            &mut diagnostics,
            expression.node.verify(&expression.ty, &context),
            &expression.source,
        );
    }
    for statement in &package.statements {
        check_source(&mut diagnostics, &statement.source, "target statement");
        for child in statement.node.child_expressions() {
            if package.expression_type(child).is_none() {
                missing(&mut diagnostics, "statement expression", &statement.source);
            }
        }
        add_violations(
            &mut diagnostics,
            statement.node.verify(&context),
            &statement.source,
        );
    }
    for file in &package.files {
        check_source(&mut diagnostics, &file.source, "target file");
        if !safe_target_path(&file.path) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnsafeOutputPath,
                format!("target AST file path {:?} is unsafe", file.path),
                file.source.clone(),
            ));
        }
        for item in &file.items {
            add_violations(&mut diagnostics, item.verify(&context), &file.source);
        }
    }
    let mut grouped = std::collections::BTreeSet::new();
    for group in &package.groups {
        check_source(&mut diagnostics, &group.source, "target file group");
        for file in &group.files {
            if package.file(*file).is_none() {
                missing(&mut diagnostics, "group file", &group.source);
            }
            if !grouped.insert(*file) {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::DuplicateDeclaration,
                    "target file appears in more than one group",
                    group.source.clone(),
                ));
            }
        }
    }
    if grouped.len() != package.files.len() {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            "not every target file belongs to exactly one group",
            root_source(package),
        ));
    }
    sort_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn check_signature<D: TypedAstDialect>(
    diagnostics: &mut Vec<Diagnostic>,
    signature: &TargetCallableSignature<D>,
    context: &TargetAstContext<'_, D>,
    source: &SourceRef,
) {
    if let Some(receiver) = &signature.receiver {
        check_type(diagnostics, receiver, context, source);
    }
    for parameter in &signature.parameters {
        check_type(diagnostics, parameter, context, source);
    }
    check_type(diagnostics, &signature.return_type, context, source);
    add_violations(
        diagnostics,
        context.dialect().verify_signature(signature),
        source,
    );
}

fn check_type<D: TypedAstDialect>(
    diagnostics: &mut Vec<Diagnostic>,
    ty: &TargetTypeRef<D>,
    context: &TargetAstContext<'_, D>,
    source: &SourceRef,
) {
    match ty {
        TargetTypeRef::Generated(id) if context.generated_type(*id).is_none() => {
            missing(diagnostics, "generated type", source)
        }
        TargetTypeRef::TypeParameter(id) if context.package.type_parameter(*id).is_none() => {
            missing(diagnostics, "target type parameter", source)
        }
        TargetTypeRef::Primitive(_)
        | TargetTypeRef::Known(_)
        | TargetTypeRef::Runtime(_)
        | TargetTypeRef::Constructed(_)
        | TargetTypeRef::Generated(_)
        | TargetTypeRef::TypeParameter(_) => {}
    }
}

fn add_violations(
    diagnostics: &mut Vec<Diagnostic>,
    violations: Vec<AstViolation>,
    source: &SourceRef,
) {
    diagnostics.extend(
        violations
            .into_iter()
            .map(|violation| Diagnostic::error(violation.code, violation.message, source.clone())),
    );
}

fn check_name(diagnostics: &mut Vec<Diagnostic>, name: &str, source: &SourceRef, category: &str) {
    if name.is_empty() || name.chars().any(char::is_control) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidIdentifier,
            format!("{category} has an invalid unresolved name"),
            source.clone(),
        ));
    }
}

fn check_source(diagnostics: &mut Vec<Diagnostic>, source: &SourceRef, category: &str) {
    let valid = match source {
        SourceRef::File(span) => {
            !span.file.is_empty()
                && !std::path::Path::new(&span.file).is_absolute()
                && span.start <= span.end
        }
        SourceRef::Logical(path) => {
            !path.segments.is_empty() && path.segments.iter().all(|part| !part.is_empty())
        }
    };
    if !valid {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::InvalidStructure,
            format!("{category} has invalid or missing provenance"),
            source.clone(),
        ));
    }
}

fn safe_target_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !std::path::Path::new(path).is_absolute()
        && path
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
}

fn missing(diagnostics: &mut Vec<Diagnostic>, category: &str, source: &SourceRef) {
    diagnostics.push(Diagnostic::error(
        DiagnosticCode::UnresolvedReference,
        format!("{category} reference is out of bounds"),
        source.clone(),
    ));
}

fn root_source<D: TypedAstDialect>(package: &TargetAstPackage<D>) -> SourceRef {
    package
        .types
        .first()
        .map(|value| value.source.clone())
        .or_else(|| package.callables.first().map(|value| value.source.clone()))
        .or_else(|| package.files.first().map(|value| value.source.clone()))
        .unwrap_or_else(|| SourceRef::logical(["target-ast", "package"]))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestDialect;

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestPrimitive {
        Bool,
        I64,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestKnownType {
        Utf8String,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestRuntimeType {
        RuntimeError,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestConstructedType {
        I64List,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestKnownCallable {
        Increment,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestRuntimeCallable {
        IsTruthy,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestInvocation {
        Static,
        Instance,
        Constructor,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestVisibility {
        Private,
        Public,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestDeclarationKind {
        Record,
        Interface,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestSymbolOrigin {
        ArithmeticRuntime,
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum TestTemplate {
        SourceFile,
        TestFile,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum TestExpression {
        BoolLiteral(bool),
        I64Literal(i64),
        Add {
            left: TargetExprId,
            right: TargetExprId,
        },
        Call {
            callable: TargetCallableRef<TestDialect>,
            invocation: TestInvocation,
            arguments: Vec<TargetExprId>,
        },
        Value(GeneratedValueId),
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum TestStatement {
        Evaluate(TargetExprId),
        Return(TargetExprId),
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum TestFileItem {
        Type(GeneratedTypeId),
        Callable(GeneratedCallableId),
        InterfaceMethod(GeneratedInterfaceMethodId),
        Value(GeneratedValueId),
        Statement(TargetStmtId),
    }

    struct BoolMarker;
    struct I64Marker;

    impl TargetTypeMarker<TestDialect> for BoolMarker {
        fn target_type() -> TargetTypeRef<TestDialect> {
            boolean()
        }
    }

    impl TargetTypeMarker<TestDialect> for I64Marker {
        fn target_type() -> TargetTypeRef<TestDialect> {
            integer()
        }
    }

    impl TargetDialect for TestDialect {
        type Unresolved = TargetAstPackage<Self>;
        type Resolved = ();

        fn verify_unresolved(&self, ast: &Self::Unresolved) -> Result<(), Vec<Diagnostic>> {
            verify_target_ast(ast)
        }

        fn verify_resolved(&self, _ast: &Self::Resolved) -> Result<(), Vec<Diagnostic>> {
            Ok(())
        }
    }

    impl TypedAstDialect for TestDialect {
        type PrimitiveType = TestPrimitive;
        type KnownType = TestKnownType;
        type RuntimeType = TestRuntimeType;
        type ConstructedType = TestConstructedType;
        type KnownCallable = TestKnownCallable;
        type RuntimeCallable = TestRuntimeCallable;
        type InvocationKind = TestInvocation;
        type Visibility = TestVisibility;
        type DeclarationKind = TestDeclarationKind;
        type SymbolOrigin = TestSymbolOrigin;
        type TemplateId = TestTemplate;
        type Expression = TestExpression;
        type Statement = TestStatement;
        type FileItem = TestFileItem;

        fn known_callable_signature(
            &self,
            callable: &Self::KnownCallable,
        ) -> TargetCallableSignature<Self> {
            match callable {
                TestKnownCallable::Increment => static_signature(vec![integer()], integer()),
            }
        }

        fn runtime_callable_signature(
            &self,
            callable: &Self::RuntimeCallable,
        ) -> TargetCallableSignature<Self> {
            match callable {
                TestRuntimeCallable::IsTruthy => static_signature(vec![integer()], boolean()),
            }
        }

        fn verify_signature(&self, signature: &TargetCallableSignature<Self>) -> Vec<AstViolation> {
            let receiver_is_valid = match signature.invocation {
                TestInvocation::Instance => signature.receiver.is_some(),
                TestInvocation::Static | TestInvocation::Constructor => {
                    signature.receiver.is_none()
                }
            };
            if receiver_is_valid {
                vec![]
            } else {
                vec![violation(
                    DiagnosticCode::InvalidInvocation,
                    "receiver does not agree with invocation kind",
                )]
            }
        }
    }

    impl TargetExpressionNode<TestDialect> for TestExpression {
        fn child_expressions(&self) -> Vec<TargetExprId> {
            match self {
                Self::Add { left, right } => vec![*left, *right],
                Self::Call { arguments, .. } => arguments.clone(),
                Self::BoolLiteral(_) | Self::I64Literal(_) | Self::Value(_) => vec![],
            }
        }

        fn verify(
            &self,
            stored_type: &TargetTypeRef<TestDialect>,
            context: &TargetAstContext<'_, TestDialect>,
        ) -> Vec<AstViolation> {
            match self {
                Self::BoolLiteral(value) => {
                    let _observed = *value;
                    expect_type(stored_type, &boolean(), "boolean literal")
                }
                Self::I64Literal(value) => {
                    let _observed = *value;
                    expect_type(stored_type, &integer(), "integer literal")
                }
                Self::Add { left, right } => {
                    let mut violations = expect_type(stored_type, &integer(), "add result");
                    check_argument_type(&mut violations, context, *left, &integer(), "add left");
                    check_argument_type(&mut violations, context, *right, &integer(), "add right");
                    violations
                }
                Self::Call {
                    callable,
                    invocation,
                    arguments,
                } => verify_call(stored_type, callable, invocation, arguments, context),
                Self::Value(id) => match context.value(*id) {
                    Some(value) => expect_type(stored_type, &value.ty, "value reference"),
                    None => vec![violation(
                        DiagnosticCode::UnresolvedReference,
                        "value reference is out of bounds",
                    )],
                },
            }
        }
    }

    impl TargetStatementNode<TestDialect> for TestStatement {
        fn child_expressions(&self) -> Vec<TargetExprId> {
            match self {
                Self::Evaluate(expression) | Self::Return(expression) => vec![*expression],
            }
        }

        fn verify(&self, _context: &TargetAstContext<'_, TestDialect>) -> Vec<AstViolation> {
            vec![]
        }
    }

    impl TargetFileItemNode<TestDialect> for TestFileItem {
        fn verify(&self, context: &TargetAstContext<'_, TestDialect>) -> Vec<AstViolation> {
            let exists = match self {
                Self::Type(id) => context.generated_type(*id).is_some(),
                Self::Callable(id) => context.callable(*id).is_some(),
                Self::InterfaceMethod(id) => context.interface_method(*id).is_some(),
                Self::Value(id) => context.value(*id).is_some(),
                Self::Statement(id) => context.contains_statement(*id),
            };
            if exists {
                vec![]
            } else {
                vec![violation(
                    DiagnosticCode::UnresolvedReference,
                    "file item reference is out of bounds",
                )]
            }
        }
    }

    fn verify_call(
        stored_type: &TargetTypeRef<TestDialect>,
        callable: &TargetCallableRef<TestDialect>,
        invocation: &TestInvocation,
        arguments: &[TargetExprId],
        context: &TargetAstContext<'_, TestDialect>,
    ) -> Vec<AstViolation> {
        let Some(signature) = context.callable_signature(callable) else {
            return vec![violation(
                DiagnosticCode::UnresolvedReference,
                "callable reference is out of bounds",
            )];
        };
        let mut violations = Vec::new();
        if invocation != &signature.invocation {
            violations.push(violation(
                DiagnosticCode::InvalidInvocation,
                "call uses the wrong invocation kind",
            ));
        }
        if arguments.len() != signature.parameters.len() {
            violations.push(violation(
                DiagnosticCode::InvalidInvocation,
                "call has the wrong argument count",
            ));
        }
        for (argument, expected) in arguments.iter().zip(&signature.parameters) {
            check_argument_type(
                &mut violations,
                context,
                *argument,
                expected,
                "call argument",
            );
        }
        violations.extend(expect_type(
            stored_type,
            &signature.return_type,
            "call result",
        ));
        violations
    }

    fn check_argument_type(
        violations: &mut Vec<AstViolation>,
        context: &TargetAstContext<'_, TestDialect>,
        expression: TargetExprId,
        expected: &TargetTypeRef<TestDialect>,
        category: &str,
    ) {
        if let Some(actual) = context.expression_type(expression)
            && actual != expected
        {
            violations.push(violation(
                DiagnosticCode::TypeMismatch,
                format!("{category} has the wrong target type"),
            ));
        }
    }

    fn expect_type(
        actual: &TargetTypeRef<TestDialect>,
        expected: &TargetTypeRef<TestDialect>,
        category: &str,
    ) -> Vec<AstViolation> {
        if actual == expected {
            vec![]
        } else {
            vec![violation(
                DiagnosticCode::TypeMismatch,
                format!("{category} has the wrong stored type"),
            )]
        }
    }

    fn violation(code: DiagnosticCode, message: impl Into<String>) -> AstViolation {
        AstViolation::new(code, message)
    }

    fn integer() -> TargetTypeRef<TestDialect> {
        TargetTypeRef::Primitive(TestPrimitive::I64)
    }

    fn boolean() -> TargetTypeRef<TestDialect> {
        TargetTypeRef::Primitive(TestPrimitive::Bool)
    }

    fn static_signature(
        parameters: Vec<TargetTypeRef<TestDialect>>,
        return_type: TargetTypeRef<TestDialect>,
    ) -> TargetCallableSignature<TestDialect> {
        TargetCallableSignature {
            invocation: TestInvocation::Static,
            receiver: None,
            parameters,
            return_type,
        }
    }

    fn source(segment: &str) -> SourceRef {
        SourceRef::logical(["test-dialect", segment])
    }

    #[derive(Clone, Copy)]
    struct FixtureIds {
        interface_type: GeneratedTypeId,
        callable: GeneratedCallableId,
        interface_method: GeneratedInterfaceMethodId,
        value: GeneratedValueId,
        sum: TargetExprId,
        call: TargetExprId,
        statement: TargetStmtId,
        file: TargetFileId,
    }

    fn valid_fixture() -> (TargetAstPackage<TestDialect>, FixtureIds) {
        let mut builder = TargetAstBuilder::new(TestDialect);
        let interface_type = builder.generated_type(GeneratedType {
            name: "Counter".to_owned(),
            kind: TestDeclarationKind::Interface,
            visibility: TestVisibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter),
            source: source("interface"),
        });
        let _record_type = builder.generated_type(GeneratedType {
            name: "CounterState".to_owned(),
            kind: TestDeclarationKind::Record,
            visibility: TestVisibility::Private,
            origin: GeneratedOrigin::Runtime(TestSymbolOrigin::ArithmeticRuntime),
            source: source("record"),
        });
        let type_parameter = builder.type_parameter(source("type-parameter"));
        let _generic_callable = builder.callable(GeneratedCallable {
            name: "identity".to_owned(),
            signature: static_signature(
                vec![TargetTypeRef::TypeParameter(type_parameter)],
                TargetTypeRef::TypeParameter(type_parameter),
            ),
            visibility: TestVisibility::Private,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::OwnershipAdapter),
            source: source("identity"),
        });
        let callable = builder.callable(GeneratedCallable {
            name: "add".to_owned(),
            signature: static_signature(vec![integer(), integer()], integer()),
            visibility: TestVisibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
            source: source("callable"),
        });
        let interface_method = builder.interface_method(GeneratedInterfaceMethod {
            owner: interface_type,
            name: "next".to_owned(),
            signature: TargetCallableSignature {
                invocation: TestInvocation::Instance,
                receiver: Some(TargetTypeRef::Generated(interface_type)),
                parameters: vec![],
                return_type: integer(),
            },
            origin: GeneratedOrigin::Synthesized(SynthesisReason::InterfaceAdapter),
            source: source("interface-method"),
        });
        let value = builder.value(GeneratedValue {
            name: "seed".to_owned(),
            ty: integer(),
            origin: GeneratedOrigin::Synthesized(SynthesisReason::EvaluationTemporary),
            source: source("value"),
        });
        let _known_value = builder.value(GeneratedValue {
            name: "label".to_owned(),
            ty: TargetTypeRef::Known(TestKnownType::Utf8String),
            origin: GeneratedOrigin::Synthesized(SynthesisReason::TestHarness),
            source: source("known-value"),
        });
        let _runtime_value = builder.value(GeneratedValue {
            name: "error".to_owned(),
            ty: TargetTypeRef::Runtime(TestRuntimeType::RuntimeError),
            origin: GeneratedOrigin::Runtime(TestSymbolOrigin::ArithmeticRuntime),
            source: source("runtime-value"),
        });
        let _constructed_value = builder.value(GeneratedValue {
            name: "numbers".to_owned(),
            ty: TargetTypeRef::Constructed(TestConstructedType::I64List),
            origin: GeneratedOrigin::Synthesized(SynthesisReason::EvaluationTemporary),
            source: source("constructed-value"),
        });

        let one = builder.expression::<I64Marker>(TestExpression::I64Literal(1), source("one"));
        let two = builder.expression::<I64Marker>(TestExpression::I64Literal(2), source("two"));
        let _truth =
            builder.expression::<BoolMarker>(TestExpression::BoolLiteral(true), source("truth"));
        let sum = builder.expression::<I64Marker>(
            TestExpression::Add {
                left: one.id(),
                right: two.id(),
            },
            source("sum"),
        );
        let _value_expression = builder.dynamic_expression(
            TestExpression::Value(value),
            integer(),
            source("value-expression"),
        );
        let _known_call = builder.expression::<I64Marker>(
            TestExpression::Call {
                callable: TargetCallableRef::Known(TestKnownCallable::Increment),
                invocation: TestInvocation::Static,
                arguments: vec![one.id()],
            },
            source("known-call"),
        );
        let _runtime_call = builder.expression::<BoolMarker>(
            TestExpression::Call {
                callable: TargetCallableRef::Runtime(TestRuntimeCallable::IsTruthy),
                invocation: TestInvocation::Static,
                arguments: vec![one.id()],
            },
            source("runtime-call"),
        );
        let _interface_call = builder.expression::<I64Marker>(
            TestExpression::Call {
                callable: TargetCallableRef::Interface(interface_method),
                invocation: TestInvocation::Instance,
                arguments: vec![],
            },
            source("interface-call"),
        );
        let call = builder.expression::<I64Marker>(
            TestExpression::Call {
                callable: TargetCallableRef::Generated(callable),
                invocation: TestInvocation::Static,
                arguments: vec![one.id(), sum.id()],
            },
            source("generated-call"),
        );
        let _return_statement =
            builder.statement(TestStatement::Return(call.id()), source("return-statement"));
        let statement = builder.statement(TestStatement::Evaluate(call.id()), source("statement"));
        let file = builder.file(TargetFile {
            path: "src/generated.polytest".to_owned(),
            role: TargetFileRole::Source,
            items: vec![
                TestFileItem::Type(interface_type),
                TestFileItem::Callable(callable),
                TestFileItem::InterfaceMethod(interface_method),
                TestFileItem::Value(value),
                TestFileItem::Statement(statement),
            ],
            template: TestTemplate::SourceFile,
            source: source("file"),
        });
        let _test_file = builder.file(TargetFile {
            path: "test/generated_test.polytest".to_owned(),
            role: TargetFileRole::Test,
            items: vec![],
            template: TestTemplate::TestFile,
            source: source("test-file"),
        });
        builder.group(TargetFileGroup {
            files: vec![file, TargetFileId::from_index(1)],
            source: source("group"),
        });
        (
            builder.build(),
            FixtureIds {
                interface_type,
                callable,
                interface_method,
                value,
                sum: sum.id(),
                call: call.id(),
                statement,
                file,
            },
        )
    }

    #[test]
    fn typed_builders_dynamic_references_and_traversal_verify() {
        let (package, ids) = valid_fixture();
        assert_eq!(verify_target_ast(&package), Ok(()));
        assert_eq!(
            package.generated_type(ids.interface_type).unwrap().name,
            "Counter"
        );
        assert_eq!(package.callable(ids.callable).unwrap().name, "add");
        assert_eq!(
            package
                .interface_method(ids.interface_method)
                .unwrap()
                .owner,
            ids.interface_type
        );
        assert_eq!(package.value(ids.value).unwrap().ty, integer());
        assert_eq!(package.expression_type(ids.sum), Some(&integer()));
        assert_eq!(package.expression_type(ids.call), Some(&integer()));
        assert_eq!(package.file(ids.file).unwrap().items.len(), 5);
        assert_eq!(package.groups[0].files.len(), 2);
        assert!(TargetAstContext { package: &package }.contains_statement(ids.statement));
    }

    #[test]
    fn canonical_dump_is_non_source_and_deterministic() {
        let first = valid_fixture().0.canonical_dump();
        let second = valid_fixture().0.canonical_dump();
        let third = valid_fixture().0.canonical_dump();
        assert_eq!(first, second);
        assert_eq!(second, third);
        assert!(first.contains("I64Literal"));
        assert!(first.contains("GeneratedCallableId"));
        assert!(!first.contains("class Counter"));
        assert!(!first.contains("fn add"));
    }

    #[test]
    fn forged_expression_faults_report_stable_diagnostics() {
        let (mut package, ids) = valid_fixture();
        package.expressions_mut()[ids.sum.index()].ty = boolean();
        package.expressions_mut()[ids.call.index()].node = TestExpression::Call {
            callable: TargetCallableRef::Generated(GeneratedCallableId::from_index(99)),
            invocation: TestInvocation::Constructor,
            arguments: vec![TargetExprId::from_index(99)],
        };
        let diagnostics = verify_target_ast(&package).unwrap_err();
        let codes = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<BTreeSet<_>>();
        assert!(codes.contains(&DiagnosticCode::TypeMismatch));
        assert!(codes.contains(&DiagnosticCode::UnresolvedReference));
        assert_eq!(diagnostics, {
            let mut repeated = verify_target_ast(&package).unwrap_err();
            sort_diagnostics(&mut repeated);
            repeated
        });
    }

    #[test]
    fn every_builder_category_has_a_rejected_forged_counterpart() {
        let (mut package, ids) = valid_fixture();
        package.types[0].name.clear();
        package.types[0].source = SourceRef::logical(Vec::<String>::new());
        package.callables[ids.callable.index()].signature.receiver = Some(integer());
        package.interface_methods[ids.interface_method.index()].owner =
            GeneratedTypeId::from_index(99);
        package.values[ids.value.index()].ty =
            TargetTypeRef::TypeParameter(TargetTypeParameterId::from_index(99));
        package.type_parameters[0] = SourceRef::logical(Vec::<String>::new());
        package.statements[ids.statement.index()].node =
            TestStatement::Evaluate(TargetExprId::from_index(99));
        package.files[ids.file.index()].path = "../escaped.polytest".to_owned();
        package.files[ids.file.index()]
            .items
            .push(TestFileItem::Callable(GeneratedCallableId::from_index(99)));
        package.groups_mut()[0].files.push(ids.file);

        let diagnostics = verify_target_ast(&package).unwrap_err();
        let codes = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<BTreeSet<_>>();
        for expected in [
            DiagnosticCode::InvalidIdentifier,
            DiagnosticCode::InvalidInvocation,
            DiagnosticCode::InvalidStructure,
            DiagnosticCode::UnresolvedReference,
            DiagnosticCode::UnsafeOutputPath,
            DiagnosticCode::DuplicateDeclaration,
        ] {
            assert!(
                codes.contains(&expected),
                "missing {expected:?}: {diagnostics:#?}"
            );
        }
    }

    #[test]
    fn ungrouped_files_and_forward_expression_references_are_rejected() {
        let (mut package, _) = valid_fixture();
        package.groups_mut()[0].files.pop();
        package.expressions_mut()[0].node = TestExpression::Add {
            left: TargetExprId::from_index(1),
            right: TargetExprId::from_index(1),
        };
        let diagnostics = verify_target_ast(&package).unwrap_err();
        let codes = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<BTreeSet<_>>();
        assert!(codes.contains(&DiagnosticCode::InvalidStructure));
        assert!(codes.contains(&DiagnosticCode::InvalidControlFlow));
    }
}
