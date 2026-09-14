//! Weighted call-graph arithmetic uses registered identities, not string names.
use super::longest;
use crate::ast::*;
use portable_codegen::RelativeOutputPath;
use std::collections::{BTreeMap, BTreeSet};

fn functions(count: usize) -> Vec<CFunctionRef> {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("tests/paths.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    (0..count)
        .map(|index| {
            registry
                .register_function(
                    &file,
                    CDeclarationKey {
                        name: CIdentifier::new(&format!("function{index}")).unwrap(),
                        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
                    },
                    CFunctionType::new(
                        CReturnType::Value(
                            CReturnValue::new(CObjectType::scalar(CScalarType::I32)).unwrap(),
                        ),
                        vec![],
                    ),
                )
                .unwrap()
        })
        .collect()
}

fn bound(weights: &[u64], graph: &[Vec<usize>]) -> Result<u64, String> {
    let functions = functions(weights.len());
    let frames = functions
        .iter()
        .cloned()
        .zip(weights.iter().copied())
        .collect();
    let edges = functions
        .iter()
        .cloned()
        .zip(
            graph
                .iter()
                .map(|row| row.iter().map(|index| functions[*index].clone()).collect()),
        )
        .collect();
    longest::bound(&frames, &edges)
}

#[test]
fn diamonds_take_the_longest_live_child_not_the_sum_of_sequential_calls() {
    assert_eq!(
        bound(
            &[100, 200, 300, 50],
            &[vec![1, 2, 1], vec![3], vec![3], vec![]]
        )
        .unwrap(),
        450
    );
    assert_eq!(
        bound(&[100, 200, 300, 50], &[vec![], vec![], vec![], vec![]]).unwrap(),
        300
    );
}

#[test]
fn cycles_and_missing_function_inventories_are_rejected() {
    for graph in [
        vec![vec![0], vec![]],
        vec![vec![1], vec![0]],
        vec![vec![], vec![1]],
    ] {
        assert!(bound(&[10, 20], &graph).unwrap_err().contains("recursive"));
    }
    let functions = functions(2);
    let frames = BTreeMap::from([(functions[0].clone(), 10)]);
    let absent = BTreeMap::new();
    assert!(
        longest::bound(&frames, &absent)
            .unwrap_err()
            .contains("inventory")
    );
    let unresolved =
        BTreeMap::from([(functions[0].clone(), BTreeSet::from([functions[1].clone()]))]);
    assert!(
        longest::bound(&frames, &unresolved)
            .unwrap_err()
            .contains("defined frame")
    );
}

#[test]
fn exact_stack_limit_one_over_and_checked_addition_overflow() {
    use super::super::{Measurements, policy};
    let limit = policy::CResourceKind::NativeFrameBytes.limit();
    for over in [0, 1] {
        let measured = Measurements {
            frame_bound: bound(&[limit - 100, 100 + over], &[vec![1], vec![]]).unwrap(),
            ..Measurements::default()
        };
        let errors = policy::check(
            &measured,
            portable_diagnostics::SourceRef::logical(["call-path"]),
        );
        assert_eq!(errors.is_empty(), over == 0);
        if over == 1 {
            assert_eq!(errors[0].kind(), policy::CResourceKind::NativeFrameBytes);
        }
    }
    assert!(
        bound(&[u64::MAX, 1], &[vec![1], vec![]])
            .unwrap_err()
            .contains("overflow")
    );
}

fn reference(weights: &[u64], graph: &[Vec<usize>], node: usize) -> u64 {
    weights[node]
        + graph[node]
            .iter()
            .map(|child| reference(weights, graph, *child))
            .max()
            .unwrap_or(0)
}

#[test]
fn seeded_dags_match_independent_path_enumeration() {
    let mut seed = 0x1ee7_0531_u32;
    for _ in 0..256 {
        let mut weights = vec![0; 8];
        let mut graph = vec![vec![]; 8];
        for (node, edges) in graph.iter_mut().enumerate() {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            weights[node] = u64::from(seed % 100 + 1);
            for child in node + 1..8 {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                if seed.is_multiple_of(3) {
                    edges.push(child);
                }
            }
        }
        let expected = (0..8)
            .map(|node| reference(&weights, &graph, node))
            .max()
            .unwrap();
        assert_eq!(bound(&weights, &graph).unwrap(), expected);
    }
}
