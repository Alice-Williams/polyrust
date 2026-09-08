//! Exact intrinsic roots; fallibility is independent of native/runtime spelling.
use super::expressions::{JavaCallOrigin, JavaExpressionNode, JavaExpressionSkeleton};
use super::{JavaMappingPlan, JavaRepresentation, sealed};
use crate::ast::{
    JavaBinaryOperator, JavaExpr, JavaKnownType, JavaLiteral, JavaMemberOrigin, JavaPrimitive,
    JavaRuntimeMember, JavaType, JavaUnaryOperator,
};
use crate::dialect::{JavaKnownCallable, JavaKnownMethod, JavaRuntimeCallable};
use crate::lower::JavaIntrinsicExpr;

#[derive(Clone, Debug)]
pub struct JavaIntrinsicPlan {
    result: JavaType,
    kind: JavaIntrinsicPlanKind,
}

#[derive(Clone, Debug)]
pub(crate) enum JavaIntrinsicPlanKind {
    Unary(JavaUnaryOperator),
    Binary(JavaBinaryOperator),
    KnownCall(JavaKnownCallable, usize),
    KnownMember(JavaKnownMethod, usize),
    Cast,
    Utf16Length,
    StripPrefix,
    RuntimeCall(JavaRuntimeCallable, usize),
    NegatedRuntimeCall(JavaRuntimeCallable, usize),
    StringOrdering(JavaBinaryOperator),
    ScalarOrdering(JavaBinaryOperator),
    OptionUnwrapOr,
    ReplaceMany(usize),
    FallibleRuntimeCall(JavaRuntimeCallable, usize),
}

impl JavaIntrinsicPlan {
    pub(crate) fn new(kind: JavaIntrinsicPlanKind, result: &JavaType) -> Self {
        Self {
            result: result.clone(),
            kind,
        }
    }

    fn skeleton(&self) -> JavaExpressionSkeleton {
        use JavaIntrinsicPlanKind as K;
        let result = self.result.clone();
        let integer = JavaType::primitive(JavaPrimitive::Int);
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        let node = match self.kind {
            K::Unary(operator) => JavaExpressionNode::Unary {
                operator,
                operand: None,
            },
            K::Binary(operator) => JavaExpressionNode::Binary {
                operator,
                left: None,
                right: None,
            },
            K::KnownCall(callable, count) => {
                return call(result, JavaCallOrigin::Known(callable), false, holes(count));
            }
            K::KnownMember(method, count) => {
                return call(
                    result,
                    JavaCallOrigin::Member(JavaMemberOrigin::Known(method)),
                    true,
                    holes(count),
                );
            }
            K::RuntimeCall(callable, count) => {
                return call(
                    result,
                    JavaCallOrigin::Runtime(callable),
                    false,
                    holes(count),
                );
            }
            K::FallibleRuntimeCall(callable, count) => {
                return call(
                    JavaType::generic(JavaKnownType::RuntimeResult, vec![result.boxed()]),
                    JavaCallOrigin::Runtime(callable),
                    false,
                    holes(count),
                );
            }
            K::NegatedRuntimeCall(callable, count) => JavaExpressionNode::Unary {
                operator: JavaUnaryOperator::Not,
                operand: owned(call(
                    result.clone(),
                    JavaCallOrigin::Runtime(callable),
                    false,
                    holes(count),
                )),
            },
            K::Cast => JavaExpressionNode::Cast {
                target: result.clone(),
                value: None,
            },
            K::Utf16Length => JavaExpressionNode::Cast {
                target: result.clone(),
                value: owned(call(
                    integer.clone(),
                    JavaCallOrigin::Member(JavaMemberOrigin::Known(JavaKnownMethod::StringLength)),
                    true,
                    vec![],
                )),
            },
            K::StringOrdering(operator) => JavaExpressionNode::Binary {
                operator,
                left: owned(call(
                    integer.clone(),
                    JavaCallOrigin::Runtime(JavaRuntimeCallable::CompareScalarStrings),
                    false,
                    holes(2),
                )),
                right: owned(JavaExpressionSkeleton {
                    ty: integer.clone(),
                    node: JavaExpressionNode::Literal(JavaLiteral::I32(0)),
                }),
            },
            K::ScalarOrdering(operator) => {
                let scalar = call(
                    integer,
                    JavaCallOrigin::Member(JavaMemberOrigin::Runtime(
                        JavaRuntimeMember::ScalarValue,
                    )),
                    true,
                    vec![],
                );
                JavaExpressionNode::Binary {
                    operator,
                    left: owned(scalar.clone()),
                    right: owned(scalar),
                }
            }
            K::OptionUnwrapOr => JavaExpressionNode::Conditional {
                condition: owned(call(
                    boolean,
                    JavaCallOrigin::Runtime(JavaRuntimeCallable::OptionIsSome),
                    false,
                    holes(1),
                )),
                when_true: owned(call(
                    result.clone(),
                    JavaCallOrigin::Runtime(JavaRuntimeCallable::OptionValue),
                    false,
                    holes(1),
                )),
                when_false: None,
            },
            K::StripPrefix => JavaExpressionNode::Conditional {
                condition: owned(call(
                    boolean,
                    JavaCallOrigin::Member(JavaMemberOrigin::Known(
                        JavaKnownMethod::StringStartsWith,
                    )),
                    true,
                    holes(1),
                )),
                when_true: owned(call(
                    result.clone(),
                    JavaCallOrigin::Member(JavaMemberOrigin::Known(
                        JavaKnownMethod::StringSubstringFrom,
                    )),
                    true,
                    vec![owned(call(
                        integer,
                        JavaCallOrigin::Member(JavaMemberOrigin::Known(
                            JavaKnownMethod::StringLength,
                        )),
                        true,
                        vec![],
                    ))],
                )),
                when_false: None,
            },
            K::ReplaceMany(count) => {
                return call(
                    result,
                    JavaCallOrigin::Runtime(JavaRuntimeCallable::StringReplaceMany),
                    false,
                    vec![
                        None,
                        owned(call(
                            JavaType::generic(
                                JavaKnownType::List,
                                vec![JavaType::known(JavaKnownType::String)],
                            ),
                            JavaCallOrigin::Known(JavaKnownCallable::ListOf),
                            false,
                            holes(count),
                        )),
                    ],
                );
            }
        };
        JavaExpressionSkeleton { ty: result, node }
    }
}

impl sealed::JavaMappingPlan for JavaIntrinsicPlan {}

impl JavaMappingPlan for JavaIntrinsicPlan {
    type Output = JavaIntrinsicExpr;

    fn representation(&self) -> JavaRepresentation {
        use JavaIntrinsicPlanKind as K;
        match self.kind {
            K::Unary(_)
            | K::Binary(_)
            | K::KnownCall(_, _)
            | K::KnownMember(_, _)
            | K::Cast
            | K::Utf16Length
            | K::StripPrefix => JavaRepresentation::Direct,
            K::RuntimeCall(_, _)
            | K::NegatedRuntimeCall(_, _)
            | K::StringOrdering(_)
            | K::ScalarOrdering(_)
            | K::OptionUnwrapOr
            | K::ReplaceMany(_)
            | K::FallibleRuntimeCall(_, _) => JavaRepresentation::RuntimeHelper,
        }
    }

    fn verify_output(&self, output: &Self::Output) -> bool {
        let expression: &JavaExpr = match (&self.kind, output) {
            (
                JavaIntrinsicPlanKind::FallibleRuntimeCall(_, _),
                JavaIntrinsicExpr::Fallible { call, value_type },
            ) if value_type == &self.result => call,
            (JavaIntrinsicPlanKind::FallibleRuntimeCall(_, _), _) => return false,
            (_, JavaIntrinsicExpr::Infallible(value)) => value,
            (_, JavaIntrinsicExpr::Fallible { .. }) => return false,
        };
        self.skeleton().matches(expression)
    }
}

pub(crate) fn holes(count: usize) -> Vec<Option<Box<JavaExpressionSkeleton>>> {
    vec![None; count]
}

pub(crate) fn owned(value: JavaExpressionSkeleton) -> Option<Box<JavaExpressionSkeleton>> {
    Some(Box::new(value))
}

pub(crate) fn call(
    ty: JavaType,
    origin: JavaCallOrigin,
    receiver: bool,
    arguments: Vec<Option<Box<JavaExpressionSkeleton>>>,
) -> JavaExpressionSkeleton {
    JavaExpressionSkeleton {
        ty,
        node: JavaExpressionNode::Call {
            origin,
            receiver,
            arguments,
        },
    }
}
