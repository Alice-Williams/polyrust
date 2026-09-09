//! Joins coalesce current identities while retaining every arm's domain and losses.
use super::{E, Key, Number, State, merge};

impl<'a> State<'a> {
    pub(super) fn observed_number(&self, key: &Key) -> Result<Option<Number<'a>>, E> {
        let mut result: Option<Number<'a>> = None;
        for (candidate, value) in &self.cells {
            if key.join_observation(candidate).is_some() {
                result = Some(match result {
                    None => value.clone(),
                    Some(old) => old.join(value, false)?,
                });
            }
        }
        Ok(result)
    }

    fn merge_observation(&mut self, mut key: Key, mut value: Number<'a>) -> Result<(), E> {
        loop {
            let existing = self.cells.keys().find_map(|old| {
                old.join_observation(&key)
                    .map(|joined| (old.clone(), joined))
            });
            let Some((old_key, joined_key)) = existing else {
                break;
            };
            let old = self.cells.remove(&old_key).expect("observed key");
            key = joined_key;
            value = old.join(&value, false)?;
        }
        self.cells.insert(key, value);
        Ok(())
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
        for (left_key, left) in &self.cells {
            let mut matched = false;
            for (right_key, right) in &other.cells {
                if let Some(key) = left_key.join_observation(right_key) {
                    matched = true;
                    result.merge_observation(key, left.join(right, widen)?)?;
                }
            }
            if !matched {
                // Absence is unknown, never an unreachable arm or numeric zero.
                let unknown = other.number(left_key, left.domain.ty())?;
                result.merge_observation(left_key.clone(), left.join(&unknown, widen)?)?;
            }
        }
        for (right_key, right) in &other.cells {
            if !self
                .cells
                .keys()
                .any(|left| left.join_observation(right_key).is_some())
            {
                let unknown = self.number(right_key, right.domain.ty())?;
                result.merge_observation(right_key.clone(), unknown.join(right, widen)?)?;
            }
        }
        Ok(result)
    }
}
