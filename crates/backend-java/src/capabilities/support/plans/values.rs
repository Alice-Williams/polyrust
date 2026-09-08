//! Closed type/value plans shared by the scalar and container capabilities.
use super::{JavaMappingPlan, JavaRepresentation, expressions::JavaExpressionSkeleton, sealed};
use crate::{ast::JavaType, capabilities::support::JavaValueNode};

pub enum JavaValuePlan {
    DirectType(JavaType),
    RuntimeType(JavaType),
    TaggedType(JavaType),
    DirectExpression(JavaExpressionSkeleton),
    RuntimeExpression(JavaExpressionSkeleton),
    TaggedExpression(JavaExpressionSkeleton),
}
impl sealed::JavaMappingPlan for JavaValuePlan {}
impl JavaMappingPlan for JavaValuePlan {
    type Output = JavaValueNode;
    fn representation(&self) -> JavaRepresentation {
        match self {
            Self::DirectType(_) | Self::DirectExpression(_) => JavaRepresentation::Direct,
            Self::RuntimeType(_) | Self::RuntimeExpression(_) => JavaRepresentation::RuntimeHelper,
            Self::TaggedType(_) | Self::TaggedExpression(_) => JavaRepresentation::TaggedValue,
        }
    }
    fn verify_output(&self, output: &Self::Output) -> bool {
        match self {
            Self::DirectType(expected)
            | Self::RuntimeType(expected)
            | Self::TaggedType(expected) => {
                matches!(output, JavaValueNode::Type(actual) if actual == expected)
            }
            Self::DirectExpression(expected)
            | Self::RuntimeExpression(expected)
            | Self::TaggedExpression(expected) => {
                matches!(output, JavaValueNode::Expression(actual) if expected.matches(actual))
            }
        }
    }
}
