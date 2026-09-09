//! Bounds describe an observation; they do not replace its current binding identity.
use super::{Key, Selector};

pub(super) fn join_selector(left: &Selector, right: &Selector) -> Option<Selector> {
    if left == right {
        return Some(left.clone());
    }
    match (left, right) {
        (Selector::Element(left), Selector::Element(right)) => left
            .join_selection(right)
            .map(|index| Selector::Element(Box::new(index))),
        _ => None,
    }
}

impl Key {
    pub(in crate::ownership) fn join_observation(&self, other: &Self) -> Option<Self> {
        if self.root != other.root || self.selectors.len() != other.selectors.len() {
            return None;
        }
        Some(Self {
            root: self.root.clone(),
            selectors: self
                .selectors
                .iter()
                .zip(&other.selectors)
                .map(|(left, right)| join_selector(left, right))
                .collect::<Option<_>>()?,
        })
    }
    pub(super) fn observes_prefix(&self, prefix: &Self) -> bool {
        self.root == prefix.root
            && self.selectors.len() >= prefix.selectors.len()
            && self
                .selectors
                .iter()
                .zip(&prefix.selectors)
                .all(|(left, right)| join_selector(left, right).is_some())
    }
}
