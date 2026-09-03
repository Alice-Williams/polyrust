use std::collections::BTreeMap;

use portable_codegen::{
    CapabilityRegistry, ControlFeature, CoreFeature, DeclarationFeature, FeatureShape, FeatureUse,
    InterfaceFeature, OperationFeature, OwnershipFeature, SelectedFeature, SupportDecision,
    TargetCapabilityRegistry, TargetId, TypeFeature, UnsupportedReason, UnsupportedSupport,
    VerifiedCore, preflight_capabilities,
};
use portable_core_ir::{
    CoreBinaryIntrinsic, CoreConstantExpr, CoreConstantExprKind, CoreIntrinsicExpr, CoreProgram,
    CoreRecordId, CoreTernaryIntrinsic, CoreType, CoreTypeId, CoreUnaryIntrinsic,
    CoreVariadicIntrinsic,
};
use portable_diagnostics::{Diagnostic, DiagnosticCode, sort_diagnostics};

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
    pub(crate) fn selected(&self) -> &[SelectedFeature<JavaLoweringStrategy>] {
        &self.selected
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
    use portable_build::{ModuleBuilder, Parameter, Type, Visibility};
    use portable_core_ir::lower_checked;

    use super::*;

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
}
