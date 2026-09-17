//! Original producer identities used to observe ordered arithmetic evaluation.
#![forbid(unsafe_code)]

/// First operand retains its original producer.
pub fn left(value: f64) -> f64 {
    identity(value)
}

/// Second operand retains its original producer.
pub fn right(value: f64) -> f64 {
    identity(value)
}

// A private implementation detail must not become a public facade member.
fn identity(value: f64) -> f64 {
    value
}
