//! Selected runtime elements are tied to current scalar bindings, not free-form IDs.
use super::{Key, Root};
use crate::ast::{CLocalRef, CObjectTypeKind, CParameterRef, CScalarType};
use crate::ownership::CSafetyError as E;

#[cfg(test)]
#[path = "../../tests/buffer_private_indices.rs"]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum IndexBinding {
    Local(Box<CLocalRef>),
    Parameter(Box<CParameterRef>),
}
impl IndexBinding {
    fn actual(key: &Key) -> Option<Self> {
        if !key.whole_root()
            || !matches!(
                key.ty().kind(),
                CObjectTypeKind::Scalar(CScalarType::Size | CScalarType::U64)
            )
        {
            return None;
        }
        match key.root() {
            Root::Local(local) => Some(Self::Local(Box::new(local.clone()))),
            Root::Parameter(parameter) => Some(Self::Parameter(Box::new(parameter.clone()))),
            _ => None,
        }
    }
    fn root(&self) -> Root {
        match self {
            Self::Local(local) => Root::Local((**local).clone()),
            Self::Parameter(parameter) => Root::Parameter((**parameter).clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ownership) struct ElementIndex {
    first: u64,
    last: u64,
    current: Option<IndexBinding>,
}
impl ElementIndex {
    pub(in crate::ownership) fn constant(value: u64) -> Self {
        Self {
            first: value,
            last: value,
            current: None,
        }
    }
    pub(in crate::ownership) fn checked(
        first: u64,
        last: u64,
        source: Option<&Key>,
    ) -> Result<Self, E> {
        if first > last {
            return Err(E::IndexOutOfBounds);
        }
        Ok(Self {
            first,
            last,
            current: source.and_then(IndexBinding::actual),
        })
    }
    pub(in crate::ownership) fn bounds(&self) -> (u64, u64) {
        (self.first, self.last)
    }
    pub(in crate::ownership) fn source(&self) -> Option<Key> {
        self.current
            .as_ref()
            .map(|binding| Key::from_root(binding.root()))
    }
    pub(in crate::ownership) fn join_selection(&self, other: &Self) -> Option<Self> {
        if self.current.is_some() && self.current == other.current {
            Some(Self {
                first: self.first.min(other.first),
                last: self.last.max(other.last),
                current: self.current.clone(),
            })
        } else if self.current.is_none()
            && other.current.is_none()
            && let Some(value) = self.constant_value()
            && other.constant_value() == Some(value)
        {
            Some(Self::constant(value))
        } else {
            None
        }
    }
    pub(in crate::ownership) fn exact(&self) -> bool {
        self.first == self.last || self.current.is_some()
    }
    pub(in crate::ownership) fn constant_value(&self) -> Option<u64> {
        (self.first == self.last).then_some(self.first)
    }
    pub(in crate::ownership) fn overlaps(&self, other: &Self) -> bool {
        self.first <= other.last && other.first <= self.last
    }
    pub(in crate::ownership) fn touches(&self, mut test: impl FnMut(&Root) -> bool) -> bool {
        self.current
            .as_ref()
            .is_some_and(|current| test(&current.root()))
    }
}
