//! Closed Java representations supported by portable ordering.
use crate::ast::{JavaKnownType, JavaPrimitive, JavaType, JavaTypeName};
use portable_diagnostics::Diagnostic;
pub(super) enum JavaOrderingDomain {
    Numeric,
    String,
    Scalar,
}
pub(super) fn select(ty: &JavaType) -> Result<JavaOrderingDomain, Vec<Diagnostic>> {
    match ty {
        JavaType::Primitive(JavaPrimitive::Int | JavaPrimitive::Long | JavaPrimitive::Double) => {
            Ok(JavaOrderingDomain::Numeric)
        }
        JavaType::Reference(JavaTypeName::Known(JavaKnownType::String)) => {
            Ok(JavaOrderingDomain::String)
        }
        JavaType::Reference(JavaTypeName::Known(JavaKnownType::RuntimeScalar)) => {
            Ok(JavaOrderingDomain::Scalar)
        }
        _ => Err(vec![crate::lower::diagnostic(
            "Java ordering requires int, long, double, String, or RuntimeScalar",
        )]),
    }
}
