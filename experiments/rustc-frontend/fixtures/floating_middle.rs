//! Separately compiled middle crate, retaining original producer authority.
pub fn relay(value: f64) -> f64 {
    floating_leaf::identity(value)
}
pub fn negate(value: f64) -> f64 {
    floating_leaf::negate(value)
}
