//! Independent nonempty entry/callee inventory; rejection is not proof success.
use crate::{
    mapping,
    owned_linear::calls::CheckedInput,
    owned_source::{OwnedBoxConstruction, local_call::OwnedLocalCall},
    source_capabilities::{Mapping, Supports},
};
use crate::{owned_linear::calls::OwnedCallGraph, owned_source::local_call::CallRole};
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use std::collections::BTreeSet;

pub(super) fn check(tcx: TyCtxt<'_>) {
    let expected = [
        (
            "source::via_producer",
            "source::produce",
            CallRole::Producer,
        ),
        (
            "source::via_consumer",
            "source::consume",
            CallRole::Consumer,
        ),
        ("source::via_relay", "source::relay", CallRole::Relay),
        ("source::aliased", "source::produce", CallRole::Producer),
        (
            "source::via_producer_return",
            "source::produce_return",
            CallRole::Producer,
        ),
        (
            "source::via_consumer_return",
            "source::consume_return",
            CallRole::Consumer,
        ),
        (
            "source::via_relay_return",
            "source::relay_return",
            CallRole::Relay,
        ),
        (
            "producer_moved",
            "source::produce_moved",
            CallRole::Producer,
        ),
        (
            "consumer_direct",
            "source::consume_direct",
            CallRole::Consumer,
        ),
        ("relay_direct", "source::relay_direct", CallRole::Relay),
    ];
    let mut accepted = BTreeSet::new();
    let mut count = 0;
    let bindings = mapping::bindings(mapping::Observe);
    let mut context = mapping::Context {
        tcx,
        calls: 0,
        boxes: 0,
    };
    for owner in tcx.hir_body_owners() {
        if tcx.def_kind(owner) != DefKind::Fn {
            continue;
        }
        count += 1;
        let path = tcx.def_path_str(owner);
        let expected = expected.iter().find(|(entry, _, _)| *entry == path);
        let result = OwnedCallGraph::read(tcx, owner);
        if let Some((_, leaf, role)) = expected {
            let graph = result.unwrap_or_else(|e| panic!("{path}: {e:?}"));
            assert_eq!(graph.entry().owner(), owner);
            assert_eq!(graph.leaf().owner(), graph.call().callee());
            assert_eq!(tcx.def_path_str(graph.leaf().owner()), *leaf);
            assert_eq!(graph.role(), *role);
            assert!(accepted.insert(path.to_owned()));
            assert_eq!(graph.entry().parameter().1.as_usize(), 1);
            assert_eq!(graph.leaf().parameter().1.as_usize(), 1);
            super::projections::check(tcx, graph.entry());
            super::projections::check(tcx, graph.leaf());
            for body in graph.into_bodies() {
                for input in body.into_inputs() {
                    match input {
                        CheckedInput::Construction(input) => {
                            Supports::<OwnedBoxConstruction>::mapping(&bindings)
                                .lower(&mut context, input)
                                .unwrap();
                        }
                        CheckedInput::LocalCall(input) => {
                            Supports::<OwnedLocalCall>::mapping(&bindings)
                                .lower(&mut context, input)
                                .unwrap();
                        }
                    }
                }
            }
        } else {
            assert!(result.is_err(), "unexpectedly admitted {path}");
        }
    }
    assert_eq!(accepted.len(), expected.len());
    assert_eq!(count, 38);
    assert_eq!(context.calls, 10);
    assert_eq!(context.boxes, 10);
    #[cfg(owned_call_graph_proof)]
    crate::owned_linear::calls::mutations::check(tcx);
    println!("ten closed graphs and twenty-eight rejected entry bodies passed");
}
