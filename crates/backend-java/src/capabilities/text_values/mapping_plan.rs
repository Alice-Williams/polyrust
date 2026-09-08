//! Authenticate every owned literal chunk and non-constant assembly call.

use super::JavaTextValuesInput;
use crate::ast::literal_limits::string_chunks;
use crate::ast::{
    JavaCallableRef, JavaExpr, JavaExprKind, JavaKnownType, JavaLiteral, JavaMemberOrigin,
    JavaPrecedence, JavaType,
};
use crate::capabilities::support::JavaValueNode;
use crate::capabilities::support::plans::{JavaMappingPlan, JavaRepresentation, sealed};
use crate::dialect::JavaKnownMethod;
use portable_diagnostics::Diagnostic;

pub enum Plan {
    Type,
    Text(Vec<String>),
}

pub(super) fn select(input: &JavaTextValuesInput) -> Result<Plan, Vec<Diagnostic>> {
    Ok(match input {
        JavaTextValuesInput::Type => Plan::Type,
        JavaTextValuesInput::Value(value) => Plan::Text(
            string_chunks(value)
                .into_iter()
                .map(str::to_owned)
                .collect(),
        ),
    })
}

impl sealed::JavaMappingPlan for Plan {}
impl JavaMappingPlan for Plan {
    type Output = JavaValueNode;

    fn representation(&self) -> JavaRepresentation {
        JavaRepresentation::Direct
    }

    fn verify_output(&self, output: &Self::Output) -> bool {
        match (self, output) {
            (Self::Type, JavaValueNode::Type(ty)) => *ty == JavaType::known(JavaKnownType::String),
            (Self::Text(chunks), JavaValueNode::Expression(value)) => matches_chunks(chunks, value),
            _ => false,
        }
    }
}

fn matches_chunks(chunks: &[String], value: &JavaExpr) -> bool {
    let string = JavaType::known(JavaKnownType::String);
    if value.ty != string || value.precedence != JavaPrecedence::Primary || chunks.is_empty() {
        return false;
    }
    if let [expected] = chunks {
        return matches!(&value.kind, JavaExprKind::Literal(JavaLiteral::String(actual)) if actual == expected);
    }
    let JavaExprKind::Call {
        callable:
            JavaCallableRef::Member {
                owner,
                name,
                signature,
                origin,
            },
        receiver: Some(receiver),
        arguments,
    } = &value.kind
    else {
        return false;
    };
    let method = JavaKnownMethod::StringConcat;
    let [argument] = arguments.as_slice() else {
        return false;
    };
    let middle = chunks.len() / 2;
    *owner == string
        && name.as_str() == method.name().text()
        && *signature == method.signature()
        && *origin == JavaMemberOrigin::Known(method)
        && matches_chunks(&chunks[..middle], receiver)
        && matches_chunks(&chunks[middle..], argument)
}
