//! Consume the certificate without privileged access to its private fields.
use crate::owned_linear::calls::{BodyEnding, BodyStep, OwnedCallBody, SourceExit};
use rustc_hir as hir;
use rustc_middle::{mir, ty::TyCtxt};
pub(super) fn check(tcx: TyCtxt<'_>, proof: &OwnedCallBody<'_>) {
    let owner = proof.owner();
    let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
    let hir::ExprKind::Block(root, None) = tcx.hir_body_owned_by(owner).value.kind else {
        panic!("root");
    };
    match proof.exit() {
        SourceExit::Tail(value) => assert!(std::ptr::eq(root.expr.unwrap(), value)),
        SourceExit::Return { expression, value } => {
            assert!(
                matches!(expression.kind, hir::ExprKind::Ret(Some(v)) if std::ptr::eq(v, value))
            );
            let actual = root.expr.unwrap_or_else(|| {
                let hir::StmtKind::Semi(e) = root.stmts.last().unwrap().kind else {
                    panic!("return");
                };
                e
            });
            assert!(std::ptr::eq(actual, expression));
        }
    }
    assert!(matches!(
        body.basic_blocks[proof.returning().block].terminator().kind,
        mir::TerminatorKind::Return
    ));
    assert_eq!(
        proof.bindings().len(),
        root.stmts
            .iter()
            .filter(|s| matches!(s.kind, hir::StmtKind::Let(_)))
            .count()
    );
    for (hir, local) in proof.bindings() {
        assert_eq!(
            tcx.typeck(owner).node_type(*hir),
            body.local_decls[*local].ty
        );
    }
    for step in proof.steps() {
        match step {
            BodyStep::Move(at) => {
                assert!(matches!(
                    body.basic_blocks[at.block].statements[at.statement_index].kind,
                    mir::StatementKind::Assign(_)
                ));
            }
            BodyStep::Allocation(site) | BodyStep::LocalCall(site) => {
                let mir::TerminatorKind::Call {
                    args, destination, ..
                } = &body.basic_blocks[site.location().block].terminator().kind
                else {
                    panic!("call");
                };
                assert_eq!(destination.local, site.destination());
                assert_eq!(args[0].node.place().unwrap().local, site.argument());
                assert!(!site.staging().is_empty());
                for at in site.staging() {
                    assert!(matches!(
                        body.basic_blocks[at.block].statements[at.statement_index].kind,
                        mir::StatementKind::Assign(_)
                    ));
                }
            }
        }
    }
    match proof.ending() {
        BodyEnding::ReadDrop { cast, read, drop } => {
            for at in [cast, read] {
                assert!(matches!(
                    body.basic_blocks[at.block].statements[at.statement_index].kind,
                    mir::StatementKind::Assign(_)
                ));
            }
            assert!(matches!(
                body.basic_blocks[drop.block].terminator().kind,
                mir::TerminatorKind::Drop { .. }
            ));
        }
        BodyEnding::OwnerReturn(at) => {
            let mir::StatementKind::Assign(pair) =
                &body.basic_blocks[at.block].statements[at.statement_index].kind
            else {
                panic!("owner return");
            };
            assert_eq!(pair.0.local, mir::RETURN_PLACE);
        }
        BodyEnding::DirectReturn => assert!(proof.steps().iter().any(|s| match s {
            BodyStep::Allocation(c) | BodyStep::LocalCall(c) =>
                c.destination() == mir::RETURN_PLACE,
            _ => false,
        })),
    }
}
