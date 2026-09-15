//! Independent parameter/drop/source-order oracles for the closed record form.
use super::mapping::{self, Context, RecordMapping};
use crate::{
    owned_linear::{
        LinearOwnedBody,
        multiple::{
            MultipleOwnedBody,
            records::{RecordOwnedBody, SourcePlace, Transfer},
        },
    },
    owned_source::{OwnedBoxConstruction, record::OwnedRecordConstruction},
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::def::DefKind;
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::TyCtxt,
};
use std::collections::{BTreeSet, HashMap};

pub(super) fn check(tcx: TyCtxt<'_>) {
    let mut seen = BTreeSet::new();
    let mut nominal = HashMap::new();
    for owner in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        let name = tcx.def_path_str(owner);
        #[cfg(owned_fields_proof)]
        if name == "first" {
            crate::owned_linear::multiple::records::mutations::check(tcx, owner);
        }
        if name == "tail" {
            assert!(RecordOwnedBody::read(tcx, owner).is_err());
            crate::compatibility::tail(tcx, owner);
            assert!(seen.insert(name));
            continue;
        }
        if [
            "repeated",
            "inline",
            "after",
            "whole",
            "remaining",
            "nested_scope",
            "borrowed",
            "conditional",
            "updated",
            "assigned",
            "generic",
            "custom",
            "nested_record",
            "tuple",
        ]
        .contains(&name.as_str())
        {
            assert!(
                RecordOwnedBody::read(tcx, owner).is_err(),
                "admitted {name}"
            );
            assert!(seen.insert(name));
            continue;
        }
        let (constructors, drops, read, fields): (&[usize], &[usize], usize, &[usize]) =
            match name.as_str() {
                "first" => (&[0, 1, 2], &[0, 1, 2], 0, &[0, 1, 2]),
                "reversed" => (&[0, 1, 2], &[1, 0, 2], 1, &[2, 0, 1]),
                "multiple" => (&[3, 0, 1, 2], &[3, 0, 2, 1], 0, &[1, 2, 0]),
                "all" => (&[0, 1, 2], &[2, 1, 0], 2, &[0, 1, 2]),
                "last" => (&[0, 1, 2], &[2, 0, 1], 2, &[0, 1, 2]),
                "shadow" => (&[0, 1, 2], &[1, 0, 2], 1, &[2, 1, 0]),
                "single" | "left::same" | "right::same" => (&[0], &[0], 0, &[0]),
                _ => panic!("unasserted field fixture {name}"),
            };
        let proof = RecordOwnedBody::read(tcx, owner).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let parameters: Vec<_> = body.args_iter().collect();
        assert_eq!(
            body.phase,
            mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup)
        );
        assert_eq!(proof.chains().len(), constructors.len());
        assert!(LinearOwnedBody::read(tcx, owner).is_err());
        assert!(MultipleOwnedBody::read(tcx, owner).is_err());
        let (record_binding, record_place, aggregate) = proof.aggregate();
        let StatementKind::Assign(pair) =
            &body.basic_blocks[aggregate.block].statements[aggregate.statement_index].kind
        else {
            panic!("record assignment");
        };
        let Rvalue::Aggregate(_, operands) = &pair.1 else {
            panic!("aggregate");
        };
        assert_eq!(pair.0, record_place);
        let mut ends = HashMap::new();
        let mut staging = HashMap::new();
        for (chain, expected) in proof.chains().iter().zip(constructors) {
            assert_eq!(chain.parameter().1, parameters[*expected]);
            assert_eq!(chain.drop_scope(), proof.scopes().read_scope());
            assert_eq!(chain.transfers().len() + 1, chain.places().len());
            for ((source, place), transfer) in chain.places().iter().skip(1).zip(chain.transfers())
            {
                match (source, transfer) {
                    (
                        SourcePlace::Field { record, index, .. },
                        Transfer::IntoField {
                            staging: temporary,
                            movement,
                            aggregate: event,
                        },
                    ) => {
                        assert_eq!(*record, record_binding);
                        assert_eq!(*event, aggregate);
                        assert_eq!(place.local, record_place.local);
                        assert!(
                            matches!(place.projection.as_slice(), [mir::ProjectionElem::Field(i, _)] if i == index)
                        );
                        assert!(matches!(&operands[*index], Operand::Move(p) if p == temporary));
                        assert!(staging.insert(index.as_usize(), *movement).is_none());
                    }
                    (SourcePlace::Local(_), Transfer::Move { location }) => {
                        let StatementKind::Assign(pair) = &body.basic_blocks[location.block]
                            .statements[location.statement_index]
                            .kind
                        else {
                            panic!("move assignment");
                        };
                        assert_eq!(pair.0, *place);
                        assert!(matches!(pair.1, Rvalue::Use(Operand::Move(_), _)));
                    }
                    _ => panic!("wrong source/transfer role"),
                }
            }
            let (source, actual) = *chain.places().last().unwrap();
            let TerminatorKind::Drop { place, .. } = body.basic_blocks[chain.drop_location().block]
                .terminator()
                .kind
            else {
                panic!("drop");
            };
            assert_eq!(place, actual);
            assert!(ends.insert(source, *expected).is_none());
        }
        assert_eq!(
            proof
                .drop_order()
                .iter()
                .map(|p| ends[p])
                .collect::<Vec<_>>(),
            drops
        );
        assert_eq!(ends[&SourcePlace::Local(proof.scalar_read().0)], read);
        assert_eq!(proof.scopes().blocks().count(), 1);
        assert!(
            proof
                .scopes()
                .bindings()
                .iter()
                .all(|(_, scope)| *scope == proof.scopes().read_scope())
        );
        let (record, chains) = proof.into_operations();
        nominal.insert(name.clone(), record.definition().did());
        assert_eq!(
            record
                .fields()
                .iter()
                .map(|f| f.index().as_usize())
                .collect::<Vec<_>>(),
            fields
        );
        let mut previous = None;
        for index in fields {
            let event = staging[index];
            if let Some(before) = previous {
                // All pinned initializer staging moves share the aggregate block.
                assert_eq!(event.block, aggregate.block);
                assert!(before < event.statement_index);
            }
            previous = Some(event.statement_index);
        }
        let bindings = mapping::bindings(RecordMapping);
        let mut context = Context {
            tcx,
            boxes: 0,
            records: 0,
        };
        Supports::<OwnedRecordConstruction>::mapping(&bindings)
            .lower(&mut context, record)
            .unwrap();
        for chain in chains {
            Supports::<OwnedBoxConstruction>::mapping(&bindings)
                .lower(&mut context, chain.into_construction())
                .unwrap();
        }
        assert_eq!(context.records, 1);
        assert_eq!(context.boxes, constructors.len());
        assert!(seen.insert(name));
    }
    assert_eq!(seen.len(), 24);
    assert_ne!(nominal["left::same"], nominal["right::same"]);
    assert_eq!(
        tcx.item_name(nominal["left::same"]),
        tcx.item_name(nominal["right::same"])
    );
    println!("owned field correspondence passed");
}
