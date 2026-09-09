//! Facts are derived snapshots; writes kill relations to the old storage value.
mod projections;
use super::provenance::{Origin, merge};
use super::storage::{Key, Root};
use crate::ast::{CScalarType, CScopeRef, CValue};
use crate::ownership::{CSafetyError as E, ranges::NumericDomain};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ownership) struct Number<'a> {
    pub(super) domain: NumericDomain,
    pub(super) losses: Vec<Origin<'a>>,
    pub(super) predicate: Option<Predicate<'a>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Predicate<'a> {
    pub operand: &'a CValue,
    pub dependencies: BTreeSet<Root>,
    pub polarity: NaNPolarity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NaNPolarity {
    Nonzero,
    Zero,
}

impl<'a> Number<'a> {
    pub(in crate::ownership) fn extent_bounds(&self) -> Result<(u64, u64), E> {
        if !self.losses.is_empty() {
            return Err(E::UnprovedSizeArithmetic);
        }
        let (first, last) = self
            .domain
            .integer_bounds()
            .ok_or(E::ExpectedNumericValue)?;
        Ok((
            u64::try_from(first).map_err(|_| E::IndexOutOfBounds)?,
            u64::try_from(last).map_err(|_| E::IndexOutOfBounds)?,
        ))
    }
    pub(in crate::ownership) fn domain(domain: NumericDomain) -> Self {
        Self {
            domain,
            losses: vec![],
            predicate: None,
        }
    }
    pub(in crate::ownership) fn inherit_losses(&mut self, other: &Self) {
        merge(&mut self.losses, &other.losses);
    }
    fn join(&self, other: &Self, widen: bool) -> Result<Self, E> {
        let converted = other.domain.convert(self.domain.ty())?;
        if converted.loss != crate::ownership::ranges::NumericLoss::None {
            return Err(E::ExpectedNumericValue);
        }
        let domain = if widen {
            self.domain.widen(&converted.domain)?
        } else {
            self.domain.join(&converted.domain)?
        };
        let mut joined = Self::domain(domain);
        joined.inherit_losses(self);
        joined.inherit_losses(other);
        if self.predicate == other.predicate {
            joined.predicate = self.predicate.clone();
        }
        Ok(joined)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::ownership) struct State<'a> {
    cells: BTreeMap<Key, Number<'a>>,
    fallback: BTreeMap<Root, Vec<Origin<'a>>>,
    pub(super) relations: super::relations::Relations<'a>,
}
impl<'a> State<'a> {
    pub(in crate::ownership::numeric_flow) fn roots(&self) -> impl Iterator<Item = Root> + '_ {
        self.cells
            .keys()
            .map(|key| key.root().clone())
            .chain(self.fallback.keys().cloned())
    }
    pub(in crate::ownership) fn retain_memory(&mut self, live: &BTreeSet<Root>) {
        let removed: BTreeSet<_> = self.roots().filter(|root| !live.contains(root)).collect();
        for root in removed {
            self.fresh(&root);
        }
    }
    pub(in crate::ownership) fn forget(&mut self, roots: impl Iterator<Item = Root>) {
        for root in roots {
            self.poison(&root, Origin::Incomplete);
        }
        self.relations = super::relations::Relations::default();
    }
    pub(in crate::ownership) fn read(&self, key: &Key) -> Option<&Number<'a>> {
        self.cells.get(key)
    }
    pub(in crate::ownership) fn number(&self, key: &Key, ty: CScalarType) -> Result<Number<'a>, E> {
        if let Some(value) = self.read(key) {
            let mut value = value.clone();
            if value.domain.ty() != ty {
                let converted = value.domain.convert(ty)?;
                if converted.loss != crate::ownership::ranges::NumericLoss::None {
                    return Err(E::UnprovedSizeArithmetic);
                }
                value.domain = converted.domain;
                value.predicate = None;
            }
            return Ok(value);
        }
        let mut value = Number::domain(NumericDomain::full(ty)?);
        if let Root::Global(object) = key.root() {
            value.losses.push(Origin::Global(Box::new(object.clone())));
        }
        if let Some(origins) = self.fallback.get(key.root()) {
            merge(&mut value.losses, origins);
        }
        Ok(value)
    }
    pub(in crate::ownership) fn set(&mut self, key: Key, mut value: Number<'a>) {
        if value
            .predicate
            .as_ref()
            .is_some_and(|p| p.dependencies.contains(key.root()))
        {
            value.predicate = None;
        }
        self.cells.insert(key, value);
    }
    pub(in crate::ownership) fn kill(&mut self, root: &Root) {
        self.relations.invalidate(|candidate| candidate == root);
        for (key, value) in &mut self.cells {
            if key.root() == root {
                value.domain = NumericDomain::full(value.domain.ty()).expect("checked scalar");
                value.predicate = None;
            }
        }
        for value in self.cells.values_mut() {
            if value
                .predicate
                .as_ref()
                .is_some_and(|p| p.dependencies.contains(root))
            {
                value.predicate = None;
            }
        }
    }
    pub(super) fn opaque(&mut self, addresses: &BTreeSet<Root>, origin: Origin<'a>) {
        for root in addresses {
            merge(
                self.fallback.entry(root.clone()).or_default(),
                std::slice::from_ref(&origin),
            );
        }
        self.relations.invalidate(|root| root.exposed(addresses));
        for (key, value) in &mut self.cells {
            if key.root().exposed(addresses) {
                value.domain = NumericDomain::full(value.domain.ty()).expect("checked scalar");
                value.predicate = None;
                merge(&mut value.losses, std::slice::from_ref(&origin));
            }
        }
        for value in self.cells.values_mut() {
            if value
                .predicate
                .as_ref()
                .is_some_and(|p| p.dependencies.iter().any(|root| root.exposed(addresses)))
            {
                value.predicate = None;
            }
        }
    }
    pub(in crate::ownership) fn leave(&mut self, scope: &CScopeRef) {
        self.relations.invalidate(|root| root.leaves(scope));
        self.cells.retain(|key, _| !key.root().leaves(scope));
        self.fallback.retain(|root, _| !root.leaves(scope));
        for value in self.cells.values_mut() {
            if value
                .predicate
                .as_ref()
                .is_some_and(|p| p.dependencies.iter().any(|root| root.leaves(scope)))
            {
                value.predicate = None;
            }
        }
    }
    pub(in crate::ownership) fn join(&self, other: &Self, widen: bool) -> Result<Self, E> {
        let mut result = Self {
            relations: self.relations.join(&other.relations),
            fallback: self.fallback.clone(),
            ..Self::default()
        };
        for (root, origins) in &other.fallback {
            merge(result.fallback.entry(root.clone()).or_default(), origins);
        }
        // Absence is an unknown typed value, not unreachable or numeric zero.
        for (key, left) in &self.cells {
            let right = other.number(key, left.domain.ty())?;
            result.set(key.clone(), left.join(&right, widen)?);
        }
        for (key, right) in &other.cells {
            if !self.cells.contains_key(key) {
                let unknown = self.number(key, right.domain.ty())?;
                result.set(key.clone(), unknown.join(right, widen)?);
            }
        }
        Ok(result)
    }
}
