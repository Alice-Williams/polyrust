//! Java lowering: call builders.

use super::JavaIntrinsicExpr;
use super::declaration_builders::identifier;
use crate::ast::{
    JavaCallableRef, JavaConstructorRef, JavaExpr, JavaExprKind, JavaKnownType, JavaMemberOrigin,
    JavaMethodSignature, JavaPrecedence, JavaType,
};
use crate::dialect::{JavaKnownCallable, JavaKnownConstructor, JavaRuntimeCallable};

pub(crate) fn known_call(callable: JavaKnownCallable, arguments: Vec<JavaExpr>) -> JavaExpr {
    let signature = callable.signature();
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

pub(crate) fn known_generic_call(
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

pub(crate) fn runtime_call(
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

pub(crate) fn runtime_fallible(
    callable: JavaRuntimeCallable,
    arguments: Vec<JavaExpr>,
    result: JavaType,
) -> JavaIntrinsicExpr {
    let wrapped = JavaType::generic(JavaKnownType::RuntimeResult, vec![result.clone().boxed()]);
    let call = runtime_call(callable, arguments, wrapped);
    JavaIntrinsicExpr::Fallible {
        call,
        value_type: result,
    }
}

pub(crate) fn member_call(
    receiver: JavaExpr,
    name: &str,
    arguments: Vec<JavaExpr>,
    result: JavaType,
    origin: JavaMemberOrigin,
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
                name: identifier(name),
                signature,
                origin,
            },
            receiver: Some(Box::new(receiver)),
            arguments,
        },
    }
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
