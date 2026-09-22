//! Mixed finite-constant owner with private, local and inherent reads.
pub use middle::TENTH as RENAMED;
pub use middle::{NEGATIVE_ZERO, POSITIVE_ZERO, TENTH};
/// Root-owned constant, not an alias of a dependency.
pub const OWN: f64 = 0.5;
const PRIVATE: f64 = -0.0;
struct Holder;
impl Holder {
    const VALUE: f64 = f64::from_bits(1);
}
fn private_read() -> f64 {
    PRIVATE
}
pub fn read_private() -> f64 {
    private_read()
}
pub fn read_local() -> f64 {
    const VALUE: f64 = 1.0 / 3.0;
    VALUE
}
pub fn read_unused() -> f64 {
    #[allow(dead_code)]
    const UNUSED: f64 = f64::MIN_POSITIVE;
    OWN
}
pub fn read_inherent() -> f64 {
    Holder::VALUE
}
pub fn read_own() -> f64 {
    OWN
}
pub fn read_positive_zero() -> f64 {
    middle::POSITIVE_ZERO
}
pub fn read_negative_zero() -> f64 {
    middle::NEGATIVE_ZERO
}
pub fn read_min_subnormal() -> f64 {
    middle::MIN_SUBNORMAL
}
pub fn read_max_subnormal() -> f64 {
    middle::MAX_SUBNORMAL
}
pub fn read_min_normal() -> f64 {
    middle::MIN_NORMAL
}
pub fn read_max_finite() -> f64 {
    middle::MAX_FINITE
}
pub fn read_negative_max() -> f64 {
    middle::NEGATIVE_MAX
}
pub fn read_one_ulp() -> f64 {
    middle::ONE_ULP
}
pub fn read_tenth() -> f64 {
    middle::TENTH
}
pub fn read_half_integer() -> f64 {
    middle::HALF_INTEGER
}
pub fn read_half_ulp() -> f64 {
    middle::HALF_ULP
}
pub fn read_sum() -> f64 {
    middle::SUM
}
pub fn read_third() -> f64 {
    middle::THIRD
}
pub fn read_half_normal() -> f64 {
    middle::HALF_NORMAL
}
pub fn read_two_subnormals() -> f64 {
    middle::TWO_SUBNORMALS
}
pub fn read_half_max() -> f64 {
    middle::HALF_MAX
}
pub fn read_zero_product() -> f64 {
    middle::ZERO_PRODUCT
}
pub fn read_indexed() -> f64 {
    middle::INDEXED
}
pub fn read_forward() -> f64 {
    middle::FORWARD
}
pub fn read_same_value() -> f64 {
    middle::SAME_VALUE
}
pub fn read_other_tenth() -> f64 {
    middle::OTHER_TENTH
}
pub fn read_alias() -> f64 {
    middle::nested::AGAIN
}
pub fn read_absolute() -> f64 {
    (-OWN).abs()
}
pub fn read_arithmetic() -> f64 {
    OWN + 0.25
}
pub fn read_negation() -> f64 {
    -OWN
}
pub fn read_nan() -> f64 {
    if OWN.is_nan() { 1.0 } else { 0.0 }
}
pub fn read_remainder() -> f64 {
    OWN % 0.25
}
pub fn read_truncation() -> f64 {
    OWN.trunc()
}
pub fn read_other_same() -> f64 {
    middle::OTHER_SAME
}
