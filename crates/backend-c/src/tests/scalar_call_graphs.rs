//! Deterministic model checking of the derived closure, independent of sorting.
use super::{ScalarCalls, fixture::Fixture};

fn expected(graph: &[Vec<usize>], current: usize, path: &mut Vec<usize>) -> bool {
    if path.contains(&current) {
        return false;
    }
    path.push(current);
    let result = graph[current]
        .iter()
        .all(|next| expected(graph, *next, path));
    path.pop();
    result
}

fn check(graph: &[Vec<usize>]) {
    let f = Fixture::new(&vec![0; graph.len()]);
    let bodies = graph
        .iter()
        .enumerate()
        .map(|(index, targets)| {
            let mut statements: Vec<_> = targets
                .iter()
                .map(|target| {
                    f.statements(index)
                        .discard(f.call(*target, vec![]))
                        .unwrap()
                })
                .collect();
            statements.extend(f.returning(index, f.literal(1)));
            statements
        })
        .collect();
    let source = f.source(bodies);
    f.registry
        .check_context(std::slice::from_ref(&source))
        .unwrap();
    let summary = ScalarCalls::derive(&[source]);
    for (index, function) in f.functions.iter().enumerate() {
        assert_eq!(
            summary.accepts(&f.values().direct(function.clone()).unwrap()),
            expected(graph, index, &mut vec![]),
            "graph={graph:?}, root={index}"
        );
    }
}

#[test]
fn diamond_repeated_edges_and_disconnected_components_match_a_reference_walk() {
    check(&[vec![1, 2, 1], vec![3], vec![3], vec![]]);
    check(&[vec![1, 2], vec![3], vec![2], vec![], vec![]]);
    check(&[vec![], vec![2], vec![1], vec![0], vec![3, 2]]);
}

#[test]
fn seeded_small_graphs_match_an_independent_path_cycle_oracle() {
    let mut seed = 0x83d2_7a91_u32;
    for _ in 0..256 {
        let mut graph = vec![vec![]; 6];
        for edges in &mut graph {
            for target in 0..6 {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                if seed.is_multiple_of(5) {
                    edges.push(target);
                }
            }
        }
        check(&graph);
    }
}

#[test]
fn long_dependency_chain_uses_an_iterative_closure() {
    let count = 512;
    let f = Fixture::new(&vec![0; count]);
    let source = f.source(
        (0..count)
            .map(|index| {
                f.returning(
                    index,
                    if index + 1 == count {
                        f.literal(1)
                    } else {
                        f.call(index + 1, vec![])
                    },
                )
            })
            .collect(),
    );
    let summary = ScalarCalls::derive(&[source]);
    assert!(
        f.functions
            .iter()
            .all(|function| summary.accepts(&f.values().direct(function.clone()).unwrap()))
    );
}
