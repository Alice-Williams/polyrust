//! Exact observations retain zero sign; a zero interval need not have one sign.
use crate::ownership::CSafetyError as E;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ownership) struct FloatRange {
    kind: FloatKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FloatKind {
    Exact(u64),
    Ordered { min: f64, max: f64, may_nan: bool },
}
impl FloatRange {
    pub(super) fn exact(value: f64) -> Self {
        Self {
            kind: FloatKind::Exact(value.to_bits()),
        }
    }
    pub(super) fn checked(min: f64, max: f64, may_nan: bool) -> Result<Self, E> {
        if min.is_nan() || max.is_nan() || min > max {
            return Err(E::InvalidNumericRange);
        }
        Ok(Self {
            kind: FloatKind::Ordered { min, max, may_nan },
        })
    }
    pub(super) fn unknown() -> Self {
        Self {
            kind: FloatKind::Ordered {
                min: f64::NEG_INFINITY,
                max: f64::INFINITY,
                may_nan: true,
            },
        }
    }
    pub(super) fn exact_value(self) -> Option<f64> {
        match self.kind {
            FloatKind::Exact(bits) => Some(f64::from_bits(bits)),
            FloatKind::Ordered { .. } => None,
        }
    }
    pub(super) fn bounds(self) -> Option<(f64, f64)> {
        match self.kind {
            FloatKind::Exact(bits) => {
                let value = f64::from_bits(bits);
                (!value.is_nan()).then_some((value, value))
            }
            FloatKind::Ordered { min, max, .. } => Some((min, max)),
        }
    }
    pub(super) fn may_nan(self) -> bool {
        match self.kind {
            FloatKind::Exact(bits) => f64::from_bits(bits).is_nan(),
            FloatKind::Ordered { may_nan, .. } => may_nan,
        }
    }
    pub(super) fn join(self, right: Self) -> Self {
        if self == right {
            return self;
        }
        let bounds = match (self.bounds(), right.bounds()) {
            (Some((a, b)), Some((c, d))) => Some((a.min(c), b.max(d))),
            (left, right) => left.or(right),
        };
        match bounds {
            Some((min, max)) => Self::checked(min, max, self.may_nan() || right.may_nan())
                .expect("ordered input bounds"),
            None => Self::exact(f64::NAN),
        }
    }
    pub(super) fn truth(self) -> Option<bool> {
        match self.bounds() {
            None => Some(true),
            Some((min, max)) if min > 0.0 || max < 0.0 => Some(true),
            Some((min, max)) if min == 0.0 && max == 0.0 && !self.may_nan() => Some(false),
            _ => None,
        }
    }
    #[cfg(test)]
    pub(super) fn contains(self, value: f64) -> bool {
        if value.is_nan() {
            return self.may_nan();
        }
        match self.kind {
            FloatKind::Exact(bits) => value.to_bits() == bits,
            FloatKind::Ordered { min, max, .. } => min <= value && value <= max,
        }
    }
}
