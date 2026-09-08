//! Mapping-owned structural output certificate.
use super::*;
use crate::ast::{JavaCallableRef, JavaExprKind, JavaUnaryOperator};
use crate::capabilities::support::{java_input_plan, plans::JavaRepresentation as R};
java_input_plan!(JavaResultPropagationInput, JavaResultPropagationPlan);
fn representation(_: &JavaResultPropagationInput) -> R {
    R::StructuredControl
}
fn runtime_member<'a>(
    value: &'a JavaExpr,
    member: JavaRuntimeMember,
    ty: &JavaType,
) -> Option<&'a JavaExpr> {
    if &value.ty != ty {
        return None;
    }
    match &value.kind {
        JavaExprKind::Call {
            callable:
                JavaCallableRef::Member {
                    origin: JavaMemberOrigin::Runtime(actual),
                    ..
                },
            receiver: Some(receiver),
            arguments,
        } if *actual == member && arguments.is_empty() => Some(receiver),
        _ => None,
    }
}
fn verify(i: &JavaResultPropagationInput, output: &JavaResultPropagationPlan) -> bool {
    let Some(owned) = output.statements.strip_prefix(i.prefix.as_slice()) else {
        return false;
    };
    let [
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty,
            name,
            value: Some(_),
        },
        JavaStmt::If {
            condition,
            then_block,
            else_block: None,
        },
    ] = owned
    else {
        return false;
    };
    if ty != &i.call.ty || name != &i.result_name {
        return false;
    }
    let local = JavaExpr::local(i.call.ty.clone(), i.result_name.clone());
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let JavaExprKind::Unary {
        operator: JavaUnaryOperator::Not,
        operand,
    } = &condition.kind
    else {
        return false;
    };
    if condition.ty != boolean
        || runtime_member(operand, JavaRuntimeMember::ResultOk, &boolean) != Some(&local)
    {
        return false;
    }
    let [JavaStmt::Return(Some(failure))] = then_block.statements.as_slice() else {
        return false;
    };
    if failure.ty != i.callable_result_type {
        return false;
    }
    let JavaExprKind::Call {
        callable:
            JavaCallableRef::Runtime {
                callable: JavaRuntimeCallable::Fail,
                ..
            },
        receiver: None,
        arguments,
    } = &failure.kind
    else {
        return false;
    };
    let [code, message] = arguments.as_slice() else {
        return false;
    };
    let string = JavaType::known(JavaKnownType::String);
    let error_type = JavaType::known(JavaKnownType::RuntimeError);
    for (value, member) in [
        (code, JavaRuntimeMember::ErrorCode),
        (message, JavaRuntimeMember::ErrorMessage),
    ] {
        let Some(error) = runtime_member(value, member, &string) else {
            return false;
        };
        if runtime_member(error, JavaRuntimeMember::ResultError, &error_type) != Some(&local) {
            return false;
        }
    }
    runtime_member(&output.value, JavaRuntimeMember::ResultValue, &i.value_type) == Some(&local)
}
