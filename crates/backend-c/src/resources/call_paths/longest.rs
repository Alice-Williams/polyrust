//! Checked reverse-topological summation over exact callable references.
use crate::ast::CFunctionRef;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) fn bound(
    frames: &BTreeMap<CFunctionRef, u64>,
    edges: &BTreeMap<CFunctionRef, BTreeSet<CFunctionRef>>,
) -> Result<u64, String> {
    if frames.keys().ne(edges.keys()) {
        return Err("C call graph and frame inventory differ".into());
    }
    let mut counts = BTreeMap::new();
    let mut callers: BTreeMap<_, Vec<_>> = BTreeMap::new();
    let mut totals = frames.clone();
    let mut longest_child: BTreeMap<CFunctionRef, u64> = BTreeMap::new();
    let mut pending = VecDeque::new();
    for (function, targets) in edges {
        counts.insert(function.clone(), targets.len());
        if targets.is_empty() {
            pending.push_back(function.clone());
        }
        for target in targets {
            if !frames.contains_key(target) {
                return Err("C call graph target has no defined frame".into());
            }
            callers
                .entry(target.clone())
                .or_default()
                .push(function.clone());
        }
    }
    let mut completed = 0usize;
    while let Some(function) = pending.pop_front() {
        completed = completed
            .checked_add(1)
            .ok_or("C call graph count overflow")?;
        let total = totals[&function];
        for caller in callers.get(&function).into_iter().flatten() {
            let child = longest_child.entry(caller.clone()).or_default();
            *child = (*child).max(total);
            let count = counts
                .get_mut(caller)
                .ok_or("C call graph caller missing")?;
            *count = count
                .checked_sub(1)
                .ok_or("C call graph edge accounting mismatch")?;
            if *count == 0 {
                totals.insert(
                    caller.clone(),
                    frames[caller]
                        .checked_add(*child)
                        .ok_or("C native call-path bound overflow")?,
                );
                pending.push_back(caller.clone());
            }
        }
    }
    if completed != frames.len() {
        return Err("recursive C call graph has no bounded native path".into());
    }
    Ok(totals.values().copied().max().unwrap_or(0))
}
