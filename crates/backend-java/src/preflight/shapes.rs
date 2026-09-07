//! Java-specific legality checks for the dynamic compatibility boundary.

use super::*;

pub(super) fn valid_shape(feature: CoreFeature, shape: &FeatureShape) -> bool {
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
            OperationFeature::Local => matches!(shape, FeatureShape::LocalBinding(_)),
            OperationFeature::Binary(
                CoreBinaryIntrinsic::Equal | CoreBinaryIntrinsic::NotEqual,
            ) => {
                matches!(shape, FeatureShape::Equality(_))
            }
            OperationFeature::Literal
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

pub(super) fn fallible_constant_diagnostics(program: &CoreProgram) -> Vec<Diagnostic> {
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
