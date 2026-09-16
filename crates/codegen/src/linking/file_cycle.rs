//! Deterministic DFS without a native call frame per generated file.
use crate::TargetFileId;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
enum State {
    Visiting(usize),
    Done,
}

enum Step {
    Enter(TargetFileId),
    Exit(TargetFileId),
}

pub(super) fn find(
    graph: &BTreeMap<TargetFileId, BTreeSet<TargetFileId>>,
) -> Option<Vec<TargetFileId>> {
    let mut states = BTreeMap::new();
    let mut active = Vec::new();
    let mut pending = Vec::new();
    for &root in graph.keys() {
        pending.push(Step::Enter(root));
        while let Some(step) = pending.pop() {
            match step {
                Step::Exit(node) => {
                    let finished = active.pop();
                    debug_assert_eq!(finished, Some(node));
                    states.insert(node, State::Done);
                }
                Step::Enter(node) => {
                    match states.get(&node) {
                        Some(State::Done) => continue,
                        Some(State::Visiting(index)) => return Some(active[*index..].to_vec()),
                        None => {}
                    }
                    states.insert(node, State::Visiting(active.len()));
                    active.push(node);
                    pending.push(Step::Exit(node));
                    if let Some(edges) = graph.get(&node) {
                        pending.extend(edges.iter().rev().copied().map(Step::Enter));
                    }
                }
            }
        }
    }
    None
}

/// Reject rather than accepting a partially explored selective-policy graph.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Error {
    Forbidden,
    Budget,
}

pub(super) fn check(
    graph: &BTreeMap<TargetFileId, BTreeSet<TargetFileId>>,
    permits: impl FnMut(&[TargetFileId]) -> bool,
) -> Result<(), Error> {
    check_with_budget(graph, permits, 100_000)
}

fn check_with_budget(
    graph: &BTreeMap<TargetFileId, BTreeSet<TargetFileId>>,
    mut permits: impl FnMut(&[TargetFileId]) -> bool,
    mut remaining: usize,
) -> Result<(), Error> {
    let Some(mut first) = find(graph) else {
        return Ok(());
    };
    let minimum = first
        .iter()
        .enumerate()
        .min_by_key(|(_, id)| **id)
        .unwrap()
        .0;
    first.rotate_left(minimum);
    if !permits(&first) {
        return Err(Error::Forbidden);
    }

    // A policy may distinguish overlapping cycles, not merely components or
    // DFS back edges. Enumerate every simple directed cycle once: its smallest
    // vertex is the root, and every other path vertex must be larger.
    // This can be exponential; bound all root/edge exploration, failing closed.
    let empty = BTreeSet::new();
    for &root in graph.keys() {
        remaining = remaining.checked_sub(1).ok_or(Error::Budget)?;
        let mut path = vec![root];
        let mut active = BTreeSet::from([root]);
        let mut frames = vec![graph.get(&root).unwrap_or(&empty).iter()];
        while let Some(frame) = frames.last_mut() {
            if let Some(&next) = frame.next() {
                remaining = remaining.checked_sub(1).ok_or(Error::Budget)?;
                if next == root {
                    if path != first && !permits(&path) {
                        return Err(Error::Forbidden);
                    }
                } else if next > root && active.insert(next) {
                    path.push(next);
                    frames.push(graph.get(&next).unwrap_or(&empty).iter());
                }
            } else {
                frames.pop();
                active.remove(&path.pop().expect("one path vertex per search frame"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_chains_and_cycles_do_not_use_recursive_call_frames() {
        let id = TargetFileId::from_index;
        let mut graph: BTreeMap<_, _> = (0..20_000)
            .map(|index| (id(index), BTreeSet::from([id(index + 1)])))
            .collect();
        graph.insert(id(20_000), BTreeSet::new());
        assert_eq!(find(&graph), None);
        graph.get_mut(&id(20_000)).unwrap().insert(id(19_998));
        assert_eq!(find(&graph), Some(vec![id(19_998), id(19_999), id(20_000)]));
    }

    #[test]
    fn completed_diamond_branches_are_not_cycles() {
        let id = TargetFileId::from_index;
        let graph = BTreeMap::from([
            (id(0), BTreeSet::from([id(1), id(2)])),
            (id(1), BTreeSet::from([id(2)])),
            (id(2), BTreeSet::new()),
        ]);
        assert_eq!(find(&graph), None);
        assert_eq!(
            find(&BTreeMap::from([(id(0), BTreeSet::from([id(0)]))])),
            Some(vec![id(0)])
        );
    }

    #[test]
    fn selective_policy_checks_disconnected_and_overlapping_cycles() {
        let id = TargetFileId::from_index;
        for graph in [
            BTreeMap::from([
                (id(0), BTreeSet::from([id(1)])),
                (id(1), BTreeSet::from([id(0)])),
                (id(2), BTreeSet::from([id(3)])),
                (id(3), BTreeSet::from([id(2)])),
            ]),
            BTreeMap::from([
                (id(0), BTreeSet::from([id(1)])),
                (id(1), BTreeSet::from([id(0), id(2)])),
                (id(2), BTreeSet::from([id(0)])),
            ]),
        ] {
            assert_eq!(
                check(&graph, |cycle| cycle == [id(0), id(1)]),
                Err(Error::Forbidden)
            );
            assert_eq!(check(&graph, |_| true), Ok(()));
            assert_eq!(check_with_budget(&graph, |_| true, 1), Err(Error::Budget));
        }
    }

    #[test]
    fn every_cycle_matches_independent_enumeration_of_all_four_vertex_graphs() {
        let id = TargetFileId::from_index;
        // Independently enumerate all ordered subsets, retaining one rotation
        // of each possible directed cycle. This is not DFS over a test graph.
        let mut possible = BTreeSet::new();
        for a in 0..4 {
            possible.insert(vec![a]);
            for b in (a + 1)..4 {
                possible.insert(vec![a, b]);
                for c in (a + 1)..4 {
                    if b == c {
                        continue;
                    }
                    possible.insert(vec![a, b, c]);
                    for d in (a + 1)..4 {
                        if d != b && d != c {
                            possible.insert(vec![a, b, c, d]);
                        }
                    }
                }
            }
        }
        let possible: Vec<_> = possible
            .into_iter()
            .map(|cycle| {
                let mask = cycle
                    .iter()
                    .zip(cycle.iter().cycle().skip(1))
                    .fold(0u32, |mask, (a, b)| mask | (1 << (a * 4 + b)));
                (mask, cycle.into_iter().map(id).collect::<Vec<_>>())
            })
            .collect();
        assert_eq!(possible.len(), 24);
        for bits in 0..(1u32 << 16) {
            let graph = (0..4)
                .map(|a| {
                    (
                        id(a),
                        (0..4)
                            .filter(|b| bits & (1 << (a * 4 + b)) != 0)
                            .map(id)
                            .collect(),
                    )
                })
                .collect();
            let expected: BTreeSet<_> = possible
                .iter()
                .filter(|(mask, _)| bits & mask == *mask)
                .map(|(_, cycle)| cycle.clone())
                .collect();
            let mut actual = BTreeSet::new();
            assert_eq!(
                check(&graph, |cycle| {
                    assert!(actual.insert(cycle.to_vec()), "duplicate witness");
                    true
                }),
                Ok(())
            );
            assert_eq!(actual, expected, "graph {bits}");
        }
    }
}
