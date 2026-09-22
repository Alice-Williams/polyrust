//! Public/local/private/inherent infinity mapping probe.
pub const POSITIVE: f64 = f64::INFINITY;
pub const NEGATIVE: f64 = f64::NEG_INFINITY;
pub const OVERFLOW: f64 = f64::MAX * 2.0;
pub const DIVISION: f64 = -1.0 / 0.0;
const PRIVATE: f64 = f64::NEG_INFINITY;
struct Holder;
impl Holder {
    const VALUE: f64 = f64::INFINITY;
}
pub fn positive() -> f64 {
    POSITIVE
}
pub fn negative() -> f64 {
    NEGATIVE
}
pub fn overflow() -> f64 {
    OVERFLOW
}
pub fn division() -> f64 {
    DIVISION
}
pub fn repeated() -> f64 {
    POSITIVE
}
pub fn private() -> f64 {
    PRIVATE
}
pub fn inherent() -> f64 {
    Holder::VALUE
}
pub fn local() -> f64 {
    const VALUE: f64 = f64::INFINITY;
    VALUE
}
pub fn unused() -> f64 {
    #[allow(dead_code)]
    const UNUSED: f64 = f64::NEG_INFINITY;
    0.0
}
