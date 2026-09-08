//! Check the owned native assertions without replaying the test lowerer.
use super::super::*;
pub(super) fn asserted<'a>(
    statement: &'a JavaStmt,
    failure_when_true: bool,
    expected_message: &str,
) -> Option<&'a JavaExpr> {
    let JavaStmt::If {
        condition,
        then_block,
        else_block: None,
    } = statement
    else {
        return None;
    };
    if condition.ty != JavaType::primitive(JavaPrimitive::Boolean)
        || !matches!(
            then_block.statements.as_slice(),
            [JavaStmt::ThrowAssertion(message)] if message == &string_literal(expected_message)
        )
    {
        return None;
    }
    if failure_when_true {
        Some(condition)
    } else {
        match &condition.kind {
            JavaExprKind::Unary {
                operator: JavaUnaryOperator::Not,
                operand,
            } if operand.ty == JavaType::primitive(JavaPrimitive::Boolean) => Some(operand),
            _ => None,
        }
    }
}
fn member<'a>(
    value: &'a JavaExpr,
    expected: JavaRuntimeMember,
    result: &JavaType,
) -> Option<&'a JavaExpr> {
    if &value.ty != result {
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
        } if *actual == expected && arguments.is_empty() => Some(receiver),
        _ => None,
    }
}
pub(super) fn verify(i: &JavaPortableTestCaseInput, actual: &[JavaStmt]) -> bool {
    let [
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty,
            name,
            value: Some(_),
        },
        status,
        equality,
    ] = actual
    else {
        return false;
    };
    if ty != &i.actual.ty || name != &identifier(&format!("actual{}", i.index)) {
        return false;
    }
    let local = JavaExpr::local(ty.clone(), name.clone());
    let (failure_when_true, helper, expected, status_message, equality_message) = match &i.expected
    {
        JavaPortableTestExpectation::Value(expected) => (
            false,
            JavaRuntimeCallable::DeepEqual,
            expected,
            "unexpectedly failed",
            "value mismatch",
        ),
        JavaPortableTestExpectation::Error(expected) => (
            true,
            JavaRuntimeCallable::SemanticEqual,
            expected,
            "unexpectedly succeeded",
            "error code mismatch",
        ),
    };
    let prefix = format!("portable test {} ({})", i.index, i.name);
    let Some(status) = asserted(
        status,
        failure_when_true,
        &format!("{prefix} {status_message}"),
    ) else {
        return false;
    };
    if member(
        status,
        JavaRuntimeMember::ResultOk,
        &JavaType::primitive(JavaPrimitive::Boolean),
    ) != Some(&local)
    {
        return false;
    }
    let Some(equality) = asserted(equality, false, &format!("{prefix} {equality_message}")) else {
        return false;
    };
    let JavaExprKind::Call {
        callable: JavaCallableRef::Runtime { callable, .. },
        receiver: None,
        arguments,
    } = &equality.kind
    else {
        return false;
    };
    let [value, expected_actual] = arguments.as_slice() else {
        return false;
    };
    if *callable != helper || expected_actual != expected {
        return false;
    }
    match &i.expected {
        JavaPortableTestExpectation::Value(expected) => {
            member(value, JavaRuntimeMember::ResultValue, &expected.ty) == Some(&local)
        }
        JavaPortableTestExpectation::Error(_) => {
            member(
                value,
                JavaRuntimeMember::ErrorCode,
                &JavaType::known(JavaKnownType::String),
            )
            .and_then(|error| {
                member(
                    error,
                    JavaRuntimeMember::ResultError,
                    &JavaType::known(JavaKnownType::RuntimeError),
                )
            }) == Some(&local)
        }
    }
}
