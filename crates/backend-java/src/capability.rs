use std::collections::BTreeMap;

use portable_codegen::{
    CapabilityRegistry, ControlFeature, CoreFeature, DeclarationFeature, FeatureShape, FeatureUse,
    InterfaceFeature, OperationFeature, OwnershipFeature, SelectedFeature, SupportDecision,
    SupportMode, TargetCapabilityRegistry, TargetId, TypeFeature, UnsupportedReason,
    UnsupportedSupport, VerifiedCore, collect_core_features, preflight_capabilities,
};
use portable_core_ir::{
    CoreBinaryIntrinsic, CoreConstantExpr, CoreConstantExprKind, CoreDeclaration,
    CoreIntrinsicExpr, CoreProgram, CoreRecordId, CoreTernaryIntrinsic, CoreType, CoreTypeId,
    CoreUnaryIntrinsic, CoreVariadicIntrinsic,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef, sort_diagnostics};

use crate::ast::JavaIdentifier;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum JavaLoweringStrategy {
    Declaration,
    DirectValue,
    StructuredControl,
    RuntimeHelper,
    TaggedValue,
    InterfaceDispatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaCapabilitySelection {
    selected: Vec<SelectedFeature<JavaLoweringStrategy>>,
}

impl JavaCapabilitySelection {
    pub(crate) fn validate_for(&self, program: &CoreProgram) -> Result<(), Vec<Diagnostic>> {
        let expected = collect_core_features(program);
        let mut diagnostics = Vec::new();

        if self.selected.len() != expected.len() {
            diagnostics.push(java_capability_diagnostic(
                format!(
                    "Java lowering received {} capability decisions for {} exact feature uses",
                    self.selected.len(),
                    expected.len()
                ),
                expected
                    .iter()
                    .next()
                    .map_or_else(java_capability_source, |usage| usage.source().clone()),
            ));
        }

        for (index, expected_use) in expected.iter().enumerate() {
            let Some(actual) = self.selected.get(index) else {
                continue;
            };
            if actual.usage != *expected_use {
                diagnostics.push(java_capability_diagnostic(
                    format!(
                        "Java lowering capability decision {index} names {:?}, but the checked program requires {:?}",
                        actual.usage.feature(),
                        expected_use.feature()
                    ),
                    expected_use.source().clone(),
                ));
                continue;
            }

            let (expected_mode, expected_strategy) = match JavaCapabilityRegistry
                .support(expected_use)
            {
                SupportDecision::Native(strategy) => (SupportMode::Native, strategy),
                SupportDecision::Emulated(strategy) => (SupportMode::Emulated, strategy),
                SupportDecision::Unsupported(value) => {
                    diagnostics.push(java_capability_diagnostic(
                        format!(
                            "Java lowering received a selected capability which preflight now rejects: {}",
                            value.detail
                        ),
                        expected_use.source().clone(),
                    ));
                    continue;
                }
            };
            if actual.mode != expected_mode || actual.strategy != expected_strategy {
                diagnostics.push(java_capability_diagnostic(
                    format!(
                        "Java lowering capability decision for {:?} has mode {:?} and strategy {:?}; expected {:?} and {:?}",
                        expected_use.feature(),
                        actual.mode,
                        actual.strategy,
                        expected_mode,
                        expected_strategy
                    ),
                    expected_use.source().clone(),
                ));
            }
        }

        for unexpected in self.selected.iter().skip(expected.len()) {
            diagnostics.push(java_capability_diagnostic(
                format!(
                    "Java lowering received an unexpected capability decision for {:?}",
                    unexpected.usage.feature()
                ),
                unexpected.usage.source().clone(),
            ));
        }

        sort_diagnostics(&mut diagnostics);
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct JavaCapabilityRegistry;

impl CapabilityRegistry for JavaCapabilityRegistry {
    type Strategy = JavaLoweringStrategy;

    fn target(&self) -> TargetId {
        TargetId::parse("org.polyrust.java").expect("static Java target ID is valid")
    }

    fn support(&self, usage: &FeatureUse) -> SupportDecision<Self::Strategy> {
        if !valid_shape(usage.feature(), usage.shape()) {
            return SupportDecision::Unsupported(UnsupportedSupport {
                reason: UnsupportedReason::UnsupportedShape,
                detail: "feature was collected with a shape outside Java's typed lowering contract"
                    .to_owned(),
                option: None,
            });
        }

        use JavaLoweringStrategy::{
            Declaration, DirectValue, InterfaceDispatch, RuntimeHelper, StructuredControl,
            TaggedValue,
        };
        match usage.feature() {
            CoreFeature::Declaration(feature) => match feature {
                DeclarationFeature::Constant
                | DeclarationFeature::Alias
                | DeclarationFeature::Record
                | DeclarationFeature::Enum
                | DeclarationFeature::Interface
                | DeclarationFeature::Implementation
                | DeclarationFeature::Function
                | DeclarationFeature::Test => SupportDecision::Native(Declaration),
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
                | TypeFeature::Record => SupportDecision::Native(DirectValue),
                TypeFeature::Option | TypeFeature::Result | TypeFeature::Enum => {
                    SupportDecision::Emulated(TaggedValue)
                }
                TypeFeature::Interface => SupportDecision::Native(InterfaceDispatch),
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
                | ControlFeature::ErrPattern => SupportDecision::Native(StructuredControl),
            },
            CoreFeature::Interface(feature) => match feature {
                InterfaceFeature::Declaration
                | InterfaceFeature::Conformance
                | InterfaceFeature::MultipleConformance
                | InterfaceFeature::StaticDispatch
                | InterfaceFeature::DynamicDispatch
                | InterfaceFeature::InterfaceValue => SupportDecision::Native(InterfaceDispatch),
            },
            CoreFeature::Operation(feature) => match feature {
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
                | OperationFeature::Block => SupportDecision::Native(DirectValue),
                OperationFeature::ConstructEnum
                | OperationFeature::ConstructSome
                | OperationFeature::ConstructNone
                | OperationFeature::ConstructOk
                | OperationFeature::ConstructErr => SupportDecision::Emulated(TaggedValue),
                OperationFeature::CoerceInterface | OperationFeature::InterfaceCall => {
                    SupportDecision::Native(InterfaceDispatch)
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
                    | CoreUnaryIntrinsic::StringFromUtf8Checked => {
                        SupportDecision::Emulated(RuntimeHelper)
                    }
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
                    | CoreBinaryIntrinsic::OptionUnwrapOr => {
                        SupportDecision::Emulated(RuntimeHelper)
                    }
                },
                OperationFeature::Ternary(operation) => match operation {
                    CoreTernaryIntrinsic::StringSliceScalars
                    | CoreTernaryIntrinsic::StringReplaceAll
                    | CoreTernaryIntrinsic::BytesReplaceAll => {
                        SupportDecision::Emulated(RuntimeHelper)
                    }
                },
                OperationFeature::Variadic(operation) => match operation {
                    CoreVariadicIntrinsic::StringReplaceMany => {
                        SupportDecision::Emulated(RuntimeHelper)
                    }
                },
            },
            CoreFeature::Ownership(feature) => match feature {
                OwnershipFeature::OnceLeftToRight | OwnershipFeature::OwnedImmutableValue => {
                    SupportDecision::Native(DirectValue)
                }
            },
        }
    }

    fn has_lowering(&self, strategy: &Self::Strategy) -> bool {
        match strategy {
            JavaLoweringStrategy::Declaration
            | JavaLoweringStrategy::DirectValue
            | JavaLoweringStrategy::StructuredControl
            | JavaLoweringStrategy::RuntimeHelper
            | JavaLoweringStrategy::TaggedValue
            | JavaLoweringStrategy::InterfaceDispatch => true,
        }
    }
}

impl TargetCapabilityRegistry<CoreProgram> for JavaCapabilityRegistry {
    type Selection = JavaCapabilitySelection;

    fn preflight(
        &self,
        core: &VerifiedCore<CoreProgram>,
    ) -> Result<Self::Selection, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let selected = match preflight_capabilities(core.value(), self) {
            Ok(selected) => selected,
            Err(mut errors) => {
                diagnostics.append(&mut errors);
                vec![]
            }
        };
        diagnostics.extend(java_illegal_shape_diagnostics(core.value()));
        diagnostics.extend(erased_conformance_collisions(core.value()));
        diagnostics.extend(fallible_constant_diagnostics(core.value()));
        sort_diagnostics(&mut diagnostics);
        if diagnostics.is_empty() {
            Ok(JavaCapabilitySelection { selected })
        } else {
            Err(diagnostics)
        }
    }
}

fn java_capability_source() -> SourceRef {
    SourceRef::logical(["java-capability-selection"])
}

fn java_capability_diagnostic(message: impl Into<String>, source: SourceRef) -> Diagnostic {
    let mut diagnostic = Diagnostic::error(DiagnosticCode::UnsupportedCapability, message, source);
    diagnostic.target = Some("org.polyrust.java".to_owned());
    diagnostic
}

fn java_illegal_shape_diagnostics(program: &CoreProgram) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for record in program.records() {
        collect_restricted_record_components(program, &record.fields, &mut diagnostics);
    }
    for variant in program.variants() {
        collect_restricted_record_components(program, &variant.fields, &mut diagnostics);
    }

    for declaration in &program.module().declarations {
        let CoreDeclaration::Interface(interface_id) = *declaration else {
            continue;
        };
        let Some(interface) = program.interface(interface_id) else {
            continue;
        };
        if !program
            .implementations()
            .iter()
            .any(|implementation| implementation.interface == interface_id)
        {
            diagnostics.push(java_capability_diagnostic(
                format!(
                    "Java immutable interface {} has no generated implementation to seal",
                    interface.header.name
                ),
                interface.header.source.clone(),
            ));
        }
        for method_id in &interface.methods {
            let Some(method) = program.interface_method(*method_id) else {
                continue;
            };
            let name = JavaIdentifier::from_portable(&method.header.name);
            let parameters = method
                .parameters
                .iter()
                .map(|parameter| erased_type(program, parameter.ty))
                .collect::<Vec<_>>();
            if is_final_object_method(name.as_str(), &parameters) {
                diagnostics.push(java_capability_diagnostic(
                    format!(
                        "Java interface method {}({}) collides with a final java.lang.Object method",
                        name.as_str(),
                        parameters.join(", ")
                    ),
                    method.header.source.clone(),
                ));
            }
        }
    }

    for implementation in program.implementations() {
        let Some(record) = program.record(implementation.record) else {
            continue;
        };
        let accessors = record
            .fields
            .iter()
            .filter_map(|field| program.field(*field))
            .map(|field| {
                JavaIdentifier::from_portable(&field.header.name)
                    .as_str()
                    .to_owned()
            })
            .collect::<Vec<_>>();
        for method_id in &implementation.methods {
            let Some(method) = program.implementation_method(*method_id) else {
                continue;
            };
            let Some(interface_method) = program.interface_method(method.interface_method) else {
                continue;
            };
            let name = JavaIdentifier::from_portable(&interface_method.header.name);
            if interface_method.parameters.is_empty()
                && accessors.iter().any(|accessor| accessor == name.as_str())
            {
                diagnostics.push(java_capability_diagnostic(
                    format!(
                        "Java record accessor {}() collides with an interface implementation method after erasure",
                        name.as_str()
                    ),
                    method.header.source.clone(),
                ));
            }
        }
    }

    diagnostics
}

fn collect_restricted_record_components(
    program: &CoreProgram,
    fields: &[portable_core_ir::CoreFieldId],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for field_id in fields {
        let Some(field) = program.field(*field_id) else {
            continue;
        };
        let name = JavaIdentifier::from_portable(&field.header.name);
        if is_restricted_record_component(name.as_str()) {
            diagnostics.push(java_capability_diagnostic(
                format!(
                    "Java record component name {:?} is forbidden by Java 21",
                    name.as_str()
                ),
                field.header.source.clone(),
            ));
        }
    }
}

fn is_restricted_record_component(name: &str) -> bool {
    matches!(
        name,
        "clone"
            | "finalize"
            | "getClass"
            | "hashCode"
            | "notify"
            | "notifyAll"
            | "toString"
            | "wait"
    )
}

fn is_final_object_method(name: &str, parameters: &[String]) -> bool {
    match name {
        "getClass" | "notify" | "notifyAll" => parameters.is_empty(),
        "wait" => parameters.is_empty() || parameters == ["long"] || parameters == ["long", "int"],
        _ => false,
    }
}

fn valid_shape(feature: CoreFeature, shape: &FeatureShape) -> bool {
    match feature {
        CoreFeature::Declaration(feature) => match feature {
            DeclarationFeature::Record | DeclarationFeature::Enum => {
                matches!(shape, FeatureShape::Aggregate { .. })
            }
            DeclarationFeature::Function => matches!(shape, FeatureShape::Callable { .. }),
            DeclarationFeature::Constant
            | DeclarationFeature::Alias
            | DeclarationFeature::Interface
            | DeclarationFeature::Implementation
            | DeclarationFeature::Test => matches!(shape, FeatureShape::Unit),
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
            | TypeFeature::Option
            | TypeFeature::Result
            | TypeFeature::Record
            | TypeFeature::Enum
            | TypeFeature::Interface => matches!(shape, FeatureShape::Unit),
        },
        CoreFeature::Control(feature) => match feature {
            ControlFeature::Match => matches!(shape, FeatureShape::Aggregate { .. }),
            ControlFeature::Block
            | ControlFeature::Let
            | ControlFeature::ForEach
            | ControlFeature::Return
            | ControlFeature::Evaluate
            | ControlFeature::If
            | ControlFeature::WildcardPattern
            | ControlFeature::BoolPattern
            | ControlFeature::EnumPattern
            | ControlFeature::NonePattern
            | ControlFeature::SomePattern
            | ControlFeature::OkPattern
            | ControlFeature::ErrPattern => matches!(shape, FeatureShape::Unit),
        },
        CoreFeature::Interface(feature) => match feature {
            InterfaceFeature::Declaration => matches!(shape, FeatureShape::Interface(_)),
            InterfaceFeature::Conformance
            | InterfaceFeature::MultipleConformance
            | InterfaceFeature::StaticDispatch
            | InterfaceFeature::DynamicDispatch
            | InterfaceFeature::InterfaceValue => matches!(shape, FeatureShape::Unit),
        },
        CoreFeature::Operation(feature) => match feature {
            OperationFeature::ConstructRecord
            | OperationFeature::ConstructEnum
            | OperationFeature::ConstructList
            | OperationFeature::Match => matches!(shape, FeatureShape::Aggregate { .. }),
            OperationFeature::Call
            | OperationFeature::StaticMethodCall
            | OperationFeature::InterfaceCall => matches!(shape, FeatureShape::Callable { .. }),
            OperationFeature::Variadic(_) => matches!(shape, FeatureShape::Variadic { .. }),
            OperationFeature::Literal
            | OperationFeature::Local
            | OperationFeature::Constant
            | OperationFeature::SelfValue
            | OperationFeature::ConstructSome
            | OperationFeature::ConstructNone
            | OperationFeature::ConstructOk
            | OperationFeature::ConstructErr
            | OperationFeature::CoerceInterface
            | OperationFeature::Field
            | OperationFeature::Unary(_)
            | OperationFeature::Binary(_)
            | OperationFeature::Ternary(_)
            | OperationFeature::If
            | OperationFeature::Block => matches!(shape, FeatureShape::Unit),
        },
        CoreFeature::Ownership(feature) => match feature {
            OwnershipFeature::OnceLeftToRight | OwnershipFeature::OwnedImmutableValue => {
                matches!(shape, FeatureShape::Unit)
            }
        },
    }
}

fn erased_conformance_collisions(program: &CoreProgram) -> Vec<Diagnostic> {
    let mut methods = BTreeMap::<(CoreRecordId, String, Vec<String>), _>::new();
    let mut diagnostics = Vec::new();
    for implementation in program.implementations() {
        for method_id in &implementation.methods {
            let Some(method) = program.implementation_method(*method_id) else {
                continue;
            };
            let Some(interface_method) = program.interface_method(method.interface_method) else {
                continue;
            };
            let key = (
                implementation.record,
                JavaIdentifier::from_portable(&interface_method.header.name)
                    .as_str()
                    .to_owned(),
                interface_method
                    .parameters
                    .iter()
                    .map(|parameter| erased_type(program, parameter.ty))
                    .collect::<Vec<_>>(),
            );
            if methods
                .insert(key.clone(), method.header.source.clone())
                .is_some()
            {
                let record_name = program
                    .record(implementation.record)
                    .map_or("<missing>", |record| record.header.name.as_str());
                let mut diagnostic = Diagnostic::error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "target org.polyrust.java cannot preserve distinct interface witnesses for record {record_name:?}: Java-erased method {}({}) collides",
                        key.1,
                        key.2.join(", ")
                    ),
                    method.header.source.clone(),
                );
                diagnostic.target = Some("org.polyrust.java".to_owned());
                diagnostics.push(diagnostic);
            }
        }
    }
    diagnostics
}

fn fallible_constant_diagnostics(program: &CoreProgram) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for constant in program.constants() {
        collect_fallible_constants(&constant.value, &mut diagnostics);
    }
    diagnostics
}

fn collect_fallible_constants(value: &CoreConstantExpr, diagnostics: &mut Vec<Diagnostic>) {
    match &value.kind {
        CoreConstantExprKind::Intrinsic(intrinsic) => {
            if constant_intrinsic_is_fallible(intrinsic) {
                let mut diagnostic = Diagnostic::error(
                    DiagnosticCode::UnsupportedCapability,
                    "target org.polyrust.java cannot preserve a fallible intrinsic in a static constant initializer",
                    value.source.clone(),
                );
                diagnostic.target = Some("org.polyrust.java".to_owned());
                diagnostics.push(diagnostic);
            }
            match &**intrinsic {
                CoreIntrinsicExpr::Unary { operand, .. } => {
                    collect_fallible_constants(operand, diagnostics);
                }
                CoreIntrinsicExpr::Binary { left, right, .. } => {
                    collect_fallible_constants(left, diagnostics);
                    collect_fallible_constants(right, diagnostics);
                }
                CoreIntrinsicExpr::Ternary {
                    first,
                    second,
                    third,
                    ..
                } => {
                    collect_fallible_constants(first, diagnostics);
                    collect_fallible_constants(second, diagnostics);
                    collect_fallible_constants(third, diagnostics);
                }
                CoreIntrinsicExpr::Variadic { arguments, .. } => {
                    for child in arguments {
                        collect_fallible_constants(child, diagnostics);
                    }
                }
            }
        }
        CoreConstantExprKind::Record { fields, .. } | CoreConstantExprKind::Enum { fields, .. } => {
            for field in fields {
                collect_fallible_constants(&field.value, diagnostics);
            }
        }
        CoreConstantExprKind::Some(child)
        | CoreConstantExprKind::Ok { value: child, .. }
        | CoreConstantExprKind::Err { value: child, .. } => {
            collect_fallible_constants(child, diagnostics)
        }
        CoreConstantExprKind::List { elements, .. } => {
            for child in elements {
                collect_fallible_constants(child, diagnostics);
            }
        }
        CoreConstantExprKind::Literal(_)
        | CoreConstantExprKind::Constant(_)
        | CoreConstantExprKind::None { .. } => {}
    }
}

fn constant_intrinsic_is_fallible(intrinsic: &CoreIntrinsicExpr<CoreConstantExpr>) -> bool {
    match intrinsic {
        CoreIntrinsicExpr::Unary { operation, .. } => matches!(
            operation,
            CoreUnaryIntrinsic::IntNegChecked
                | CoreUnaryIntrinsic::StringScalarLength
                | CoreUnaryIntrinsic::NarrowI64ToI32Checked
                | CoreUnaryIntrinsic::StringFromUtf8Checked
        ),
        CoreIntrinsicExpr::Binary { operation, .. } => matches!(
            operation,
            CoreBinaryIntrinsic::IntAddChecked
                | CoreBinaryIntrinsic::IntSubChecked
                | CoreBinaryIntrinsic::IntMulChecked
                | CoreBinaryIntrinsic::IntDivChecked
                | CoreBinaryIntrinsic::IntRemChecked
                | CoreBinaryIntrinsic::IntShiftLeftChecked
                | CoreBinaryIntrinsic::IntShiftRightChecked
                | CoreBinaryIntrinsic::ListGetChecked
        ),
        CoreIntrinsicExpr::Ternary { operation, .. } => match operation {
            CoreTernaryIntrinsic::StringSliceScalars
            | CoreTernaryIntrinsic::StringReplaceAll
            | CoreTernaryIntrinsic::BytesReplaceAll => false,
        },
        CoreIntrinsicExpr::Variadic { operation, .. } => match operation {
            CoreVariadicIntrinsic::StringReplaceMany => false,
        },
    }
}

fn erased_type(program: &CoreProgram, ty: CoreTypeId) -> String {
    match program.types().get(ty) {
        Some(CoreType::Unit) => "Runtime.Unit".to_owned(),
        Some(CoreType::Bool) => "boolean".to_owned(),
        Some(CoreType::I32) => "int".to_owned(),
        Some(CoreType::I64) => "long".to_owned(),
        Some(CoreType::F64) => "double".to_owned(),
        Some(CoreType::Char) => "Runtime.Scalar".to_owned(),
        Some(CoreType::String) => "String".to_owned(),
        Some(CoreType::Bytes) => "Runtime.Bytes".to_owned(),
        Some(CoreType::List(_)) => "List".to_owned(),
        Some(CoreType::Option(_)) => "Runtime.PolyOption".to_owned(),
        Some(CoreType::Result { .. }) => "Runtime.PolyValueResult".to_owned(),
        Some(CoreType::Record(id)) => program
            .record(*id)
            .map(|record| {
                JavaIdentifier::from_portable(&record.header.name)
                    .as_str()
                    .to_owned()
            })
            .unwrap_or_else(|| "<missing-record>".to_owned()),
        Some(CoreType::Enum(id)) => program
            .enumeration(*id)
            .map(|item| {
                JavaIdentifier::from_portable(&item.header.name)
                    .as_str()
                    .to_owned()
            })
            .unwrap_or_else(|| "<missing-enum>".to_owned()),
        Some(CoreType::Interface(id)) => program
            .interface(*id)
            .map(|item| {
                JavaIdentifier::from_portable(&item.header.name)
                    .as_str()
                    .to_owned()
            })
            .unwrap_or_else(|| "<missing-interface>".to_owned()),
        None => "<missing-type>".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use portable_build::{ModuleBuilder, Parameter, Type, Value, Visibility};
    use portable_codegen::{
        Backend, BackendOptions, OutputContents, TypedCompiler, TypedGenerationError,
        TypedPipelineStage,
    };
    use portable_core_ir::lower_checked;

    use super::*;

    fn preflight_diagnostics(checked: &portable_check::v0::CheckedProgram) -> Vec<Diagnostic> {
        match crate::JavaBackend::compiler()
            .compile_checked(checked, &BackendOptions::default())
            .expect_err("Java-illegal shape must fail")
        {
            TypedGenerationError::Phase {
                stage: TypedPipelineStage::CapabilityPreflight,
                diagnostics,
            } => diagnostics,
            other => panic!("unexpected generation error: {other:?}"),
        }
    }

    #[test]
    fn target_id_matches_java_backend() {
        assert_eq!(
            JavaCapabilityRegistry.target().as_str(),
            "org.polyrust.java"
        );
    }

    #[test]
    fn empty_checked_program_selects_every_collected_feature() {
        let mut module = ModuleBuilder::new("capability_java");
        module.function("identity", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", Type::bool()));
            function.returns(Type::bool());
            function.body(|body| {
                let value = body.local("value");
                body.block([], Some(value))
            });
        });
        let checked = module.finish().unwrap();
        let core = lower_checked(&checked).unwrap();
        let selected = preflight_capabilities(&core, &JavaCapabilityRegistry).unwrap();
        assert_eq!(
            selected.len(),
            portable_codegen::collect_core_features(&core).len()
        );
    }

    #[test]
    fn exact_selection_rejects_feature_and_strategy_permutations() {
        let mut module = ModuleBuilder::new("capability_selection");
        module.function("identity", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", Type::bool()));
            function.returns(Type::bool());
            function.body(|body| {
                let value = body.local("value");
                body.block([], Some(value))
            });
        });
        let checked = module.finish().unwrap();
        let core = lower_checked(&checked).unwrap();
        let selected = preflight_capabilities(&core, &JavaCapabilityRegistry).unwrap();

        let mut permuted_usage = JavaCapabilitySelection {
            selected: selected.clone(),
        };
        permuted_usage.selected.swap(0, 1);
        assert!(permuted_usage.validate_for(&core).is_err());

        let mut mismatched_strategy = JavaCapabilitySelection { selected };
        let declaration = mismatched_strategy
            .selected
            .iter()
            .position(|value| value.strategy == JavaLoweringStrategy::Declaration)
            .expect("declaration strategy");
        let direct = mismatched_strategy
            .selected
            .iter()
            .position(|value| value.strategy == JavaLoweringStrategy::DirectValue)
            .expect("direct strategy");
        let declaration_strategy = mismatched_strategy.selected[declaration].strategy;
        mismatched_strategy.selected[declaration].strategy =
            mismatched_strategy.selected[direct].strategy;
        mismatched_strategy.selected[direct].strategy = declaration_strategy;
        assert!(mismatched_strategy.validate_for(&core).is_err());
    }

    #[test]
    fn record_restricted_component_stops_at_preflight() {
        let mut module = ModuleBuilder::new("java_record_restricted");
        module.record("Bad", Visibility::Public, vec![], |record| {
            record.field("hashCode", Type::i32(), vec![]);
        });
        let checked = module.finish().unwrap();
        let diagnostics = preflight_diagnostics(&checked);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .message
                .contains("record component name \"hashCode\"")
        );
        assert_eq!(diagnostics[0].target.as_deref(), Some("org.polyrust.java"));
    }

    #[test]
    fn final_object_interface_method_stops_at_preflight() {
        let mut module = ModuleBuilder::new("java_object_final");
        let (interface, method) =
            module.interface("Bad", Visibility::Public, vec![], |interface| {
                interface.method("getClass", vec![], vec![], Some(Type::bool()))
            });
        let (record, ()) = module.record("Value", Visibility::Public, vec![], |_| {});
        module.implementation(
            "ValueBad",
            Visibility::Package,
            vec![],
            interface,
            record,
            |implementation| {
                implementation.method("getClass", method, vec![], |method| {
                    method.returns(Type::bool());
                    method.body(|body| {
                        let value = body.literal(Value::bool(true));
                        body.block([], Some(value))
                    });
                });
            },
        );
        let checked = module.finish().unwrap();
        let diagnostics = preflight_diagnostics(&checked);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .message
                .contains("final java.lang.Object method")
        );
        assert_eq!(diagnostics[0].target.as_deref(), Some("org.polyrust.java"));
    }

    #[test]
    fn record_accessor_implementation_method_stops_at_preflight() {
        let mut module = ModuleBuilder::new("java_accessor_collision");
        let (interface, method) =
            module.interface("Readable", Visibility::Public, vec![], |interface| {
                interface.method("read", vec![], vec![], Some(Type::bool()))
            });
        let (record, ()) = module.record("Value", Visibility::Public, vec![], |record| {
            record.field("read", Type::bool(), vec![]);
        });
        module.implementation(
            "ValueReadable",
            Visibility::Package,
            vec![],
            interface,
            record,
            |implementation| {
                implementation.method("read", method, vec![], |method| {
                    method.returns(Type::bool());
                    method.body(|body| {
                        let value = body.literal(Value::bool(true));
                        body.block([], Some(value))
                    });
                });
            },
        );
        let checked = module.finish().unwrap();
        let diagnostics = preflight_diagnostics(&checked);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .message
                .contains("record accessor read() collides")
        );
        assert_eq!(diagnostics[0].target.as_deref(), Some("org.polyrust.java"));
    }

    #[test]
    fn nested_fallible_constant_intrinsic_stops_at_preflight() {
        let mut module = ModuleBuilder::new("java_fallible_constant");
        module.constant(
            "INVALID",
            Visibility::Public,
            vec![],
            Type::bool(),
            |body| {
                let one = body.constant_literal(Value::i32(1));
                let two = body.constant_literal(Value::i32(2));
                let checked =
                    body.constant_intrinsic(portable_build::Operation::IntAddChecked, [one, two]);
                let three = body.constant_literal(Value::i32(3));
                body.constant_intrinsic(portable_build::Operation::Equal, [checked, three])
            },
        );
        let checked = module.finish().unwrap();
        let diagnostics = preflight_diagnostics(&checked);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .message
                .contains("fallible intrinsic in a static constant initializer")
        );
        assert_eq!(diagnostics[0].target.as_deref(), Some("org.polyrust.java"));
    }

    #[test]
    fn minimal_program_does_not_pull_optional_runtime_helpers() {
        let mut module = ModuleBuilder::new("java_minimal_helpers");
        module.constant("FLAG", Visibility::Public, vec![], Type::bool(), |body| {
            body.constant_literal(Value::bool(true))
        });
        let checked = module.finish().unwrap();
        let manifest = crate::JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let runtime = match manifest
            .file("src/main/java/org/polyrust/generated/Runtime.java")
            .expect("runtime file")
            .contents()
        {
            OutputContents::Text(value) => value,
            OutputContents::Bytes(_) => panic!("Java runtime must be text"),
        };
        let conformance = match manifest
            .file("src/test/java/org/polyrust/generated/ConformanceTest.java")
            .expect("conformance file")
            .contents()
        {
            OutputContents::Text(value) => value,
            OutputContents::Bytes(_) => panic!("Java conformance file must be text"),
        };
        assert!(!runtime.contains("import "));
        for absent in [
            "PolyOption",
            "PolyResult",
            "checkedAdd",
            "requireScalarString",
            "appendList",
            "class Bytes",
        ] {
            assert!(!runtime.contains(absent), "unexpected helper {absent}");
        }
        assert!(!conformance.contains("Runtime."));
    }
}
