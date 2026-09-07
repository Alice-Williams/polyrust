//! Support selection and target-specific checks before Java lowering.

mod registration;
mod shapes;

use shapes::{
    erased_conformance_collisions, fallible_constant_diagnostics, java_illegal_shape_diagnostics,
    valid_shape,
};
use std::collections::BTreeMap;

use portable_build::{
    BoolValues, BooleanLogic, BytesOperations, BytesValues, CharValues, CheckedIntegerArithmetic,
    CheckedIntegerShifts, Conditionals, Constants, Enums, Equality, F64Values,
    FloatingPointArithmetic, FloatingPointInspection, Functions, I32Values, I64Values,
    IntegerBitwise, IntegerConversions, Interfaces, ListOperations, ListValues, LocalBindings,
    Loops, Modules, OptionOperations, OptionValues, Ordering, PatternMatching, PortableTests,
    Records, ResultOperations, ResultPropagation, ResultValues, StringConcatenation,
    StringInspection, StringTransformation, Supports, TextValues, TypeAliases, UnitValues,
    Utf8Conversions, WrappingIntegerArithmetic,
};
use portable_codegen::{
    CapabilityRegistry, ControlFeature, CoreFeature, DeclarationFeature, EqualityOperandShape,
    FeatureShape, FeatureUse, InterfaceFeature, OperationFeature, OwnershipFeature,
    SelectedFeature, SupportDecision, SupportMode, TargetCapabilityRegistry, TargetId, TypeFeature,
    UnsupportedReason, UnsupportedSupport, VerifiedCore, collect_core_features,
    preflight_capabilities,
};
use portable_core_ir::{
    CoreBinaryIntrinsic, CoreConstantExpr, CoreConstantExprKind, CoreDeclaration,
    CoreIntrinsicExpr, CoreLocalKind, CoreProgram, CoreRecordId, CoreTernaryIntrinsic, CoreType,
    CoreTypeId, CoreUnaryIntrinsic, CoreVariadicIntrinsic,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, SourceRef, sort_diagnostics};

use crate::{
    ast::JavaIdentifier,
    capabilities::{JavaCapabilitySet, java_capabilities},
};

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JavaLoweringStrategy {
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
    #[cfg(test)]
    pub(crate) fn for_test(core: &CoreProgram) -> Self {
        Self {
            selected: preflight_capabilities(core, &JavaCapabilityRegistry::default())
                .expect("boundary fixture capabilities"),
        }
    }

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

            let registry = JavaCapabilityRegistry::default();
            let (expected_mode, expected_strategy) = match registry.support(expected_use) {
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

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct JavaCapabilityRegistry {
    features: JavaCapabilitySet,
}

impl Default for JavaCapabilityRegistry {
    fn default() -> Self {
        Self::new(java_capabilities())
    }
}

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

        self.confirm_registered_mapping(usage);

        if matches!(
            usage.shape(),
            FeatureShape::Equality(EqualityOperandShape::PayloadFreeEnum)
        ) {
            return SupportDecision::Native(JavaLoweringStrategy::DirectValue);
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
                TypeFeature::Option | TypeFeature::Result => SupportDecision::Emulated(TaggedValue),
                TypeFeature::Enum => SupportDecision::Native(DirectValue),
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
                | OperationFeature::If
                | OperationFeature::Match
                | OperationFeature::Block => SupportDecision::Native(DirectValue),
                OperationFeature::ConstructEnum => match usage.shape() {
                    FeatureShape::Aggregate { field_count: 0 } => {
                        SupportDecision::Native(DirectValue)
                    }
                    FeatureShape::Aggregate { .. } => SupportDecision::Emulated(TaggedValue),
                    _ => unreachable!("Java enum construction shape was validated above"),
                },
                OperationFeature::ConstructSome
                | OperationFeature::ConstructNone
                | OperationFeature::ConstructOk
                | OperationFeature::ConstructErr => SupportDecision::Emulated(TaggedValue),
                OperationFeature::CoerceInterface
                | OperationFeature::StaticMethodCall
                | OperationFeature::InterfaceCall => SupportDecision::Native(InterfaceDispatch),
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

#[cfg(test)]
#[path = "tests/preflight_ownership.rs"]
mod preflight_ownership_tests;

#[cfg(test)]
#[path = "tests/capability_selection.rs"]
mod tests;
