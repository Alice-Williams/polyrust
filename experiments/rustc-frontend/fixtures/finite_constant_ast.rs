//! Typed probes for owned declarations and reads.
pub const POSITIVE_ZERO: f64 = 0.0;
pub const NEGATIVE_ZERO: f64 = -0.0;
pub const TENTH: f64 = 0.1;
pub const SAME_VALUE: f64 = 0.1;
const PRIVATE: f64 = f64::from_bits(1);
struct Holder;
impl Holder {
    const VALUE: f64 = f64::MIN_POSITIVE / 2.0;
}
pub fn read_positive() -> f64 {
    POSITIVE_ZERO
}
pub fn read_negative() -> f64 {
    NEGATIVE_ZERO
}
pub fn read_tenth() -> f64 {
    TENTH
}
pub fn read_same() -> f64 {
    SAME_VALUE
}
pub fn read_private() -> f64 {
    PRIVATE
}
pub fn read_local() -> f64 {
    const VALUE: f64 = 1.0 / 3.0;
    VALUE
}
pub fn read_unused() -> f64 {
    #[allow(dead_code)]
    const UNUSED: f64 = -0.0;
    POSITIVE_ZERO
}
pub fn read_inherent() -> f64 {
    Holder::VALUE
}
