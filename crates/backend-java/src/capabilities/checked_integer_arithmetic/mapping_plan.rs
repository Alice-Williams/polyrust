//! Mapping-owned exhaustive input plans; operands remain opaque holes.
use super::JavaCheckedIntegerArithmeticInput;
use crate::capabilities::support::plans::intrinsics::{
    JavaIntrinsicPlan as PlanType, JavaIntrinsicPlanKind as K,
};
use crate::dialect::JavaRuntimeCallable;

pub type Plan = PlanType;
pub(super) fn select(
    input: &JavaCheckedIntegerArithmeticInput,
) -> Result<Plan, Vec<portable_diagnostics::Diagnostic>> {
    Ok(match input {
        JavaCheckedIntegerArithmeticInput::Neg {
            operand, result, ..
        } => {
            let callable = match operand.ty {
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Int) => {
                    JavaRuntimeCallable::CheckedNegI32
                }
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Long) => {
                    JavaRuntimeCallable::CheckedNegI64
                }
                _ => {
                    return Err(vec![crate::lower::diagnostic(
                        "checked integer plan requires int or long",
                    )]);
                }
            };
            Plan::new(K::FallibleRuntimeCall(callable, 1), result)
        }
        JavaCheckedIntegerArithmeticInput::Add { left, result, .. } => {
            let callable = match left.ty {
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Int) => {
                    JavaRuntimeCallable::CheckedAddI32
                }
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Long) => {
                    JavaRuntimeCallable::CheckedAddI64
                }
                _ => {
                    return Err(vec![crate::lower::diagnostic(
                        "checked integer plan requires int or long",
                    )]);
                }
            };
            Plan::new(K::FallibleRuntimeCall(callable, 2), result)
        }
        JavaCheckedIntegerArithmeticInput::Subtract { left, result, .. } => {
            let callable = match left.ty {
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Int) => {
                    JavaRuntimeCallable::CheckedSubI32
                }
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Long) => {
                    JavaRuntimeCallable::CheckedSubI64
                }
                _ => {
                    return Err(vec![crate::lower::diagnostic(
                        "checked integer plan requires int or long",
                    )]);
                }
            };
            Plan::new(K::FallibleRuntimeCall(callable, 2), result)
        }
        JavaCheckedIntegerArithmeticInput::Multiply { left, result, .. } => {
            let callable = match left.ty {
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Int) => {
                    JavaRuntimeCallable::CheckedMulI32
                }
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Long) => {
                    JavaRuntimeCallable::CheckedMulI64
                }
                _ => {
                    return Err(vec![crate::lower::diagnostic(
                        "checked integer plan requires int or long",
                    )]);
                }
            };
            Plan::new(K::FallibleRuntimeCall(callable, 2), result)
        }
        JavaCheckedIntegerArithmeticInput::Divide { left, result, .. } => {
            let callable = match left.ty {
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Int) => {
                    JavaRuntimeCallable::CheckedDivI32
                }
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Long) => {
                    JavaRuntimeCallable::CheckedDivI64
                }
                _ => {
                    return Err(vec![crate::lower::diagnostic(
                        "checked integer plan requires int or long",
                    )]);
                }
            };
            Plan::new(K::FallibleRuntimeCall(callable, 2), result)
        }
        JavaCheckedIntegerArithmeticInput::Remainder { left, result, .. } => {
            let callable = match left.ty {
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Int) => {
                    JavaRuntimeCallable::CheckedRemI32
                }
                crate::ast::JavaType::Primitive(crate::ast::JavaPrimitive::Long) => {
                    JavaRuntimeCallable::CheckedRemI64
                }
                _ => {
                    return Err(vec![crate::lower::diagnostic(
                        "checked integer plan requires int or long",
                    )]);
                }
            };
            Plan::new(K::FallibleRuntimeCall(callable, 2), result)
        }
    })
}
