//! Actual direct-call edges and per-definition live-frame accounting.
#[path = "call_paths/longest.rs"]
mod longest;
#[cfg(test)]
#[path = "../tests/call_path_math.rs"]
mod tests;

use super::{Measurements, frame_bound, profile::Node};
use crate::ast::{CCallableKind, CDefinitionKind, CFileItem, CFunctionRef, CValueKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Default)]
struct Storage {
    nodes: u64,
    automatic_bytes: u64,
    automatic_objects: u64,
    value_bytes: u64,
}

impl Storage {
    fn from(value: &Measurements) -> Self {
        Self {
            nodes: value.nodes,
            automatic_bytes: value.automatic_bytes,
            automatic_objects: value.automatic_objects,
            value_bytes: value.value_bytes,
        }
    }
    fn add(self, other: Self) -> Result<Self, String> {
        Ok(Self {
            nodes: self
                .nodes
                .checked_add(other.nodes)
                .ok_or("C frame node overflow")?,
            automatic_bytes: self
                .automatic_bytes
                .checked_add(other.automatic_bytes)
                .ok_or("C automatic byte overflow")?,
            automatic_objects: self
                .automatic_objects
                .checked_add(other.automatic_objects)
                .ok_or("C automatic object overflow")?,
            value_bytes: self
                .value_bytes
                .checked_add(other.value_bytes)
                .ok_or("C value byte overflow")?,
        })
    }
    fn subtract(self, other: Self) -> Result<Self, String> {
        Ok(Self {
            nodes: self
                .nodes
                .checked_sub(other.nodes)
                .ok_or("C frame node accounting mismatch")?,
            automatic_bytes: self
                .automatic_bytes
                .checked_sub(other.automatic_bytes)
                .ok_or("C automatic byte accounting mismatch")?,
            automatic_objects: self
                .automatic_objects
                .checked_sub(other.automatic_objects)
                .ok_or("C automatic object accounting mismatch")?,
            value_bytes: self
                .value_bytes
                .checked_sub(other.value_bytes)
                .ok_or("C value byte accounting mismatch")?,
        })
    }
    fn bound(self) -> Result<u64, String> {
        frame_bound(&Measurements {
            nodes: self.nodes,
            automatic_bytes: self.automatic_bytes,
            automatic_objects: self.automatic_objects,
            value_bytes: self.value_bytes,
            ..Measurements::default()
        })
    }
}

#[derive(Default)]
pub(super) struct Inventory {
    current: Option<CFunctionRef>,
    frames: BTreeMap<CFunctionRef, Storage>,
    edges: BTreeMap<CFunctionRef, BTreeSet<CFunctionRef>>,
    library_frames: BTreeMap<CFunctionRef, std::num::NonZeroU64>,
}

impl Inventory {
    pub(super) fn observe(
        &mut self,
        node: Node<'_>,
        before: &Measurements,
        after: &Measurements,
    ) -> Result<(), String> {
        if let Node::Item(item) = node {
            self.current = None;
            if let CFileItem::Definition(definition) = item
                && let CDefinitionKind::Function { function, .. } = definition.kind()
            {
                if self
                    .frames
                    .insert(function.clone(), Storage::default())
                    .is_some()
                {
                    return Err("duplicate C frame definition".into());
                }
                self.edges.insert(function.clone(), BTreeSet::new());
                self.current = Some(function.clone());
            }
        }
        if let Some(function) = &self.current {
            let frame = self
                .frames
                .get_mut(function)
                .ok_or("missing C frame owner")?;
            *frame = frame.add(Storage::from(after).subtract(Storage::from(before))?)?;
        }
        let call = match node {
            Node::Value(value) => match value.kind() {
                CValueKind::Call(call) => Some(call),
                _ => None,
            },
            Node::Effect(effect) => Some(effect.call()),
            _ => None,
        };
        if let Some(call) = call {
            let owner = self.current.as_ref().ok_or("C call has no frame owner")?;
            match call.callable().kind() {
                CCallableKind::Direct(target) => {
                    self.edges
                        .get_mut(owner)
                        .ok_or("missing C call inventory")?
                        .insert(target.as_ref().clone());
                }
                CCallableKind::Known(callable) => {
                    let reserve = callable
                        .stack_bound_bytes()
                        .ok_or("C known call lacks a supported nonzero stack reserve")?;
                    self.library_frames
                        .entry(owner.clone())
                        .and_modify(|old| *old = (*old).max(reserve))
                        .or_insert(reserve);
                }
                _ => return Err("C stack profile requires an admitted call".into()),
            }
        }
        Ok(())
    }

    pub(super) fn finish(
        self,
        total: &Measurements,
        registry: &crate::ast::CRegistry,
    ) -> Result<(BTreeMap<CFunctionRef, u64>, u64), String> {
        let bodies = self
            .frames
            .values()
            .try_fold(Storage::default(), |sum, frame| sum.add(*frame))?;
        let shared = Storage::from(total).subtract(bodies)?;
        // Charge non-body syntax to every frame conservatively. This preserves
        // the existing single-function policy, including its fixed allowances.
        let library_max = self
            .library_frames
            .values()
            .map(|value| value.get())
            .max()
            .unwrap_or(0);
        let frames = self
            .frames
            .into_iter()
            .map(|(function, frame)| {
                // Sequential standard calls do not have simultaneous frames.
                // Charging the reserve to its caller even on a separate direct
                // call path is conservative; no lifetime overlap is assumed.
                let reserve = self
                    .library_frames
                    .get(&function)
                    .map_or(0, |value| value.get());
                let bound = frame
                    .add(shared)?
                    .bound()?
                    .checked_add(reserve)
                    .ok_or("C library stack reserve overflow")?;
                Ok((function, bound))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        if frames.is_empty() {
            return Ok((frames, 0));
        }
        let mut weighted = frames.clone();
        let mut edges = self.edges.clone();
        for target in self.edges.values().flatten() {
            if frames.contains_key(target) || weighted.contains_key(target) {
                continue;
            }
            let proof = registry
                .imported_function(target)
                .map_err(|_| "C external call lacks certified frame evidence")?;
            let bound = proof.stack_bound_bytes();
            if bound == 0 {
                return Err("C dependency cannot have a zero-cost stack proof".into());
            }
            weighted.insert(target.clone(), bound);
            edges.insert(target.clone(), BTreeSet::new());
        }
        let worst = longest::bound(&weighted, &edges)?;
        // Preserve the older, stricter whole-unit no-call policy unchanged.
        let bound = if self.edges.values().all(BTreeSet::is_empty) {
            frame_bound(total)?
                .checked_add(library_max)
                .ok_or("C whole-unit library stack reserve overflow")?
        } else {
            worst
        };
        Ok((frames, bound))
    }
}
