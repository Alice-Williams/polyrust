//! Pin the selected fixtures' actual shared-borrow edges and ordered drops.
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::{self, TyCtxt},
};

pub(super) fn check<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, explicit: bool) {
    let call = |definition| {
        let matches: Vec<_> = body.basic_blocks.iter_enumerated().filter_map(|(block, data)| {
            let TerminatorKind::Call { func, args, destination, .. } = &data.terminator().kind else { return None; };
            matches!(func.ty(&body.local_decls, tcx).kind(), ty::FnDef(actual, _) if *actual == definition).then_some((block, args, *destination))
        }).collect();
        assert_eq!(matches.len(), 1);
        matches[0]
    };
    let (_, _, original) = call(tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap());
    let (block, arguments, cloned) = call(tcx.lang_items().clone_fn().unwrap());
    assert_ne!(original, cloned);
    assert_eq!(
        original.ty(&body.local_decls, tcx).ty,
        cloned.ty(&body.local_decls, tcx).ty
    );
    assert_eq!(arguments.len(), 1);
    let Operand::Move(reference) = arguments[0].node else {
        panic!("shared-reference staging")
    };
    assert!(reference.projection.is_empty());
    let reference_type = body.local_decls[reference.local].ty;
    assert!(
        matches!(reference_type.kind(), ty::Ref(_, target, rustc_hir::Mutability::Not) if *target == original.ty(&body.local_decls, tcx).ty)
    );
    let assignments: Vec<_> = body
        .basic_blocks
        .iter_enumerated()
        .flat_map(|(bb, data)| {
            data.statements
                .iter()
                .enumerate()
                .filter_map(move |(index, statement)| {
                    let StatementKind::Assign(assignment) = &statement.kind else {
                        return None;
                    };
                    let (destination, value) = &**assignment;
                    Some((bb, index, *destination, value))
                })
        })
        .collect();
    let shared = |destination| {
        let found: Vec<_> = assignments
            .iter()
            .filter(|(_, _, actual, _)| *actual == destination)
            .collect();
        assert_eq!(found.len(), 1);
        let &(bb, index, _, value) = found[0];
        let Rvalue::Ref(_, mir::BorrowKind::Shared, source) = value else {
            panic!("shared borrow")
        };
        assert_eq!(bb, block);
        (index, *source)
    };
    let (last, source) = shared(reference);
    if explicit {
        assert_eq!(source.projection.len(), 1);
        assert_eq!(source.projection[0], mir::ProjectionElem::Deref);
        let stage = mir::Place::from(source.local);
        assert_ne!(stage, reference);
        assert_eq!(body.local_decls[stage.local].ty, reference_type);
        let (first, borrowed) = shared(stage);
        assert!(first < last);
        assert_eq!(borrowed, original);
    } else {
        assert_eq!(source, original);
    }
    // Fixture CFGs are linear through cleanup; use edges, not block-number order.
    let mut cursor = block;
    let mut seen = std::collections::HashSet::new();
    let mut drops = Vec::new();
    loop {
        assert!(seen.insert(cursor));
        match &body.basic_blocks[cursor].terminator().kind {
            TerminatorKind::Call {
                target: Some(next), ..
            }
            | TerminatorKind::Goto { target: next }
            | TerminatorKind::Assert { target: next, .. } => cursor = *next,
            TerminatorKind::Drop { place, target, .. } => {
                drops.push(*place);
                cursor = *target;
            }
            TerminatorKind::Return => break,
            _ => panic!("observed normal cleanup path"),
        }
    }
    assert_eq!(drops, [cloned, original]);
}
