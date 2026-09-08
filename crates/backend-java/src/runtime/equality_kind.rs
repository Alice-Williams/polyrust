//! Semantic equality and portable test equality are different operations.

use super::call_builders::known_call;
use super::expression_builders::{binary, cast};
use crate::ast::{JavaBinaryOperator, JavaExpr, JavaPrimitive, JavaRuntimeMember, JavaType};
use crate::dialect::{JavaKnownCallable, JavaRuntimeCallable};

#[derive(Clone, Copy)]
pub(super) enum EqualityKind {
    Semantic,
    PortableExpectation,
}

impl EqualityKind {
    pub(super) fn callable(self) -> JavaRuntimeCallable {
        match self {
            Self::Semantic => JavaRuntimeCallable::SemanticEqual,
            Self::PortableExpectation => JavaRuntimeCallable::DeepEqual,
        }
    }

    pub(super) fn member(self) -> JavaRuntimeMember {
        match self {
            Self::Semantic => JavaRuntimeMember::SemanticEquals,
            Self::PortableExpectation => JavaRuntimeMember::DeepEquals,
        }
    }

    pub(super) fn float_comparison(self, left: JavaExpr, right: JavaExpr) -> JavaExpr {
        let boolean = JavaType::primitive(JavaPrimitive::Boolean);
        match self {
            Self::Semantic => binary(
                JavaBinaryOperator::Equal,
                cast(JavaType::primitive(JavaPrimitive::Double), left),
                cast(JavaType::primitive(JavaPrimitive::Double), right),
                boolean,
            ),
            Self::PortableExpectation => binary(
                JavaBinaryOperator::LogicalOr,
                binary(
                    JavaBinaryOperator::Equal,
                    known_call(JavaKnownCallable::DoubleToRawLongBits, vec![left.clone()]),
                    known_call(JavaKnownCallable::DoubleToRawLongBits, vec![right.clone()]),
                    boolean.clone(),
                ),
                binary(
                    JavaBinaryOperator::LogicalAnd,
                    known_call(JavaKnownCallable::DoubleIsNaN, vec![left]),
                    known_call(JavaKnownCallable::DoubleIsNaN, vec![right]),
                    boolean.clone(),
                ),
                boolean,
            ),
        }
    }
}
