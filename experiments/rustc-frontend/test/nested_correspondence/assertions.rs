//! Ordinary consumers inspect only query-only correspondence projections.
use super::{fixtures, projections, trace::Trace};
use crate::{
    owned_linear::{multiple::records::RecordOwnedBody, nested::NestedOwnedBody},
    owned_source::{
        OwnedBoxConstruction, nested_record::OwnedNestedRecordConstruction,
        record::OwnedRecordConstruction,
    },
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::{self as hir, def::DefKind};
use rustc_middle::{
    mir::{self, Operand, Rvalue},
    ty::TyCtxt,
};
use std::collections::BTreeSet;

pub(super) fn check(tcx: TyCtxt<'_>) {
    let mut seen = BTreeSet::new();
    for owner in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        let name = tcx.def_path_str(owner);
        let expected = fixtures::expected(&name);
        let proof = NestedOwnedBody::read(tcx, owner).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert!(RecordOwnedBody::read(tcx, owner).is_err());
        assert_eq!(proof.owner(), owner);
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let trace = Trace::read(&body);
        crate::exit_consumer::check(
            tcx,
            owner,
            proof.scopes().read_scope(),
            proof.exit(),
            proof.returning(),
        );
        assert_eq!(trace.returning, proof.returning());
        let hir::ExprKind::Block(root, None) = tcx.hir_body_owned_by(owner).value.kind else {
            panic!("root");
        };
        let source_bindings: Vec<_> = root
            .stmts
            .iter()
            .filter_map(|s| match s.kind {
                hir::StmtKind::Let(decl) => {
                    let hir::PatKind::Binding(_, id, _, None) = decl.pat.kind else {
                        panic!("binding");
                    };
                    Some(id)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            proof
                .bindings()
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            source_bindings
        );
        assert_eq!(
            proof
                .scopes()
                .bindings()
                .iter()
                .map(|(id, scope)| {
                    assert_eq!(*scope, root.hir_id);
                    *id
                })
                .collect::<Vec<_>>(),
            source_bindings
        );
        assert_eq!(proof.constructions().len(), 3);
        assert_eq!(proof.movements().len(), expected.moves);
        projections::events(tcx, &proof, &body, &trace, name == "reversed");
        assert_eq!(proof.leaves().len(), 3);
        for ((leaf, &constructor), &(binding, path)) in proof
            .leaves()
            .iter()
            .zip(expected.drops)
            .zip(expected.paths)
        {
            assert_eq!(leaf.constructor(), source_bindings[constructor]);
            assert_eq!(leaf.source().binding(), source_bindings[binding]);
            assert_eq!(
                leaf.source()
                    .fields()
                    .iter()
                    .map(|f| f.index().as_usize())
                    .collect::<Vec<_>>(),
                path
            );
            projections::path(
                tcx,
                &body,
                leaf.source(),
                proof.bindings()[binding].1,
                leaf.actual(),
            );
        }
        assert_eq!(
            proof
                .cleanup()
                .iter()
                .map(|cleanup| (cleanup.location(), cleanup.actual()))
                .collect::<Vec<_>>(),
            trace.drops
        );
        projections::cleanup(tcx, &proof, &body, &name);
        let selected = proof
            .leaves()
            .iter()
            .find(|leaf| {
                leaf.source().binding() == proof.scalar_read().0
                    && leaf.source().fields().is_empty()
            })
            .unwrap();
        assert_eq!(selected.constructor(), source_bindings[expected.read]);
        let (read, value) = trace.definition(mir::Place::from(mir::RETURN_PLACE));
        assert_eq!(proof.scalar_read().1, read);
        let Rvalue::Use(Operand::Copy(pointer), _) = value else {
            panic!("read");
        };
        assert_eq!(pointer.projection.as_slice(), &[mir::ProjectionElem::Deref]);
        let (cast, value) = trace.definition(mir::Place::from(pointer.local));
        let Rvalue::Cast(mir::CastKind::Transmute, Operand::Copy(source), target) = value else {
            panic!("pointer");
        };
        assert_eq!(source.local, selected.actual().local);
        projections::pointer(tcx, &body, *source, *target, pointer.local);
        trace.before(proof.movements().last().unwrap().location(), cast);
        trace.before(cast, read);
        for leaf in proof.leaves() {
            trace.before(read, leaf.cleanup_location());
            trace.before(leaf.cleanup_location(), proof.returning());
        }
        let (inner, outer) = proof.record_inputs();
        assert_eq!(inner.owner(), owner);
        assert_eq!(outer.owner(), owner);
        let definitions = (inner.definition().did(), outer.layout().definition().did());
        let (boxes, inner, outer) = proof.into_operations();
        let mut flat = super::flat_mapping::Context {
            tcx,
            boxes: 0,
            records: 0,
        };
        let flat_bindings = super::flat_mapping::bindings(super::flat_mapping::RecordMapping);
        for input in boxes {
            Supports::<OwnedBoxConstruction>::mapping(&flat_bindings)
                .lower(&mut flat, input)
                .unwrap();
        }
        assert_eq!(
            Supports::<OwnedRecordConstruction>::mapping(&flat_bindings)
                .lower(&mut flat, inner)
                .unwrap(),
            definitions.0
        );
        assert_eq!((flat.boxes, flat.records), (3, 1));
        let mut nested = super::nested_mapping::Context { tcx, records: 0 };
        assert_eq!(
            Supports::<OwnedNestedRecordConstruction>::mapping(&super::nested_mapping::bindings(
                super::nested_mapping::RecordMapping
            ))
            .lower(&mut nested, outer)
            .unwrap(),
            definitions.1
        );
        assert_eq!(nested.records, 1);
        #[cfg(owned_nested_proof)]
        crate::owned_linear::nested::mutations::check(tcx, owner);
        assert!(seen.insert(name));
    }
    assert_eq!(seen.len(), 14);
    println!("nested consumer projections passed");
    println!("nested corruption inventory passed");
}
