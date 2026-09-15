//! Ordinary consumer assertions for the two-owner correspondence certificate.
use crate::{
    exit_consumer, mapping,
    owned_linear::cloning::{CloneOwnedBody, Loan, Owner},
    owned_source::{OwnedBoxConstruction, cloning::OwnedBoxClone},
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::{self as hir, def::DefKind};
use rustc_middle::{mir, ty::TyCtxt};
use std::collections::BTreeMap;

pub(super) fn check(tcx: TyCtxt<'_>) {
    let mut inventory = BTreeMap::new();
    let mut context = mapping::Context {
        tcx,
        clones: 0,
        boxes: 0,
    };
    for owner in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        let name = tcx.def_path_str(owner);
        let proof = CloneOwnedBody::read(tcx, owner);
        let expected = match name.as_str() {
            "method" | "qualified" | "read_original" | "return_original" | "return_clone"
            | "moves_before" | "moves_after" | "interleaved" | "shadow_original"
            | "shadow_clone" | "bounded" => true,
            "no_clone" | "second_clone" | "second_allocation" | "sum" | "nested" | "mutable"
            | "borrowed" | "temporary" | "conditional" | "wrong_payload" | "owner_return"
            | "extra_call" | "field_receiver" => false,
            _ => panic!("unexpected fixture function {name}"),
        };
        if expected && proof.is_err() {
            let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
            for (block, data) in body.basic_blocks.iter_enumerated() {
                eprintln!(
                    "{name} {block:?}: {:?} {:?}",
                    data.statements,
                    data.terminator().kind
                );
            }
        }
        assert_eq!(
            proof.is_ok(),
            expected,
            "{name}: {:?}",
            proof.as_ref().err()
        );
        assert!(inventory.insert(name.clone(), expected).is_none());
        let Ok(proof) = proof else {
            continue;
        };
        assert_eq!(proof.owner(), owner);
        exit_consumer::check(tcx, owner, proof.scope(), proof.exit(), proof.returning());
        assert_eq!(
            matches!(proof.exit(), crate::owned_linear::SourceExit::Return { .. }),
            matches!(name.as_str(), "return_clone" | "return_original")
        );
        assert_eq!(proof.parameter().0.owner.def_id, owner);
        assert_eq!(proof.parameter().1, mir::Local::from_usize(1));
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        for &(_, binding, local, location) in proof.bindings() {
            assert_eq!(binding.owner.def_id, owner);
            assert!(matches!(tcx.hir_node(binding), hir::Node::Pat(_)));
            assert_eq!(
                tcx.typeck(owner).node_type(binding),
                body.local_decls[local].ty
            );
            assert!(location.statement_index <= body.basic_blocks[location.block].statements.len());
        }
        let original_read = matches!(name.as_str(), "read_original" | "return_original");
        assert_eq!(
            proof.read_owner(),
            if original_read {
                Owner::Original
            } else {
                Owner::Cloned
            }
        );
        let explicit = matches!(name.as_str(), "qualified" | "return_original");
        let steps = match proof.loan() {
            Loan::Direct(step) => {
                assert!(!explicit);
                vec![step]
            }
            Loan::Reborrow { initial, argument } => {
                assert!(explicit);
                vec![initial, argument]
            }
        };
        for step in steps {
            let statement = &body.basic_blocks[step.location().block].statements
                [step.location().statement_index];
            let mir::StatementKind::Assign(assignment) = &statement.kind else {
                panic!("borrow assignment")
            };
            assert_eq!(assignment.0, mir::Place::from(step.reference()));
            assert!(
                matches!(assignment.1, mir::Rvalue::Ref(_, mir::BorrowKind::Shared, source) if source == step.source())
            );
        }
        let original_first = matches!(name.as_str(), "interleaved" | "shadow_original");
        assert_eq!(
            proof.drops()[0].0,
            if original_first {
                Owner::Original
            } else {
                Owner::Cloned
            }
        );
        assert_ne!(proof.drops()[0].0, proof.drops()[1].0);
        for &(_, place, location) in proof.drops() {
            let block = &body.basic_blocks[location.block];
            assert_eq!(location.statement_index, block.statements.len());
            assert!(
                matches!(block.terminator().kind, mir::TerminatorKind::Drop { place: actual, .. } if actual == place)
            );
        }
        crate::projections::check(tcx, &body, &proof);
        let (construction, cloning) = proof.into_inputs();
        let bindings = mapping::bindings(mapping::Observe);
        Supports::<OwnedBoxConstruction>::mapping(&bindings)
            .lower(&mut context, construction)
            .unwrap();
        Supports::<OwnedBoxClone>::mapping(&bindings)
            .lower(&mut context, cloning)
            .unwrap();
    }
    assert_eq!(inventory.len(), 24);
    assert_eq!((context.clones, context.boxes), (11, 11));
    #[cfg(clone_flow_proof)]
    crate::owned_linear::cloning::mutations::check(tcx);
    println!("eleven clone bodies and thirteen source rejections passed");
}
