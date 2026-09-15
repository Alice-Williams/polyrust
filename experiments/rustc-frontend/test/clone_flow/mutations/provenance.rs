//! Isolated type-argument, owner-provenance and normal-return corruptions.
use super::{Reject, relations, term, value};
use crate::owned_linear::cloning::Owner;
use rustc_middle::{
    mir::{self, Operand, Rvalue, TerminatorKind},
    ty::{self, Ty, TyCtxt},
};

pub(super) fn cases<'tcx>(
    tcx: TyCtxt<'tcx>,
    original: &mir::Body<'tcx>,
    matched: &relations::Matched<'tcx>,
    reject: &mut Reject<'_, 'tcx>,
) {
    let clone_at = matched
        .bindings
        .iter()
        .find(|(tag, ..)| *tag == Owner::Cloned)
        .unwrap()
        .3;
    reject(
        "same clone definition with different trait arguments",
        &|body| {
            let TerminatorKind::Call {
                func: Operand::Constant(function),
                ..
            } = term(body, clone_at)
            else {
                unreachable!()
            };
            let ty::FnDef(definition, arguments) = function.const_.ty().kind() else {
                unreachable!()
            };
            let changed = tcx.mk_args(&[tcx.types.i32.into()]);
            assert_ne!(*arguments, changed);
            function.const_ = mir::Const::Val(
                mir::ConstValue::ZeroSized,
                Ty::new_fn_def(tcx, *definition, changed),
            );
        },
    );
    reject("return relocated before first cleanup", &|body| {
        *term(body, matched.returning) = TerminatorKind::Unreachable;
        *term(body, matched.drops[0].2) = TerminatorKind::Return;
    });
    let mut current: [Option<mir::Local>; 2] = [None, None];
    let mut first: [Option<mir::Local>; 2] = [None, None];
    for &(tag, _, local, at) in &matched.bindings {
        let index = match tag {
            Owner::Original => 0,
            Owner::Cloned => 1,
        };
        if let Some(previous) = current[index] {
            let mir::StatementKind::Assign(pair) =
                &original.basic_blocks[at.block].statements[at.statement_index].kind
            else {
                unreachable!()
            };
            assert!(
                matches!(pair.1, Rvalue::Use(Operand::Move(source), _) if source == mir::Place::from(previous))
            );
            if let Some(other) = current[1 - index] {
                assert_eq!(
                    original.local_decls[other].ty,
                    original.local_decls[previous].ty
                );
                reject("whole move consumes the other live owner", &|body| {
                    let Rvalue::Use(operand, _) = value(body, at) else {
                        unreachable!()
                    };
                    *operand = Operand::Move(mir::Place::from(other));
                });
            }
            if let Some(stale) = first[index].filter(|old| *old != previous) {
                reject("whole move reuses an already moved source", &|body| {
                    let Rvalue::Use(operand, _) = value(body, at) else {
                        unreachable!()
                    };
                    *operand = Operand::Move(mir::Place::from(stale));
                });
            }
        } else {
            first[index] = Some(local);
        }
        current[index] = Some(local);
    }
}
