//! Private flow unions; only checked same-type ranges enter canonical domains.
mod integers;
mod restrictions;
mod transfer;
pub(in crate::ownership) use transfer::{DomainTransfer, NumericLoss};

use super::{CScalarRange, floating::FloatRange, integer::IntegerRange};
use crate::{
    ast::CScalarType,
    ownership::{CSafetyError as E, constants::CNumber},
};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ownership) struct NumericDomain {
    kind: DomainKind,
}
#[derive(Clone, Debug, PartialEq)]
enum DomainKind {
    Integer {
        ty: CScalarType,
        parts: Vec<IntegerRange>,
    },
    Float(Option<FloatRange>),
}

impl NumericDomain {
    pub(in crate::ownership) fn full(ty: CScalarType) -> Result<Self, E> {
        Ok(Self::range(CScalarRange::full(ty)?))
    }
    pub(in crate::ownership) fn exact(value: CNumber) -> Self {
        Self::range(CScalarRange::exact(value))
    }
    pub(in crate::ownership) fn range(value: CScalarRange) -> Self {
        Self {
            kind: match value {
                CScalarRange::Integer(value) => DomainKind::Integer {
                    ty: value.ty(),
                    parts: vec![value],
                },
                CScalarRange::Float(value) => DomainKind::Float(Some(value)),
            },
        }
    }
    pub(in crate::ownership) fn empty(ty: CScalarType) -> Self {
        Self {
            kind: if ty == CScalarType::F64 {
                DomainKind::Float(None)
            } else {
                DomainKind::Integer {
                    ty,
                    parts: Vec::new(),
                }
            },
        }
    }
    pub(in crate::ownership) fn ty(&self) -> CScalarType {
        match &self.kind {
            DomainKind::Integer { ty, .. } => *ty,
            DomainKind::Float(_) => CScalarType::F64,
        }
    }
    pub(in crate::ownership) fn is_empty(&self) -> bool {
        match &self.kind {
            DomainKind::Integer { parts, .. } => parts.is_empty(),
            DomainKind::Float(value) => value.is_none(),
        }
    }
    pub(in crate::ownership) fn exact_value(&self) -> Option<CNumber> {
        match &self.kind {
            DomainKind::Integer { parts, .. } if parts.len() == 1 => {
                parts[0].exact_value().map(CNumber::Integer)
            }
            DomainKind::Float(Some(value)) => value.exact_value().map(CNumber::Double),
            _ => None,
        }
    }
    pub(in crate::ownership) fn truth(&self) -> Option<bool> {
        let mut result = None;
        for part in self.parts() {
            let truth = part.truth()?;
            if result.is_some_and(|old| old != truth) {
                return None;
            }
            result = Some(truth);
        }
        result
    }
    pub(in crate::ownership) fn join(&self, other: &Self) -> Result<Self, E> {
        if self.ty() != other.ty() {
            return Err(E::InvalidNumericRange);
        }
        match (&self.kind, &other.kind) {
            (DomainKind::Integer { ty, parts: left }, DomainKind::Integer { parts: right, .. }) => {
                Self::integers(*ty, left.iter().chain(right).copied().collect())
            }
            (DomainKind::Float(left), DomainKind::Float(right)) => Ok(Self {
                kind: DomainKind::Float(match (left, right) {
                    (Some(left), Some(right)) => Some(left.join(*right)),
                    (left, right) => left.or(*right),
                }),
            }),
            _ => Err(E::InvalidNumericRange),
        }
    }
    /// Called only at a cyclic graph join: growth loses precision, never values.
    pub(in crate::ownership) fn widen(&self, incoming: &Self) -> Result<Self, E> {
        let joined = self.join(incoming)?;
        if joined == *self || self.is_empty() {
            Ok(joined)
        } else {
            Self::full(self.ty())
        }
    }
    pub(in crate::ownership) fn integer_bounds(&self) -> Option<(i128, i128)> {
        match &self.kind {
            DomainKind::Integer { parts, .. } => Some((parts.first()?.min(), parts.last()?.max())),
            DomainKind::Float(_) => None,
        }
    }
    pub(in crate::ownership) fn floating_bounds(&self) -> Option<(f64, f64)> {
        match self.kind {
            DomainKind::Float(value) => value?.bounds(),
            _ => None,
        }
    }
    pub(in crate::ownership) fn may_nan(&self) -> bool {
        matches!(self.kind, DomainKind::Float(Some(value)) if value.may_nan())
    }
    fn parts(&self) -> Vec<CScalarRange> {
        match &self.kind {
            DomainKind::Integer { parts, .. } => {
                parts.iter().copied().map(CScalarRange::Integer).collect()
            }
            DomainKind::Float(value) => value.iter().copied().map(CScalarRange::Float).collect(),
        }
    }
    #[cfg(test)]
    pub(in crate::ownership) fn contains(&self, value: CNumber) -> bool {
        self.parts().iter().any(|part| part.contains(value))
    }
}
