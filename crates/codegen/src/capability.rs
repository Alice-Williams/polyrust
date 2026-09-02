use std::collections::BTreeMap;

use portable_core_ir::*;
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef, sort_diagnostics};

use crate::TargetId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeclarationFeature {
    Constant,
    Alias,
    Record,
    Enum,
    Interface,
    Implementation,
    Function,
    Test,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeFeature {
    Unit,
    Bool,
    I32,
    I64,
    F64,
    Char,
    String,
    Bytes,
    List,
    Option,
    Result,
    Record,
    Enum,
    Interface,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ControlFeature {
    Block,
    Let,
    ForEach,
    Return,
    Evaluate,
    If,
    Match,
    WildcardPattern,
    BoolPattern,
    EnumPattern,
    NonePattern,
    SomePattern,
    OkPattern,
    ErrPattern,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterfaceFeature {
    Declaration,
    Conformance,
    MultipleConformance,
    StaticDispatch,
    DynamicDispatch,
    InterfaceValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OwnershipFeature {
    OnceLeftToRight,
    OwnedImmutableValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperationFeature {
    Literal,
    Local,
    Constant,
    SelfValue,
    ConstructRecord,
    ConstructEnum,
    ConstructSome,
    ConstructNone,
    ConstructOk,
    ConstructErr,
    ConstructList,
    CoerceInterface,
    Field,
    Call,
    StaticMethodCall,
    InterfaceCall,
    Unary(CoreUnaryIntrinsic),
    Binary(CoreBinaryIntrinsic),
    Ternary(CoreTernaryIntrinsic),
    Variadic(CoreVariadicIntrinsic),
    If,
    Match,
    Block,
}

/// Closed portable feature families. Registries must match every family.
///
/// ```compile_fail
/// use portable_codegen::CoreFeature;
///
/// fn incomplete(feature: CoreFeature) {
///     match feature {
///         CoreFeature::Type(_) => {}
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoreFeature {
    Declaration(DeclarationFeature),
    Type(TypeFeature),
    Control(ControlFeature),
    Interface(InterfaceFeature),
    Operation(OperationFeature),
    Ownership(OwnershipFeature),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InterfaceUse {
    pub method_count: u32,
    pub has_interface_values: bool,
    pub parameter_types: Vec<CoreTypeId>,
    pub return_types: Vec<CoreTypeId>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureShape {
    Unit,
    Aggregate { field_count: u32 },
    Callable { parameter_count: u32 },
    Interface(InterfaceUse),
    Variadic { operand_count: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeatureUse {
    feature: CoreFeature,
    shape: FeatureShape,
    source: SourceRef,
}

impl FeatureUse {
    pub const fn feature(&self) -> CoreFeature {
        self.feature
    }

    pub const fn shape(&self) -> &FeatureShape {
        &self.shape
    }

    pub const fn source(&self) -> &SourceRef {
        &self.source
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FeatureUseSet {
    uses: Vec<FeatureUse>,
}

impl FeatureUseSet {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &FeatureUse> {
        self.uses.iter()
    }

    pub fn contains(&self, feature: CoreFeature) -> bool {
        self.uses.iter().any(|usage| usage.feature == feature)
    }

    pub fn len(&self) -> usize {
        self.uses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.uses.is_empty()
    }
}

pub fn collect_core_features(program: &CoreProgram) -> FeatureUseSet {
    FeatureCollector::new(program).collect()
}

struct FeatureCollector<'a> {
    program: &'a CoreProgram,
    uses: BTreeMap<(CoreFeature, FeatureShape), SourceRef>,
}

impl<'a> FeatureCollector<'a> {
    fn new(program: &'a CoreProgram) -> Self {
        Self {
            program,
            uses: BTreeMap::new(),
        }
    }

    fn collect(mut self) -> FeatureUseSet {
        self.collect_declarations();
        self.collect_expressions();
        self.collect_blocks();
        FeatureUseSet {
            uses: self
                .uses
                .into_iter()
                .map(|((feature, shape), source)| FeatureUse {
                    feature,
                    shape,
                    source,
                })
                .collect(),
        }
    }

    fn add(&mut self, feature: CoreFeature, shape: FeatureShape, source: &SourceRef) {
        self.uses
            .entry((feature, shape))
            .or_insert_with(|| source.clone());
    }

    fn unit(&mut self, feature: CoreFeature, source: &SourceRef) {
        self.add(feature, FeatureShape::Unit, source);
    }

    fn collect_type(&mut self, id: CoreTypeId, source: &SourceRef) {
        let Some(ty) = self.program.types().get(id) else {
            return;
        };
        let feature = match ty {
            CoreType::Unit => TypeFeature::Unit,
            CoreType::Bool => TypeFeature::Bool,
            CoreType::I32 => TypeFeature::I32,
            CoreType::I64 => TypeFeature::I64,
            CoreType::F64 => TypeFeature::F64,
            CoreType::Char => TypeFeature::Char,
            CoreType::String => TypeFeature::String,
            CoreType::Bytes => TypeFeature::Bytes,
            CoreType::List(inner) => {
                self.collect_type(*inner, source);
                TypeFeature::List
            }
            CoreType::Option(inner) => {
                self.collect_type(*inner, source);
                TypeFeature::Option
            }
            CoreType::Result { ok, error } => {
                self.collect_type(*ok, source);
                self.collect_type(*error, source);
                TypeFeature::Result
            }
            CoreType::Record(_) => TypeFeature::Record,
            CoreType::Enum(_) => TypeFeature::Enum,
            CoreType::Interface(_) => {
                self.unit(
                    CoreFeature::Interface(InterfaceFeature::InterfaceValue),
                    source,
                );
                TypeFeature::Interface
            }
        };
        self.unit(CoreFeature::Type(feature), source);
    }

    fn collect_signature(
        &mut self,
        parameters: &[CoreParameter],
        return_type: CoreTypeId,
        source: &SourceRef,
    ) {
        for parameter in parameters {
            self.collect_type(parameter.ty, &parameter.header.source);
        }
        self.collect_type(return_type, source);
    }

    fn collect_declarations(&mut self) {
        for constant in self.program.constants() {
            self.unit(
                CoreFeature::Declaration(DeclarationFeature::Constant),
                &constant.header.source,
            );
            self.collect_type(constant.ty, &constant.header.source);
            self.collect_constant(&constant.value);
        }
        for alias in self.program.aliases() {
            self.unit(
                CoreFeature::Declaration(DeclarationFeature::Alias),
                &alias.header.source,
            );
            self.collect_type(alias.target, &alias.header.source);
        }
        for record in self.program.records() {
            self.add(
                CoreFeature::Declaration(DeclarationFeature::Record),
                FeatureShape::Aggregate {
                    field_count: usize_to_u32(record.fields.len()),
                },
                &record.header.source,
            );
            for field in &record.fields {
                if let Some(field) = self.program.field(*field) {
                    self.collect_type(field.ty, &field.header.source);
                }
            }
        }
        for enumeration in self.program.enums() {
            self.add(
                CoreFeature::Declaration(DeclarationFeature::Enum),
                FeatureShape::Aggregate {
                    field_count: usize_to_u32(enumeration.variants.len()),
                },
                &enumeration.header.source,
            );
            for variant in &enumeration.variants {
                if let Some(variant) = self.program.variant(*variant) {
                    for field in &variant.fields {
                        if let Some(field) = self.program.field(*field) {
                            self.collect_type(field.ty, &field.header.source);
                        }
                    }
                }
            }
        }
        for interface in self.program.interfaces() {
            self.unit(
                CoreFeature::Declaration(DeclarationFeature::Interface),
                &interface.header.source,
            );
            let shape = self.interface_use(interface);
            self.add(
                CoreFeature::Interface(InterfaceFeature::Declaration),
                FeatureShape::Interface(shape),
                &interface.header.source,
            );
            for method in &interface.methods {
                if let Some(method) = self.program.interface_method(*method) {
                    self.collect_signature(
                        &method.parameters,
                        method.return_type,
                        &method.header.source,
                    );
                }
            }
        }
        let mut conformances = BTreeMap::<CoreRecordId, usize>::new();
        for implementation in self.program.implementations() {
            self.unit(
                CoreFeature::Declaration(DeclarationFeature::Implementation),
                &implementation.header.source,
            );
            self.unit(
                CoreFeature::Interface(InterfaceFeature::Conformance),
                &implementation.header.source,
            );
            *conformances.entry(implementation.record).or_default() += 1;
            for method in &implementation.methods {
                if let Some(method) = self.program.implementation_method(*method) {
                    self.collect_signature(
                        &method.parameters,
                        method.return_type,
                        &method.header.source,
                    );
                }
            }
        }
        for (record, count) in conformances {
            if count > 1
                && let Some(record) = self.program.record(record)
            {
                self.unit(
                    CoreFeature::Interface(InterfaceFeature::MultipleConformance),
                    &record.header.source,
                );
            }
        }
        for function in self.program.functions() {
            self.add(
                CoreFeature::Declaration(DeclarationFeature::Function),
                FeatureShape::Callable {
                    parameter_count: usize_to_u32(function.parameters.len()),
                },
                &function.header.source,
            );
            self.collect_signature(
                &function.parameters,
                function.return_type,
                &function.header.source,
            );
        }
        for test in self.program.tests() {
            self.unit(
                CoreFeature::Declaration(DeclarationFeature::Test),
                &test.header.source,
            );
            match &test.invocation {
                CoreTestInvocation::Function { arguments, .. } => {
                    self.collect_typed_values(arguments, &test.header.source)
                }
                CoreTestInvocation::Method {
                    receiver,
                    arguments,
                    ..
                } => {
                    self.collect_type(receiver.ty, &test.header.source);
                    self.collect_typed_values(arguments, &test.header.source);
                }
            }
            match &test.expected {
                CoreExpectedOutcome::Value(value) | CoreExpectedOutcome::Error(value) => {
                    self.collect_type(value.ty, &test.header.source)
                }
            }
        }
    }

    fn interface_use(&self, interface: &CoreInterface) -> InterfaceUse {
        let methods = interface
            .methods
            .iter()
            .filter_map(|method| self.program.interface_method(*method))
            .collect::<Vec<_>>();
        InterfaceUse {
            method_count: usize_to_u32(methods.len()),
            has_interface_values: self.program.types().iter().any(|(_, ty)| {
                matches!(ty, CoreType::Interface(candidate) if self.program.interface(*candidate) == Some(interface))
            }),
            parameter_types: methods
                .iter()
                .flat_map(|method| method.parameters.iter().map(|parameter| parameter.ty))
                .collect(),
            return_types: methods.iter().map(|method| method.return_type).collect(),
        }
    }

    fn collect_typed_values(&mut self, values: &[CoreTypedValue], source: &SourceRef) {
        for value in values {
            self.collect_type(value.ty, source);
        }
    }

    fn collect_constant(&mut self, expression: &CoreConstantExpr) {
        let (feature, shape) = match &expression.kind {
            CoreConstantExprKind::Literal(_) => (OperationFeature::Literal, FeatureShape::Unit),
            CoreConstantExprKind::Constant(_) => (OperationFeature::Constant, FeatureShape::Unit),
            CoreConstantExprKind::Record { fields, .. } => (
                OperationFeature::ConstructRecord,
                FeatureShape::Aggregate {
                    field_count: usize_to_u32(fields.len()),
                },
            ),
            CoreConstantExprKind::Enum { fields, .. } => (
                OperationFeature::ConstructEnum,
                FeatureShape::Aggregate {
                    field_count: usize_to_u32(fields.len()),
                },
            ),
            CoreConstantExprKind::Some(_) => (OperationFeature::ConstructSome, FeatureShape::Unit),
            CoreConstantExprKind::None { .. } => {
                (OperationFeature::ConstructNone, FeatureShape::Unit)
            }
            CoreConstantExprKind::Ok { .. } => (OperationFeature::ConstructOk, FeatureShape::Unit),
            CoreConstantExprKind::Err { .. } => {
                (OperationFeature::ConstructErr, FeatureShape::Unit)
            }
            CoreConstantExprKind::List { elements, .. } => (
                OperationFeature::ConstructList,
                FeatureShape::Aggregate {
                    field_count: usize_to_u32(elements.len()),
                },
            ),
            CoreConstantExprKind::Intrinsic(intrinsic) => intrinsic_feature(intrinsic),
        };
        self.add(CoreFeature::Operation(feature), shape, &expression.source);
        match &expression.kind {
            CoreConstantExprKind::Record { fields, .. }
            | CoreConstantExprKind::Enum { fields, .. } => {
                for field in fields {
                    self.collect_constant(&field.value);
                }
            }
            CoreConstantExprKind::Some(value)
            | CoreConstantExprKind::Ok { value, .. }
            | CoreConstantExprKind::Err { value, .. } => self.collect_constant(value),
            CoreConstantExprKind::List { elements, .. } => {
                for value in elements {
                    self.collect_constant(value);
                }
            }
            CoreConstantExprKind::Intrinsic(intrinsic) => {
                for value in intrinsic_values(intrinsic) {
                    self.collect_constant(value);
                }
            }
            CoreConstantExprKind::Literal(_)
            | CoreConstantExprKind::Constant(_)
            | CoreConstantExprKind::None { .. } => {}
        }
    }

    fn collect_expressions(&mut self) {
        for (_, expression) in self.program.expressions().iter() {
            self.collect_type(expression.ty, &expression.source);
            self.unit(
                CoreFeature::Ownership(OwnershipFeature::OnceLeftToRight),
                &expression.source,
            );
            self.unit(
                CoreFeature::Ownership(OwnershipFeature::OwnedImmutableValue),
                &expression.source,
            );
            let (feature, shape) = expression_feature(&expression.kind);
            self.add(CoreFeature::Operation(feature), shape, &expression.source);
            match &expression.kind {
                CoreExprKind::StaticMethodCall { .. } => self.unit(
                    CoreFeature::Interface(InterfaceFeature::StaticDispatch),
                    &expression.source,
                ),
                CoreExprKind::InterfaceCall { .. } => self.unit(
                    CoreFeature::Interface(InterfaceFeature::DynamicDispatch),
                    &expression.source,
                ),
                CoreExprKind::CoerceInterface { .. } => self.unit(
                    CoreFeature::Interface(InterfaceFeature::InterfaceValue),
                    &expression.source,
                ),
                CoreExprKind::If { .. } => {
                    self.unit(CoreFeature::Control(ControlFeature::If), &expression.source)
                }
                CoreExprKind::Match { arms, .. } => {
                    self.add(
                        CoreFeature::Control(ControlFeature::Match),
                        FeatureShape::Aggregate {
                            field_count: usize_to_u32(arms.len()),
                        },
                        &expression.source,
                    );
                    for arm in arms {
                        self.collect_pattern(&arm.pattern);
                    }
                }
                CoreExprKind::Block(_) => self.unit(
                    CoreFeature::Control(ControlFeature::Block),
                    &expression.source,
                ),
                CoreExprKind::Literal(_)
                | CoreExprKind::Local(_)
                | CoreExprKind::Constant(_)
                | CoreExprKind::SelfValue(_)
                | CoreExprKind::ConstructRecord { .. }
                | CoreExprKind::ConstructEnum { .. }
                | CoreExprKind::ConstructSome(_)
                | CoreExprKind::ConstructNone { .. }
                | CoreExprKind::ConstructOk { .. }
                | CoreExprKind::ConstructErr { .. }
                | CoreExprKind::ConstructList { .. }
                | CoreExprKind::Field { .. }
                | CoreExprKind::Call { .. }
                | CoreExprKind::Intrinsic(_) => {}
            }
        }
    }

    fn collect_blocks(&mut self) {
        for (_, block) in self.program.blocks().iter() {
            self.unit(CoreFeature::Control(ControlFeature::Block), &block.source);
            self.collect_type(block.result_type, &block.source);
            for statement in &block.statements {
                match statement {
                    CoreStatement::Let { source, .. } => {
                        self.unit(CoreFeature::Control(ControlFeature::Let), source)
                    }
                    CoreStatement::ForEach { source, .. } => {
                        self.unit(CoreFeature::Control(ControlFeature::ForEach), source)
                    }
                    CoreStatement::Return { source, .. } => {
                        self.unit(CoreFeature::Control(ControlFeature::Return), source)
                    }
                    CoreStatement::Evaluate { source, .. } => {
                        self.unit(CoreFeature::Control(ControlFeature::Evaluate), source)
                    }
                }
            }
        }
    }

    fn collect_pattern(&mut self, pattern: &CorePattern) {
        let (feature, source) = match pattern {
            CorePattern::Wildcard { source } => (ControlFeature::WildcardPattern, source),
            CorePattern::Bool { source, .. } => (ControlFeature::BoolPattern, source),
            CorePattern::EnumVariant { source, .. } => (ControlFeature::EnumPattern, source),
            CorePattern::None { source } => (ControlFeature::NonePattern, source),
            CorePattern::Some { source, .. } => (ControlFeature::SomePattern, source),
            CorePattern::Ok { source, .. } => (ControlFeature::OkPattern, source),
            CorePattern::Err { source, .. } => (ControlFeature::ErrPattern, source),
        };
        self.unit(CoreFeature::Control(feature), source);
    }
}

fn expression_feature(expression: &CoreExprKind) -> (OperationFeature, FeatureShape) {
    match expression {
        CoreExprKind::Literal(_) => (OperationFeature::Literal, FeatureShape::Unit),
        CoreExprKind::Local(_) => (OperationFeature::Local, FeatureShape::Unit),
        CoreExprKind::Constant(_) => (OperationFeature::Constant, FeatureShape::Unit),
        CoreExprKind::SelfValue(_) => (OperationFeature::SelfValue, FeatureShape::Unit),
        CoreExprKind::ConstructRecord { fields, .. } => (
            OperationFeature::ConstructRecord,
            FeatureShape::Aggregate {
                field_count: usize_to_u32(fields.len()),
            },
        ),
        CoreExprKind::ConstructEnum { fields, .. } => (
            OperationFeature::ConstructEnum,
            FeatureShape::Aggregate {
                field_count: usize_to_u32(fields.len()),
            },
        ),
        CoreExprKind::ConstructSome(_) => (OperationFeature::ConstructSome, FeatureShape::Unit),
        CoreExprKind::ConstructNone { .. } => (OperationFeature::ConstructNone, FeatureShape::Unit),
        CoreExprKind::ConstructOk { .. } => (OperationFeature::ConstructOk, FeatureShape::Unit),
        CoreExprKind::ConstructErr { .. } => (OperationFeature::ConstructErr, FeatureShape::Unit),
        CoreExprKind::ConstructList { elements, .. } => (
            OperationFeature::ConstructList,
            FeatureShape::Aggregate {
                field_count: usize_to_u32(elements.len()),
            },
        ),
        CoreExprKind::CoerceInterface { .. } => {
            (OperationFeature::CoerceInterface, FeatureShape::Unit)
        }
        CoreExprKind::Field { .. } => (OperationFeature::Field, FeatureShape::Unit),
        CoreExprKind::Call { arguments, .. } => (
            OperationFeature::Call,
            FeatureShape::Callable {
                parameter_count: usize_to_u32(arguments.len()),
            },
        ),
        CoreExprKind::StaticMethodCall { arguments, .. } => (
            OperationFeature::StaticMethodCall,
            FeatureShape::Callable {
                parameter_count: usize_to_u32(arguments.len()),
            },
        ),
        CoreExprKind::InterfaceCall { arguments, .. } => (
            OperationFeature::InterfaceCall,
            FeatureShape::Callable {
                parameter_count: usize_to_u32(arguments.len()),
            },
        ),
        CoreExprKind::Intrinsic(intrinsic) => intrinsic_feature(intrinsic),
        CoreExprKind::If { .. } => (OperationFeature::If, FeatureShape::Unit),
        CoreExprKind::Match { arms, .. } => (
            OperationFeature::Match,
            FeatureShape::Aggregate {
                field_count: usize_to_u32(arms.len()),
            },
        ),
        CoreExprKind::Block(_) => (OperationFeature::Block, FeatureShape::Unit),
    }
}

fn intrinsic_feature<T>(intrinsic: &CoreIntrinsicExpr<T>) -> (OperationFeature, FeatureShape) {
    match intrinsic {
        CoreIntrinsicExpr::Unary { operation, .. } => {
            (OperationFeature::Unary(*operation), FeatureShape::Unit)
        }
        CoreIntrinsicExpr::Binary { operation, .. } => {
            (OperationFeature::Binary(*operation), FeatureShape::Unit)
        }
        CoreIntrinsicExpr::Ternary { operation, .. } => {
            (OperationFeature::Ternary(*operation), FeatureShape::Unit)
        }
        CoreIntrinsicExpr::Variadic {
            operation,
            arguments,
        } => (
            OperationFeature::Variadic(*operation),
            FeatureShape::Variadic {
                operand_count: usize_to_u32(arguments.len()),
            },
        ),
    }
}

fn intrinsic_values<T>(intrinsic: &CoreIntrinsicExpr<T>) -> Vec<&T> {
    match intrinsic {
        CoreIntrinsicExpr::Unary { operand, .. } => vec![operand],
        CoreIntrinsicExpr::Binary { left, right, .. } => vec![left, right],
        CoreIntrinsicExpr::Ternary {
            first,
            second,
            third,
            ..
        } => vec![first, second, third],
        CoreIntrinsicExpr::Variadic { arguments, .. } => arguments.iter().collect(),
    }
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnsupportedReason {
    NotImplemented,
    Unrepresentable,
    UnsupportedShape,
    ToolchainUnavailable,
    ConflictingOptions,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedSupport {
    pub reason: UnsupportedReason,
    pub detail: String,
    pub option: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SupportDecision<S> {
    Native(S),
    Emulated(S),
    Unsupported(UnsupportedSupport),
}

pub trait CapabilityRegistry: Send + Sync + 'static {
    type Strategy: Clone + Eq + std::fmt::Debug + Send + Sync + 'static;

    fn target(&self) -> TargetId;
    fn support(&self, usage: &FeatureUse) -> SupportDecision<Self::Strategy>;
    fn has_lowering(&self, strategy: &Self::Strategy) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SupportMode {
    Native,
    Emulated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedFeature<S> {
    pub usage: FeatureUse,
    pub mode: SupportMode,
    pub strategy: S,
}

pub type CapabilityMatrix<S> = BTreeMap<TargetId, Vec<SelectedFeature<S>>>;
pub type ValidatedPreflight<O, S> = (ValidatedOptions<O>, Vec<SelectedFeature<S>>);

pub fn preflight_capabilities<R: CapabilityRegistry>(
    program: &CoreProgram,
    registry: &R,
) -> Result<Vec<SelectedFeature<R::Strategy>>, Vec<Diagnostic>> {
    let mut selected = Vec::new();
    let mut diagnostics = Vec::new();
    for usage in collect_core_features(program).iter() {
        match registry.support(usage) {
            SupportDecision::Native(strategy) | SupportDecision::Emulated(strategy)
                if !registry.has_lowering(&strategy) =>
            {
                let mut diagnostic = Diagnostic::error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "target {} selected unregistered strategy {strategy:?} for {:?}",
                        registry.target(),
                        usage.feature()
                    ),
                    usage.source().clone(),
                );
                diagnostic.target = Some(registry.target().to_string());
                diagnostics.push(diagnostic);
            }
            SupportDecision::Native(strategy) => selected.push(SelectedFeature {
                usage: usage.clone(),
                mode: SupportMode::Native,
                strategy,
            }),
            SupportDecision::Emulated(strategy) => selected.push(SelectedFeature {
                usage: usage.clone(),
                mode: SupportMode::Emulated,
                strategy,
            }),
            SupportDecision::Unsupported(unsupported) => {
                let mut diagnostic = Diagnostic::error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "target {} cannot preserve {:?} {:?}: {:?}: {}",
                        registry.target(),
                        usage.feature(),
                        usage.shape(),
                        unsupported.reason,
                        unsupported.detail
                    ),
                    usage.source().clone(),
                );
                diagnostic.target = Some(registry.target().to_string());
                if let Some(option) = unsupported.option {
                    diagnostic
                        .notes
                        .push(format!("conflicting option: {option}"));
                }
                diagnostics.push(diagnostic);
            }
        }
    }
    sort_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        Ok(selected)
    } else {
        Err(diagnostics)
    }
}

pub fn preflight_capability_matrix<R: CapabilityRegistry>(
    program: &CoreProgram,
    registries: &[R],
) -> Result<CapabilityMatrix<R::Strategy>, Vec<Diagnostic>> {
    let mut supported = BTreeMap::new();
    let mut diagnostics = Vec::new();
    for registry in registries {
        match preflight_capabilities(program, registry) {
            Ok(selected) => {
                supported.insert(registry.target(), selected);
            }
            Err(mut errors) => diagnostics.append(&mut errors),
        }
    }
    sort_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        Ok(supported)
    } else {
        Err(diagnostics)
    }
}

pub trait TypedTargetOptions: Clone + Send + Sync + 'static {
    fn validate(&self) -> Result<(), Vec<TypedOptionError>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedOptionError {
    pub reason: UnsupportedReason,
    pub option: &'static str,
    pub detail: String,
}

/// Validated options accepted by typed target preflight.
///
/// ```compile_fail
/// use portable_codegen::ValidatedOptions;
///
/// let _bypass = ValidatedOptions(());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedOptions<O>(O);

impl<O> ValidatedOptions<O> {
    pub const fn get(&self) -> &O {
        &self.0
    }
}

pub fn validate_typed_options<O: TypedTargetOptions>(
    options: O,
) -> Result<ValidatedOptions<O>, Vec<TypedOptionError>> {
    options.validate()?;
    Ok(ValidatedOptions(options))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedPreflightError {
    InvalidOptions(Vec<TypedOptionError>),
    UnsupportedFeatures(Vec<Diagnostic>),
}

pub fn validate_options_and_preflight<R: CapabilityRegistry, O: TypedTargetOptions>(
    program: &CoreProgram,
    registry: &R,
    options: O,
) -> Result<ValidatedPreflight<O, R::Strategy>, TypedPreflightError> {
    let options = validate_typed_options(options).map_err(TypedPreflightError::InvalidOptions)?;
    let selected = preflight_capabilities(program, registry)
        .map_err(TypedPreflightError::UnsupportedFeatures)?;
    Ok((options, selected))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltInStrategy {
    Direct,
    RuntimeHelper,
    TaggedUnion,
    FunctionTable,
    CompilerDerivedTypeScript,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltInLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    Cpp,
    C,
}

impl BuiltInLanguage {
    pub const ALL: [Self; 8] = [
        Self::Rust,
        Self::TypeScript,
        Self::JavaScript,
        Self::Python,
        Self::Go,
        Self::Java,
        Self::Cpp,
        Self::C,
    ];

    pub const fn target(self) -> &'static str {
        match self {
            Self::Rust => "dev.polyrust.rust",
            Self::TypeScript => "dev.polyrust.typescript",
            Self::JavaScript => "dev.polyrust.javascript",
            Self::Python => "dev.polyrust.python",
            Self::Go => "dev.polyrust.go",
            Self::Java => "dev.polyrust.java",
            Self::Cpp => "dev.polyrust.cpp",
            Self::C => "dev.polyrust.c",
        }
    }
}

macro_rules! built_in_registry {
    ($name:ident, $language:expr) => {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name;

        impl CapabilityRegistry for $name {
            type Strategy = BuiltInStrategy;

            fn target(&self) -> TargetId {
                TargetId::parse($language.target()).expect("built-in target ID is valid")
            }

            fn support(&self, usage: &FeatureUse) -> SupportDecision<Self::Strategy> {
                built_in_support($language, usage.feature())
            }

            fn has_lowering(&self, strategy: &Self::Strategy) -> bool {
                match strategy {
                    BuiltInStrategy::Direct
                    | BuiltInStrategy::RuntimeHelper
                    | BuiltInStrategy::TaggedUnion
                    | BuiltInStrategy::FunctionTable
                    | BuiltInStrategy::CompilerDerivedTypeScript => true,
                }
            }
        }
    };
}

built_in_registry!(RustCapabilityRegistry, BuiltInLanguage::Rust);
built_in_registry!(TypeScriptCapabilityRegistry, BuiltInLanguage::TypeScript);
built_in_registry!(JavaScriptCapabilityRegistry, BuiltInLanguage::JavaScript);
built_in_registry!(PythonCapabilityRegistry, BuiltInLanguage::Python);
built_in_registry!(GoCapabilityRegistry, BuiltInLanguage::Go);
built_in_registry!(JavaCapabilityRegistry, BuiltInLanguage::Java);
built_in_registry!(CppCapabilityRegistry, BuiltInLanguage::Cpp);
built_in_registry!(CCapabilityRegistry, BuiltInLanguage::C);

pub fn preflight_all_builtins(
    program: &CoreProgram,
) -> Result<BTreeMap<BuiltInLanguage, Vec<SelectedFeature<BuiltInStrategy>>>, Vec<Diagnostic>> {
    let mut supported = BTreeMap::new();
    let mut diagnostics = Vec::new();
    macro_rules! run {
        ($language:expr, $registry:expr) => {
            match preflight_capabilities(program, &$registry) {
                Ok(value) => {
                    supported.insert($language, value);
                }
                Err(mut errors) => diagnostics.append(&mut errors),
            }
        };
    }
    run!(BuiltInLanguage::Rust, RustCapabilityRegistry);
    run!(BuiltInLanguage::TypeScript, TypeScriptCapabilityRegistry);
    run!(BuiltInLanguage::JavaScript, JavaScriptCapabilityRegistry);
    run!(BuiltInLanguage::Python, PythonCapabilityRegistry);
    run!(BuiltInLanguage::Go, GoCapabilityRegistry);
    run!(BuiltInLanguage::Java, JavaCapabilityRegistry);
    run!(BuiltInLanguage::Cpp, CppCapabilityRegistry);
    run!(BuiltInLanguage::C, CCapabilityRegistry);
    sort_diagnostics(&mut diagnostics);
    if diagnostics.is_empty() {
        Ok(supported)
    } else {
        Err(diagnostics)
    }
}

fn built_in_support(
    language: BuiltInLanguage,
    feature: CoreFeature,
) -> SupportDecision<BuiltInStrategy> {
    use BuiltInStrategy::{Direct, TaggedUnion};
    let strategy = match feature {
        CoreFeature::Declaration(feature) => match feature {
            DeclarationFeature::Constant
            | DeclarationFeature::Alias
            | DeclarationFeature::Record
            | DeclarationFeature::Enum
            | DeclarationFeature::Interface
            | DeclarationFeature::Implementation
            | DeclarationFeature::Function
            | DeclarationFeature::Test => Direct,
        },
        CoreFeature::Type(feature) => match feature {
            TypeFeature::Unit
            | TypeFeature::Bool
            | TypeFeature::I32
            | TypeFeature::I64
            | TypeFeature::F64
            | TypeFeature::Char
            | TypeFeature::String
            | TypeFeature::Bytes
            | TypeFeature::List
            | TypeFeature::Record => Direct,
            TypeFeature::Option | TypeFeature::Result | TypeFeature::Enum => TaggedUnion,
            TypeFeature::Interface => interface_strategy(language),
        },
        CoreFeature::Control(feature) => match feature {
            ControlFeature::Block
            | ControlFeature::Let
            | ControlFeature::ForEach
            | ControlFeature::Return
            | ControlFeature::Evaluate
            | ControlFeature::If
            | ControlFeature::Match
            | ControlFeature::WildcardPattern
            | ControlFeature::BoolPattern
            | ControlFeature::EnumPattern
            | ControlFeature::NonePattern
            | ControlFeature::SomePattern
            | ControlFeature::OkPattern
            | ControlFeature::ErrPattern => Direct,
        },
        CoreFeature::Interface(feature) => match feature {
            InterfaceFeature::Declaration
            | InterfaceFeature::Conformance
            | InterfaceFeature::MultipleConformance
            | InterfaceFeature::StaticDispatch => Direct,
            InterfaceFeature::DynamicDispatch | InterfaceFeature::InterfaceValue => {
                interface_strategy(language)
            }
        },
        CoreFeature::Operation(feature) => operation_strategy(feature, language),
        CoreFeature::Ownership(feature) => match feature {
            OwnershipFeature::OnceLeftToRight | OwnershipFeature::OwnedImmutableValue => Direct,
        },
    };
    if language == BuiltInLanguage::JavaScript {
        SupportDecision::Emulated(BuiltInStrategy::CompilerDerivedTypeScript)
    } else if matches!(strategy, Direct) {
        SupportDecision::Native(strategy)
    } else {
        SupportDecision::Emulated(strategy)
    }
}

fn interface_strategy(language: BuiltInLanguage) -> BuiltInStrategy {
    match language {
        BuiltInLanguage::C => BuiltInStrategy::FunctionTable,
        BuiltInLanguage::Rust
        | BuiltInLanguage::TypeScript
        | BuiltInLanguage::JavaScript
        | BuiltInLanguage::Python
        | BuiltInLanguage::Go
        | BuiltInLanguage::Java
        | BuiltInLanguage::Cpp => BuiltInStrategy::Direct,
    }
}

fn operation_strategy(feature: OperationFeature, language: BuiltInLanguage) -> BuiltInStrategy {
    use BuiltInStrategy::{Direct, RuntimeHelper, TaggedUnion};
    match feature {
        OperationFeature::Literal
        | OperationFeature::Local
        | OperationFeature::Constant
        | OperationFeature::SelfValue
        | OperationFeature::ConstructRecord
        | OperationFeature::ConstructList
        | OperationFeature::Field
        | OperationFeature::Call
        | OperationFeature::StaticMethodCall
        | OperationFeature::If
        | OperationFeature::Match
        | OperationFeature::Block => Direct,
        OperationFeature::ConstructEnum
        | OperationFeature::ConstructSome
        | OperationFeature::ConstructNone
        | OperationFeature::ConstructOk
        | OperationFeature::ConstructErr => TaggedUnion,
        OperationFeature::CoerceInterface | OperationFeature::InterfaceCall => {
            interface_strategy(language)
        }
        OperationFeature::Unary(operation) => match operation {
            CoreUnaryIntrinsic::BoolNot
            | CoreUnaryIntrinsic::IntNegChecked
            | CoreUnaryIntrinsic::IntNegWrapping
            | CoreUnaryIntrinsic::IntBitNot
            | CoreUnaryIntrinsic::FloatNeg
            | CoreUnaryIntrinsic::FloatTrunc
            | CoreUnaryIntrinsic::FloatIsNaN
            | CoreUnaryIntrinsic::FloatIsNegativeZero
            | CoreUnaryIntrinsic::FloatAbs
            | CoreUnaryIntrinsic::StringScalarLength
            | CoreUnaryIntrinsic::StringUtf16Length
            | CoreUnaryIntrinsic::StringIsEmpty
            | CoreUnaryIntrinsic::BytesLength
            | CoreUnaryIntrinsic::BytesIsEmpty
            | CoreUnaryIntrinsic::ListLength
            | CoreUnaryIntrinsic::ListIsEmpty
            | CoreUnaryIntrinsic::OptionIsSome
            | CoreUnaryIntrinsic::OptionIsNone
            | CoreUnaryIntrinsic::ResultIsOk
            | CoreUnaryIntrinsic::ResultIsErr
            | CoreUnaryIntrinsic::WidenI32ToI64
            | CoreUnaryIntrinsic::NarrowI64ToI32Checked
            | CoreUnaryIntrinsic::StringToUtf8
            | CoreUnaryIntrinsic::StringFromUtf8Checked => RuntimeHelper,
        },
        OperationFeature::Binary(operation) => match operation {
            CoreBinaryIntrinsic::BoolAnd
            | CoreBinaryIntrinsic::BoolOr
            | CoreBinaryIntrinsic::Equal
            | CoreBinaryIntrinsic::NotEqual
            | CoreBinaryIntrinsic::Less
            | CoreBinaryIntrinsic::LessEqual
            | CoreBinaryIntrinsic::Greater
            | CoreBinaryIntrinsic::GreaterEqual
            | CoreBinaryIntrinsic::IntAddChecked
            | CoreBinaryIntrinsic::IntSubChecked
            | CoreBinaryIntrinsic::IntMulChecked
            | CoreBinaryIntrinsic::IntDivChecked
            | CoreBinaryIntrinsic::IntRemChecked
            | CoreBinaryIntrinsic::IntAddWrapping
            | CoreBinaryIntrinsic::IntSubWrapping
            | CoreBinaryIntrinsic::IntMulWrapping
            | CoreBinaryIntrinsic::IntBitAnd
            | CoreBinaryIntrinsic::IntBitOr
            | CoreBinaryIntrinsic::IntBitXor
            | CoreBinaryIntrinsic::IntShiftLeftChecked
            | CoreBinaryIntrinsic::IntShiftRightChecked
            | CoreBinaryIntrinsic::FloatAdd
            | CoreBinaryIntrinsic::FloatSub
            | CoreBinaryIntrinsic::FloatMul
            | CoreBinaryIntrinsic::FloatDiv
            | CoreBinaryIntrinsic::FloatRemTrunc
            | CoreBinaryIntrinsic::StringConcat
            | CoreBinaryIntrinsic::StringIndexOfLiteral
            | CoreBinaryIntrinsic::StringContains
            | CoreBinaryIntrinsic::StringStartsWith
            | CoreBinaryIntrinsic::StringStripPrefix
            | CoreBinaryIntrinsic::StringEndsWith
            | CoreBinaryIntrinsic::StringTruncateUtf8Bytes
            | CoreBinaryIntrinsic::StringTrimStart
            | CoreBinaryIntrinsic::StringTrimEnd
            | CoreBinaryIntrinsic::BytesConcat
            | CoreBinaryIntrinsic::ListGetChecked
            | CoreBinaryIntrinsic::ListAppend
            | CoreBinaryIntrinsic::ListConcat
            | CoreBinaryIntrinsic::ListContains
            | CoreBinaryIntrinsic::ListIndexOf
            | CoreBinaryIntrinsic::OptionUnwrapOr => RuntimeHelper,
        },
        OperationFeature::Ternary(operation) => match operation {
            CoreTernaryIntrinsic::StringSliceScalars
            | CoreTernaryIntrinsic::StringReplaceAll
            | CoreTernaryIntrinsic::BytesReplaceAll => RuntimeHelper,
        },
        OperationFeature::Variadic(operation) => match operation {
            CoreVariadicIntrinsic::StringReplaceMany => RuntimeHelper,
        },
    }
}

#[cfg(test)]
mod tests {
    use portable_build::{ModuleBuilder, Parameter, Type, Value, Visibility};

    use super::*;

    fn bool_program() -> CoreProgram {
        let mut module = ModuleBuilder::new("bool_capability");
        module.function("identity", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", Type::bool()));
            function.returns(Type::bool());
            function.body(|body| {
                let value = body.local("value");
                body.block([], Some(value))
            });
        });
        let checked = module.finish().unwrap();
        lower_checked(&checked).unwrap()
    }

    fn string_program() -> CoreProgram {
        let mut module = ModuleBuilder::new("string_capability");
        module.function("literal", Visibility::Public, vec![], |function| {
            function.returns(Type::string());
            function.body(|body| {
                let value = body.literal(Value::string("feature"));
                body.block([], Some(value))
            });
        });
        let checked = module.finish().unwrap();
        lower_checked(&checked).unwrap()
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TestStrategy {
        Direct,
        Missing,
    }

    #[derive(Clone)]
    struct SelectiveRegistry {
        target: &'static str,
        rejected: Option<CoreFeature>,
        select_missing: bool,
    }

    impl CapabilityRegistry for SelectiveRegistry {
        type Strategy = TestStrategy;

        fn target(&self) -> TargetId {
            TargetId::parse(self.target).unwrap()
        }

        fn support(&self, usage: &FeatureUse) -> SupportDecision<Self::Strategy> {
            if self.rejected == Some(usage.feature()) {
                SupportDecision::Unsupported(UnsupportedSupport {
                    reason: UnsupportedReason::NotImplemented,
                    detail: "fixture intentionally rejects this feature".to_owned(),
                    option: None,
                })
            } else if self.select_missing {
                SupportDecision::Native(TestStrategy::Missing)
            } else {
                SupportDecision::Native(TestStrategy::Direct)
            }
        }

        fn has_lowering(&self, strategy: &Self::Strategy) -> bool {
            match strategy {
                TestStrategy::Direct => true,
                TestStrategy::Missing => false,
            }
        }
    }

    #[test]
    fn feature_collection_is_minimal_structural_source_located_and_stable() {
        let boolean = collect_core_features(&bool_program());
        assert!(boolean.contains(CoreFeature::Type(TypeFeature::Bool)));
        assert!(!boolean.contains(CoreFeature::Type(TypeFeature::String)));
        assert!(boolean.contains(CoreFeature::Operation(OperationFeature::Local)));
        assert!(boolean.iter().all(|usage| match usage.source() {
            SourceRef::Logical(path) => !path.segments.is_empty(),
            SourceRef::File(span) => !span.file.is_empty(),
        }));
        assert_eq!(boolean, collect_core_features(&bool_program()));
    }

    #[test]
    fn unsupported_feature_isolated_to_used_program_and_requested_target() {
        let registry = SelectiveRegistry {
            target: "test.selective",
            rejected: Some(CoreFeature::Type(TypeFeature::String)),
            select_missing: false,
        };
        assert!(preflight_capabilities(&bool_program(), &registry).is_ok());
        let first = preflight_capabilities(&string_program(), &registry).unwrap_err();
        let second = preflight_capabilities(&string_program(), &registry).unwrap_err();
        assert_eq!(first, second);
        assert!(first.iter().all(|diagnostic| {
            diagnostic.code == DiagnosticCode::UnsupportedCapability
                && diagnostic.target.as_deref() == Some("test.selective")
        }));
    }

    #[test]
    fn matrix_is_atomic_and_reports_only_the_rejecting_target() {
        let registries = [
            SelectiveRegistry {
                target: "test.one",
                rejected: None,
                select_missing: false,
            },
            SelectiveRegistry {
                target: "test.two",
                rejected: Some(CoreFeature::Type(TypeFeature::String)),
                select_missing: false,
            },
            SelectiveRegistry {
                target: "test.three",
                rejected: None,
                select_missing: false,
            },
        ];
        let diagnostics = preflight_capability_matrix(&string_program(), &registries).unwrap_err();
        assert!(!diagnostics.is_empty());
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.target.as_deref() == Some("test.two"))
        );
    }

    #[test]
    fn selected_strategy_must_have_a_registered_lowering() {
        let registry = SelectiveRegistry {
            target: "test.missing",
            rejected: None,
            select_missing: true,
        };
        let diagnostics = preflight_capabilities(&bool_program(), &registry).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.message.contains("unregistered strategy"))
        );
    }

    #[test]
    fn all_eight_builtins_explicitly_support_every_used_feature() {
        let program = string_program();
        let uses = collect_core_features(&program);
        let matrix = preflight_all_builtins(&program).unwrap();
        assert_eq!(matrix.len(), BuiltInLanguage::ALL.len());
        for language in BuiltInLanguage::ALL {
            let selected = &matrix[&language];
            assert_eq!(selected.len(), uses.len());
            assert!(selected.iter().all(|selection| match selection.mode {
                SupportMode::Native | SupportMode::Emulated => true,
            }));
        }
        assert!(
            matrix[&BuiltInLanguage::JavaScript]
                .iter()
                .all(|selection| selection.strategy == BuiltInStrategy::CompilerDerivedTypeScript)
        );
    }

    #[derive(Clone, Debug)]
    struct Options {
        conflict: bool,
    }

    impl TypedTargetOptions for Options {
        fn validate(&self) -> Result<(), Vec<TypedOptionError>> {
            if self.conflict {
                Err(vec![TypedOptionError {
                    reason: UnsupportedReason::ConflictingOptions,
                    option: "interface_mode",
                    detail: "dynamic dispatch conflicts with static-only mode".to_owned(),
                }])
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn typed_options_are_validated_before_the_proof_object_exists() {
        let errors = validate_typed_options(Options { conflict: true }).unwrap_err();
        assert_eq!(errors[0].reason, UnsupportedReason::ConflictingOptions);
        let valid = validate_typed_options(Options { conflict: false }).unwrap();
        assert!(!valid.get().conflict);
    }

    #[test]
    fn invalid_typed_options_stop_before_capability_preflight() {
        struct MustNotRun;

        impl CapabilityRegistry for MustNotRun {
            type Strategy = TestStrategy;

            fn target(&self) -> TargetId {
                TargetId::parse("test.must-not-run").unwrap()
            }

            fn support(&self, _usage: &FeatureUse) -> SupportDecision<Self::Strategy> {
                panic!("capability preflight ran after invalid options")
            }

            fn has_lowering(&self, _strategy: &Self::Strategy) -> bool {
                panic!("strategy lookup ran after invalid options")
            }
        }

        assert!(matches!(
            validate_options_and_preflight(
                &bool_program(),
                &MustNotRun,
                Options { conflict: true },
            ),
            Err(TypedPreflightError::InvalidOptions(_))
        ));
    }
}
