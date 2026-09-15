//! Cleanup groups, scalar read provenance and complete control-flow accounting.
use super::{Reject, relations::Matched, statement, value};
use rustc_middle::{
    mir::{self, Operand, ProjectionElem, Rvalue, TerminatorKind},
    ty::TyCtxt,
};

pub(super) fn check<'tcx>(
    tcx: TyCtxt<'tcx>,
    original: &mir::Body<'tcx>,
    matched: &Matched<'tcx>,
    reject: &mut Reject<'_, 'tcx>,
) {
    for (i, cleanup) in matched.cleanup.iter().enumerate() {
        let location = cleanup.location();
        reject("duplicate cleanup owner", &|b| {
            *drop_place(b, location) = matched.cleanup[(i + 1) % matched.cleanup.len()].actual();
        });
        reject("stale constructor cleanup", &|b| {
            *drop_place(b, location) = matched.constructions[0].binding().1.into();
        });
        reject("whole Outer cleanup", &|b| {
            *drop_place(b, location) = matched.aggregates[1].destination();
        });
        reject("cleanup unwind", &|b| {
            let TerminatorKind::Drop { unwind, .. } = &mut b.basic_blocks.as_mut()[location.block]
                .terminator_mut()
                .kind
            else {
                panic!("drop")
            };
            *unwind = mir::UnwindAction::Continue;
        });
        if cleanup.kind() == super::super::events::CleanupKind::InnerRecord {
            reject("whole inner replaced by one leaf", &|b| {
                *drop_place(b, location) = matched
                    .leaves
                    .iter()
                    .find(|l| l.cleanup_location() == location)
                    .unwrap()
                    .actual();
            });
        }
        if !cleanup.actual().projection.is_empty() {
            reject("cleanup projection depth", &|b| {
                *drop_place(b, location) = cleanup.actual().local.into();
            });
        }
    }
    reject("cleanup order", &|b| {
        *drop_place(b, matched.cleanup[0].location()) = matched.cleanup[1].actual();
        *drop_place(b, matched.cleanup[1].location()) = matched.cleanup[0].actual();
    });
    let mir::StatementKind::Assign(pair) =
        &original.basic_blocks[matched.read.block].statements[matched.read.statement_index].kind
    else {
        panic!("read")
    };
    let Rvalue::Use(Operand::Copy(pointer), _) = &pair.1 else {
        panic!("pointer")
    };
    let trace = crate::owned_linear::flow::trace(original).unwrap();
    let cast = trace.definition(pointer.local).unwrap().location;
    reject("read pointer type", &|b| {
        b.local_decls[pointer.local].ty = tcx.types.u32
    });
    reject("read cast target", &|b| {
        let Rvalue::Cast(_, _, target) = value(b, cast) else {
            panic!("cast")
        };
        *target = tcx.types.u32;
    });
    reject("read original moved owner", &|b| {
        let Rvalue::Cast(_, Operand::Copy(source), _) = value(b, cast) else {
            panic!("cast")
        };
        source.local = matched.constructions[0].binding().1;
    });
    reject("read without dereference", &|b| {
        let Rvalue::Use(operand, _) = value(b, matched.read) else {
            panic!("read")
        };
        *operand = Operand::Copy(pointer.local.into());
    });
    for index in 0..2 {
        reject("read representation projection type", &|b| {
            let Rvalue::Cast(_, Operand::Copy(source), _) = value(b, cast) else {
                panic!("cast")
            };
            let mut fields = source.projection.to_vec();
            assert_eq!(fields.len(), 2);
            let ProjectionElem::Field(field, _) = fields[index] else {
                panic!("field")
            };
            fields[index] = ProjectionElem::Field(field, tcx.types.u32);
            *source = mir::Place::from(source.local).project_deeper(&fields, tcx);
        });
    }
    reject("unaccounted scalar assignment", &|b| {
        let duplicate = statement(b, matched.read).clone();
        b.basic_blocks.as_mut()[matched.returning.block]
            .statements
            .push(duplicate);
    });
    reject("unaccounted extra normal block", &|b| {
        b.basic_blocks
            .as_mut()
            .push(original.basic_blocks[matched.returning.block].clone());
    });
    reject("normal control-flow cycle", &|b| {
        b.basic_blocks.as_mut()[matched.returning.block]
            .terminator_mut()
            .kind = TerminatorKind::Goto {
            target: mir::START_BLOCK,
        };
    });
    reject("normal block becomes unwind cleanup", &|b| {
        b.basic_blocks.as_mut()[matched.returning.block].is_cleanup = true;
    });
    reject("extra constructor call", &|b| {
        let call = original.basic_blocks[matched.constructions[0].call().block].clone();
        b.basic_blocks.as_mut().push(call);
    });
}

fn drop_place<'a, 'tcx>(
    body: &'a mut mir::Body<'tcx>,
    location: mir::Location,
) -> &'a mut mir::Place<'tcx> {
    let TerminatorKind::Drop { place, .. } = &mut body.basic_blocks.as_mut()[location.block]
        .terminator_mut()
        .kind
    else {
        panic!("drop")
    };
    place
}
