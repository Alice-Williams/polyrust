//! Public forwarding without copying its dependency implementation.
#![forbid(unsafe_code)]
pub fn remainder(left: f64, right: f64) -> f64 {
    remainder_middle::operations::remainder(left, right)
}
