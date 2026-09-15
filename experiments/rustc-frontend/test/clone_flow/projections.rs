//! Sibling consumer checks exact projected operations without private relations.
use crate::owned_linear::cloning::{CloneOwnedBody, Owner};
use rustc_middle::{
    mir::{self, Operand, Rvalue, StatementKind, TerminatorKind},
    ty::{self, TyCtxt},
};

pub(super) fn check<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>, proof: &CloneOwnedBody<'tcx>) {
    let mut current = [None, None];
    for &(tag, binding, local, at) in proof.bindings() {
        let index = match tag {
            Owner::Original => 0,
            Owner::Cloned => 1,
        };
        let block = &body.basic_blocks[at.block];
        if let Some(previous) = current[index] {
            let StatementKind::Assign(pair) = &block.statements[at.statement_index].kind else {
                panic!("move assignment")
            };
            assert_eq!(pair.0, mir::Place::from(local));
            assert!(
                matches!(pair.1, Rvalue::Use(Operand::Move(source), _) if source == mir::Place::from(previous))
            );
        } else {
            assert_eq!(at.statement_index, block.statements.len());
            let TerminatorKind::Call {
                func, destination, ..
            } = &block.terminator().kind
            else {
                panic!("producer call")
            };
            assert_eq!(*destination, mir::Place::from(local));
            let rustc_hir::Node::LetStmt(declaration) = tcx.hir_node(tcx.parent_hir_id(binding))
            else {
                panic!("source declaration")
            };
            let expression = declaration.init.unwrap();
            let typeck = tcx.typeck(proof.owner());
            let (definition, arguments) = match expression.kind {
                rustc_hir::ExprKind::MethodCall(..) => (
                    typeck.type_dependent_def_id(expression.hir_id).unwrap(),
                    typeck.node_args(expression.hir_id),
                ),
                rustc_hir::ExprKind::Call(callee, _) => {
                    let ty::FnDef(definition, arguments) = typeck.expr_ty(callee).kind() else {
                        panic!("source function type")
                    };
                    (*definition, *arguments)
                }
                _ => panic!("source producer"),
            };
            assert!(
                matches!(func.ty(&body.local_decls, tcx).kind(), ty::FnDef(actual, args) if *actual == definition && *args == arguments)
            );
        }
        current[index] = Some(local);
    }
    let at = proof.scalar_read();
    let StatementKind::Assign(pair) =
        &body.basic_blocks[at.block].statements[at.statement_index].kind
    else {
        panic!("scalar read")
    };
    assert_eq!(pair.0, mir::Place::from(mir::RETURN_PLACE));
    let Rvalue::Use(Operand::Copy(source), _) = pair.1 else {
        panic!("scalar Copy")
    };
    assert_eq!(source.projection.as_ref(), &[mir::ProjectionElem::Deref]);
    let selected = current[match proof.read_owner() {
        Owner::Original => 0,
        Owner::Cloned => 1,
    }]
    .unwrap();
    let definitions: Vec<_> = body
        .basic_blocks
        .iter()
        .flat_map(|block| &block.statements)
        .filter_map(|statement| {
            let StatementKind::Assign(pair) = &statement.kind else {
                return None;
            };
            (pair.0 == mir::Place::from(source.local)).then_some(&pair.1)
        })
        .collect();
    assert_eq!(definitions.len(), 1);
    let Rvalue::Cast(_, Operand::Copy(pointer), _) = definitions[0] else {
        panic!("authenticated owner pointer cast")
    };
    assert_eq!(pointer.local, selected);
}
