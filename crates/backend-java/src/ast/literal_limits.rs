//! Java 21 literal encoding bounds, independent of rendered source escaping.

use super::operator_signatures::literal_matches_type;
use super::{JavaLiteral, JavaType};
use portable_codegen::AstViolation;
use portable_diagnostics::DiagnosticCode;

// javac rejects literal encodings >= 65,535, even though the classfile length
// field itself can encode 65,535. Native boundary tests lock the compiler rule.
pub(crate) const MAX_LITERAL_BYTES: usize = 65_534;

fn unit_bytes(unit: u16) -> usize {
    match unit {
        1..=0x7f => 1,
        0..=0x7ff => 2,
        _ => 3,
    }
}

fn units_fit(units: impl Iterator<Item = u16>) -> bool {
    let mut size = 0;
    for unit in units {
        size += unit_bytes(unit);
        if size > MAX_LITERAL_BYTES {
            return false;
        }
    }
    true
}

/// Never splits a Unicode scalar (including its UTF-16 surrogate pair).
pub(crate) fn string_chunks(value: &str) -> Vec<&str> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut size = 0;
    for (index, scalar) in value.char_indices() {
        let mut buffer = [0; 2];
        let bytes: usize = scalar
            .encode_utf16(&mut buffer)
            .iter()
            .copied()
            .map(unit_bytes)
            .sum();
        if size + bytes > MAX_LITERAL_BYTES {
            chunks.push(&value[start..index]);
            start = index;
            size = 0;
        }
        size += bytes;
    }
    chunks.push(&value[start..]);
    chunks
}

pub(super) fn verify_literal(literal: &JavaLiteral, ty: &JavaType) -> Vec<AstViolation> {
    let mut violations = Vec::new();
    if !literal_matches_type(literal, ty) {
        violations.push(AstViolation::new(
            DiagnosticCode::TypeMismatch,
            "literal does not match its declared Java type",
        ));
    }
    if matches!(literal, JavaLiteral::CharScalar(value) if char::from_u32(*value).is_none()) {
        violations.push(AstViolation::new(
            DiagnosticCode::TypeMismatch,
            "character literal is not a Unicode scalar",
        ));
    }
    let fits = match literal {
        JavaLiteral::String(value) => units_fit(value.encode_utf16()),
        JavaLiteral::Utf16Units(units) => units_fit(units.iter().copied()),
        JavaLiteral::Boolean(_)
        | JavaLiteral::I32(_)
        | JavaLiteral::I64(_)
        | JavaLiteral::CharScalar(_)
        | JavaLiteral::InternalNull(_) => true,
    };
    if !fits {
        violations.push(AstViolation::new(
            DiagnosticCode::InvalidStructure,
            "Java literal exceeds the modified-UTF-8 constant limit",
        ));
    }
    violations
}
