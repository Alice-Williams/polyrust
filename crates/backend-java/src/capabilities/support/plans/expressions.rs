//! Plan-guided matching of mapping-owned expression nodes.
//!
//! Missing child skeletons designate input operand holes. They never discover
//! or classify arbitrary operand subtrees.
use crate::ast::{
    JavaBinaryOperator, JavaCallableRef, JavaConstructorRef, JavaExpr, JavaExprKind, JavaLiteral,
    JavaMemberOrigin, JavaType, JavaUnaryOperator,
};
use crate::dialect::{JavaKnownCallable, JavaKnownConstructor, JavaRuntimeCallable};

#[derive(Clone, Debug)]
pub struct JavaExpressionSkeleton {
    pub(crate) ty: JavaType,
    pub(crate) node: JavaExpressionNode,
}

#[derive(Clone, Debug)]
pub(crate) enum JavaCallOrigin {
    Known(JavaKnownCallable),
    Runtime(JavaRuntimeCallable),
    Member(JavaMemberOrigin),
}

#[derive(Clone, Debug)]
pub(crate) enum JavaConstructionOrigin {
    Known(JavaKnownConstructor),
}

type Child = Option<Box<JavaExpressionSkeleton>>;

#[derive(Clone, Debug)]
pub(crate) enum JavaExpressionNode {
    Literal(JavaLiteral),
    Unary {
        operator: JavaUnaryOperator,
        operand: Child,
    },
    Binary {
        operator: JavaBinaryOperator,
        left: Child,
        right: Child,
    },
    Call {
        origin: JavaCallOrigin,
        receiver: bool,
        arguments: Vec<Child>,
    },
    New {
        origin: JavaConstructionOrigin,
        arguments: Vec<Child>,
    },
    Cast {
        target: JavaType,
        value: Child,
    },
    Conditional {
        condition: Child,
        when_true: Child,
        when_false: Child,
    },
}

fn child_matches(expected: &Child, actual: &JavaExpr) -> bool {
    expected
        .as_ref()
        .is_none_or(|expected| expected.matches(actual))
}

fn arguments_match(expected: &[Child], actual: &[JavaExpr]) -> bool {
    expected.len() == actual.len()
        && expected
            .iter()
            .zip(actual)
            .all(|(expected, actual)| child_matches(expected, actual))
}

impl JavaExpressionSkeleton {
    pub(crate) fn matches(&self, actual: &JavaExpr) -> bool {
        if self.ty != actual.ty {
            return false;
        }
        match (&self.node, &actual.kind) {
            (JavaExpressionNode::Literal(expected), JavaExprKind::Literal(actual)) => {
                expected == actual
            }
            (
                JavaExpressionNode::Unary { operator, operand },
                JavaExprKind::Unary {
                    operator: actual_op,
                    operand: actual_operand,
                },
            ) => operator == actual_op && child_matches(operand, actual_operand),
            (
                JavaExpressionNode::Binary {
                    operator,
                    left,
                    right,
                },
                JavaExprKind::Binary {
                    operator: actual_op,
                    left: actual_left,
                    right: actual_right,
                },
            ) => {
                operator == actual_op
                    && child_matches(left, actual_left)
                    && child_matches(right, actual_right)
            }
            (
                JavaExpressionNode::Call {
                    origin,
                    receiver,
                    arguments,
                },
                JavaExprKind::Call {
                    callable,
                    receiver: actual_receiver,
                    arguments: actual_args,
                },
            ) => {
                origin.matches(callable)
                    && *receiver == actual_receiver.is_some()
                    && arguments_match(arguments, actual_args)
            }
            (
                JavaExpressionNode::New { origin, arguments },
                JavaExprKind::New {
                    constructor,
                    arguments: actual_args,
                },
            ) => {
                origin.matches(constructor, &self.ty, actual_args)
                    && arguments_match(arguments, actual_args)
            }
            (
                JavaExpressionNode::Cast { target, value },
                JavaExprKind::Cast {
                    target: actual_target,
                    value: actual_value,
                },
            ) => target == actual_target && child_matches(value, actual_value),
            (
                JavaExpressionNode::Conditional {
                    condition,
                    when_true,
                    when_false,
                },
                JavaExprKind::Conditional {
                    condition: actual_condition,
                    when_true: actual_true,
                    when_false: actual_false,
                },
            ) => {
                child_matches(condition, actual_condition)
                    && child_matches(when_true, actual_true)
                    && child_matches(when_false, actual_false)
            }
            _ => false,
        }
    }
}

impl JavaCallOrigin {
    fn matches(&self, actual: &JavaCallableRef) -> bool {
        match (self, actual) {
            (Self::Known(expected), JavaCallableRef::Known { callable, .. }) => {
                expected == callable
            }
            (Self::Runtime(expected), JavaCallableRef::Runtime { callable, .. }) => {
                expected == callable
            }
            (Self::Member(expected), JavaCallableRef::Member { origin, .. }) => expected == origin,
            _ => false,
        }
    }
}

impl JavaConstructionOrigin {
    fn matches(
        &self,
        actual: &JavaConstructorRef,
        result: &JavaType,
        arguments: &[JavaExpr],
    ) -> bool {
        match (self, actual) {
            (
                Self::Known(expected),
                JavaConstructorRef::Known {
                    constructor,
                    owner,
                    parameters,
                },
            ) => {
                expected == constructor
                    && owner == result
                    && expected.accepts(owner, parameters)
                    && parameters.len() == arguments.len()
                    && parameters
                        .iter()
                        .zip(arguments)
                        .all(|(ty, value)| ty == &value.ty)
            }
            _ => false,
        }
    }
}
