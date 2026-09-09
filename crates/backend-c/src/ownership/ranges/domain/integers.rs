//! Canonical disjoint inclusive intervals; adjacency is merged without wrapping.
use super::{DomainKind, E, IntegerRange, NumericDomain};
use crate::ast::CScalarType;

impl NumericDomain {
    pub(super) fn integers(ty: CScalarType, mut parts: Vec<IntegerRange>) -> Result<Self, E> {
        if ty == CScalarType::F64 || parts.iter().any(|part| part.ty() != ty) {
            return Err(E::InvalidNumericRange);
        }
        parts.sort_by_key(|part| part.min());
        let mut merged: Vec<IntegerRange> = Vec::new();
        for part in parts {
            if let Some(last) = merged.last_mut()
                && part.min() <= last.max() + 1
            {
                *last = IntegerRange::checked(ty, last.min(), last.max().max(part.max()))?;
            } else {
                merged.push(part);
            }
        }
        Ok(Self {
            kind: DomainKind::Integer { ty, parts: merged },
        })
    }

    pub(in crate::ownership) fn restrict_integer(&self, min: i128, max: i128) -> Result<Self, E> {
        let DomainKind::Integer { ty, parts } = &self.kind else {
            return Err(E::InvalidNumericRange);
        };
        let parts = parts
            .iter()
            .filter_map(|part| {
                let low = min.max(part.min());
                let high = max.min(part.max());
                (low <= high).then(|| IntegerRange::checked(*ty, low, high))
            })
            .collect::<Result<Vec<_>, E>>()?;
        Self::integers(*ty, parts)
    }

    pub(in crate::ownership) fn exclude_integer(&self, excluded: i128) -> Result<Self, E> {
        let DomainKind::Integer { ty, parts } = &self.kind else {
            return Err(E::InvalidNumericRange);
        };
        let mut result = Vec::new();
        for part in parts {
            if !part.contains(excluded) {
                result.push(*part);
                continue;
            }
            if part.min() < excluded {
                result.push(IntegerRange::checked(*ty, part.min(), excluded - 1)?);
            }
            if excluded < part.max() {
                result.push(IntegerRange::checked(*ty, excluded + 1, part.max())?);
            }
        }
        Self::integers(*ty, result)
    }

    pub(in crate::ownership) fn intersect(&self, other: &Self) -> Result<Self, E> {
        if self.ty() != other.ty() {
            return Err(E::InvalidNumericRange);
        }
        match (&self.kind, &other.kind) {
            (DomainKind::Integer { ty, parts: left }, DomainKind::Integer { parts: right, .. }) => {
                let mut result = Vec::new();
                for left in left {
                    for right in right {
                        let min = left.min().max(right.min());
                        let max = left.max().min(right.max());
                        if min <= max {
                            result.push(IntegerRange::checked(*ty, min, max)?);
                        }
                    }
                }
                Self::integers(*ty, result)
            }
            (DomainKind::Float(left), DomainKind::Float(right)) => Ok(Self {
                kind: DomainKind::Float(super::restrictions::intersection(*left, *right)),
            }),
            _ => Err(E::InvalidNumericRange),
        }
    }
}
