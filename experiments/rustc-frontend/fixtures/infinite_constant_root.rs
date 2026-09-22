//! Mixed infinity owner with private, local, inherent and composed reads.
pub use middle::NAMED_POSITIVE as RENAMED;
pub use middle::{NAMED_NEGATIVE, NAMED_POSITIVE};
/// Root-owned negative infinity, not an alias of another crate.
pub const OWN: f64 = f64::NEG_INFINITY;
const PRIVATE: f64 = f64::INFINITY;
struct Holder;
impl Holder {
    const VALUE: f64 = f64::NEG_INFINITY;
}
fn private_read() -> f64 {
    PRIVATE
}
pub fn read_named_positive() -> f64 {
    middle::NAMED_POSITIVE
}
pub fn read_named_negative() -> f64 {
    middle::NAMED_NEGATIVE
}
pub fn read_negate_negative() -> f64 {
    middle::NEGATE_NEGATIVE
}
pub fn read_negate_positive() -> f64 {
    middle::NEGATE_POSITIVE
}
pub fn read_positive_over_positive_zero() -> f64 {
    middle::POSITIVE_OVER_POSITIVE_ZERO
}
pub fn read_positive_over_negative_zero() -> f64 {
    middle::POSITIVE_OVER_NEGATIVE_ZERO
}
pub fn read_negative_over_negative_zero() -> f64 {
    middle::NEGATIVE_OVER_NEGATIVE_ZERO
}
pub fn read_negative_over_positive_zero() -> f64 {
    middle::NEGATIVE_OVER_POSITIVE_ZERO
}
pub fn read_positive_add_overflow() -> f64 {
    middle::POSITIVE_ADD_OVERFLOW
}
pub fn read_negative_add_overflow() -> f64 {
    middle::NEGATIVE_ADD_OVERFLOW
}
pub fn read_positive_mul_overflow() -> f64 {
    middle::POSITIVE_MUL_OVERFLOW
}
pub fn read_negative_mul_overflow() -> f64 {
    middle::NEGATIVE_MUL_OVERFLOW
}
pub fn read_positive_div_overflow() -> f64 {
    middle::POSITIVE_DIV_OVERFLOW
}
pub fn read_negative_div_overflow() -> f64 {
    middle::NEGATIVE_DIV_OVERFLOW
}
pub fn read_bits_positive() -> f64 {
    middle::BITS_POSITIVE
}
pub fn read_bits_negative() -> f64 {
    middle::BITS_NEGATIVE
}
pub fn read_negative_times_negative() -> f64 {
    middle::NEGATIVE_TIMES_NEGATIVE
}
pub fn read_positive_times_negative() -> f64 {
    middle::POSITIVE_TIMES_NEGATIVE
}
pub fn read_alias_positive() -> f64 {
    middle::ALIAS_POSITIVE
}
pub fn read_alias_negative() -> f64 {
    middle::ALIAS_NEGATIVE
}
pub fn read_other_negative() -> f64 {
    middle::OTHER_NEGATIVE
}
pub fn read_other_same() -> f64 {
    middle::OTHER_SAME
}
pub fn read_alias() -> f64 {
    middle::nested::AGAIN
}
pub fn read_own() -> f64 {
    OWN
}
pub fn read_private() -> f64 {
    private_read()
}
pub fn read_local() -> f64 {
    {
        const VALUE: f64 = -f64::MAX * 2.0;
        VALUE
    }
}
pub fn read_unused() -> f64 {
    {
        #[allow(dead_code)]
        const UNUSED: f64 = f64::INFINITY;
        OWN
    }
}
pub fn read_inherent() -> f64 {
    Holder::VALUE
}
pub fn read_absolute() -> f64 {
    middle::NAMED_NEGATIVE.abs()
}
pub fn read_negation() -> f64 {
    -middle::NAMED_POSITIVE
}
pub fn read_addition() -> f64 {
    middle::NAMED_NEGATIVE + 1.0
}
pub fn read_subtraction() -> f64 {
    middle::NAMED_POSITIVE - 1.0
}
pub fn read_multiplication() -> f64 {
    middle::NAMED_NEGATIVE * -2.0
}
pub fn read_division() -> f64 {
    1.0 / middle::NAMED_POSITIVE
}
pub fn read_negative_zero() -> f64 {
    1.0 / middle::NAMED_NEGATIVE
}
pub fn read_nan() -> f64 {
    if middle::NAMED_POSITIVE.is_nan() {
        1.0
    } else {
        0.0
    }
}
pub fn read_remainder() -> f64 {
    if (middle::NAMED_POSITIVE % 2.0).is_nan() {
        1.0
    } else {
        0.0
    }
}
pub fn read_truncation() -> f64 {
    middle::NAMED_NEGATIVE.trunc()
}
pub fn read_comparison() -> f64 {
    if middle::NAMED_NEGATIVE < 0.0 && middle::NAMED_POSITIVE > 0.0 {
        1.0
    } else {
        0.0
    }
}
