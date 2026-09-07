//! Typed runtime construction: call builders.
use super::declaration_builders::identifier;

use super::expression_builders::string_literal;
use crate::ast::{
    JavaCallableRef, JavaConstructorRef, JavaExpr, JavaExprKind, JavaMemberOrigin,
    JavaMethodSignature, JavaPrecedence, JavaRuntimeMember, JavaType, JavaValueRef,
};
use crate::dialect::{
    JavaKnownCallable, JavaKnownConstructor, JavaKnownField, JavaKnownMethod, JavaRuntimeCallable,
};

pub(super) fn known_call(callable: JavaKnownCallable, arguments: Vec<JavaExpr>) -> JavaExpr {
    let expected = callable.signature();
    let signature = JavaMethodSignature {
        receiver: None,
        parameters: arguments
            .iter()
            .map(|argument| argument.ty.clone())
            .collect(),
        result: expected.result,
        checked_exceptions: expected.checked_exceptions,
        nullable_result: expected.nullable_result,
        pure: expected.pure,
    };
    JavaExpr {
        ty: signature.result.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable,
                signature,
            },
            receiver: None,
            arguments,
        },
    }
}
pub(super) fn known_field(field: JavaKnownField) -> JavaExpr {
    JavaExpr {
        ty: field.ty(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Value(JavaValueRef::KnownField(field)),
    }
}
pub(super) fn known_method_call(
    method: JavaKnownMethod,
    receiver: JavaExpr,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaExpr {
    let expected = method.signature();
    let signature = JavaMethodSignature {
        receiver: Some(receiver.ty.clone()),
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: expected.checked_exceptions,
        nullable_result: expected.nullable_result,
        pure: expected.pure,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: receiver.ty.clone(),
                name: identifier(method.name().text()),
                signature,
                origin: JavaMemberOrigin::Known(method),
            },
            receiver: Some(Box::new(receiver)),
            arguments,
        },
    }
}
pub(super) fn known_generic_call(
    callable: JavaKnownCallable,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaExpr {
    let signature = JavaMethodSignature {
        receiver: None,
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable,
                signature,
            },
            receiver: None,
            arguments,
        },
    }
}
pub(super) fn runtime_call(
    callable: JavaRuntimeCallable,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaExpr {
    let signature = JavaMethodSignature {
        receiver: None,
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Runtime {
                callable,
                signature,
            },
            receiver: None,
            arguments,
        },
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JavaRuntimeFailure {
    CheckedOverflow,
    DivisionByZero,
    IndexOutOfBounds,
    InvalidShift,
    InvalidUnicodeScalar,
    InvalidUtf8,
    NarrowingOutOfRange,
    RemainderByZero,
}

impl JavaRuntimeFailure {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::CheckedOverflow => "checked_overflow",
            Self::DivisionByZero => "division_by_zero",
            Self::IndexOutOfBounds => "index_out_of_bounds",
            Self::InvalidShift => "invalid_shift",
            Self::InvalidUnicodeScalar => "invalid_unicode_scalar",
            Self::InvalidUtf8 => "invalid_utf8",
            Self::NarrowingOutOfRange => "narrowing_out_of_range",
            Self::RemainderByZero => "remainder_by_zero",
        }
    }
}

pub(super) fn runtime_fail(result: JavaType, failure: JavaRuntimeFailure) -> JavaExpr {
    runtime_call(
        JavaRuntimeCallable::Fail,
        vec![
            string_literal(failure.name()),
            string_literal(failure.name()),
        ],
        result,
    )
}
pub(super) fn runtime_ok(result: JavaType, value: JavaExpr) -> JavaExpr {
    runtime_call(JavaRuntimeCallable::Ok, vec![value], result)
}
pub(super) fn new_known(
    constructor: JavaKnownConstructor,
    owner: JavaType,
    arguments: Vec<JavaExpr>,
) -> JavaExpr {
    JavaExpr {
        ty: owner.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::New {
            constructor: JavaConstructorRef::Known {
                constructor,
                owner,
                parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
            },
            arguments,
        },
    }
}
pub(super) fn member_call(
    receiver: JavaExpr,
    member: JavaRuntimeMember,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaExpr {
    let signature = JavaMethodSignature {
        receiver: Some(receiver.ty.clone()),
        parameters: arguments.iter().map(|value| value.ty.clone()).collect(),
        result: result.clone(),
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    };
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: receiver.ty.clone(),
                name: identifier(member.name()),
                signature,
                origin: JavaMemberOrigin::Runtime(member),
            },
            receiver: Some(Box::new(receiver)),
            arguments,
        },
    }
}
