//! Java lowering: constant dependencies.

use portable_core_ir::{CoreConstantExpr, CoreConstantExprKind, CoreConstantId, CoreIntrinsicExpr};
use std::collections::BTreeSet;

pub(super) fn collect_constant_dependencies(
    value: &CoreConstantExpr,
    dependencies: &mut BTreeSet<CoreConstantId>,
) {
    match &value.kind {
        CoreConstantExprKind::Constant(id) => {
            dependencies.insert(*id);
        }
        CoreConstantExprKind::Record { fields, .. } | CoreConstantExprKind::Enum { fields, .. } => {
            for field in fields {
                collect_constant_dependencies(&field.value, dependencies);
            }
        }
        CoreConstantExprKind::Some(value)
        | CoreConstantExprKind::Ok { value, .. }
        | CoreConstantExprKind::Err { value, .. } => {
            collect_constant_dependencies(value, dependencies);
        }
        CoreConstantExprKind::List { elements, .. } => {
            for element in elements {
                collect_constant_dependencies(element, dependencies);
            }
        }
        CoreConstantExprKind::Intrinsic(value) => match value.as_ref() {
            CoreIntrinsicExpr::Unary { operand, .. } => {
                collect_constant_dependencies(operand, dependencies);
            }
            CoreIntrinsicExpr::Binary { left, right, .. } => {
                collect_constant_dependencies(left, dependencies);
                collect_constant_dependencies(right, dependencies);
            }
            CoreIntrinsicExpr::Ternary {
                first,
                second,
                third,
                ..
            } => {
                collect_constant_dependencies(first, dependencies);
                collect_constant_dependencies(second, dependencies);
                collect_constant_dependencies(third, dependencies);
            }
            CoreIntrinsicExpr::Variadic { arguments, .. } => {
                for argument in arguments {
                    collect_constant_dependencies(argument, dependencies);
                }
            }
        },
        CoreConstantExprKind::Literal(_) | CoreConstantExprKind::None { .. } => {}
    }
}
