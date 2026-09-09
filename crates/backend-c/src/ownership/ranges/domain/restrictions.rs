//! Floating restriction never invents an exact zero sign or drops a possible NaN.
use super::{DomainKind, E, FloatRange, NumericDomain};

impl NumericDomain {
    pub(in crate::ownership) fn restrict_float(
        &self,
        min: f64,
        max: f64,
        may_nan: bool,
    ) -> Result<Self, E> {
        let DomainKind::Float(value) = self.kind else {
            return Err(E::InvalidNumericRange);
        };
        let restriction = if min.is_nan() || max.is_nan() {
            return Err(E::InvalidNumericRange);
        } else if min <= max {
            Some(FloatRange::checked(min, max, may_nan)?)
        } else if may_nan {
            Some(FloatRange::exact(f64::NAN))
        } else {
            None
        };
        Ok(Self {
            kind: DomainKind::Float(intersection(value, restriction)),
        })
    }
    pub(in crate::ownership) fn only_nan(&self) -> Result<Self, E> {
        let DomainKind::Float(value) = self.kind else {
            return Err(E::InvalidNumericRange);
        };
        Ok(Self {
            kind: DomainKind::Float(
                value
                    .filter(|value| value.may_nan())
                    .map(|_| FloatRange::exact(f64::NAN)),
            ),
        })
    }
}

pub(super) fn intersection(
    left: Option<FloatRange>,
    right: Option<FloatRange>,
) -> Option<FloatRange> {
    let (left, right) = (left?, right?);
    if let Some(value) = left.exact_value() {
        return exact_intersection(left, value, right);
    }
    if let Some(value) = right.exact_value() {
        return exact_intersection(right, value, left);
    }
    let nan = left.may_nan() && right.may_nan();
    let ((a, b), (c, d)) = (left.bounds()?, right.bounds()?);
    let min = a.max(c);
    let max = b.min(d);
    if min <= max {
        Some(FloatRange::checked(min, max, nan).expect("ordered intersection"))
    } else {
        nan.then(|| FloatRange::exact(f64::NAN))
    }
}

fn exact_intersection(exact: FloatRange, value: f64, other: FloatRange) -> Option<FloatRange> {
    if value.is_nan() {
        return other.may_nan().then_some(exact);
    }
    if let Some(other) = other.exact_value() {
        return (value.to_bits() == other.to_bits()).then_some(exact);
    }
    let (min, max) = other.bounds()?;
    (min <= value && value <= max).then_some(exact)
}
