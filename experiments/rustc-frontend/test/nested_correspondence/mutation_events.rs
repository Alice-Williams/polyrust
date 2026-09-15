//! Independent edits to constructor anchors, aggregates and complete move paths.
use super::{Reject, relations::Matched, statement, value};
use rustc_abi::{FieldIdx, VariantIdx};
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
    reject("return type", &|b| {
        b.local_decls[mir::RETURN_PLACE].ty = tcx.types.u32
    });
    for (i, construction) in matched.constructions.iter().enumerate() {
        reject("parameter type", &|b| {
            b.local_decls[construction.parameter().1].ty = tcx.types.u32
        });
        reject("constructor result type", &|b| {
            b.local_decls[construction.binding().1].ty = tcx.types.u32
        });
        let stage = construction.scalar_staging()[0];
        let other = &matched.constructions[(i + 1) % 3];
        reject("constructor parameter origin", &|b| {
            let Rvalue::Use(operand, _) = value(b, stage) else {
                panic!("stage")
            };
            *operand = Operand::Copy(other.parameter().1.into());
        });
        reject("constructor stage type", &|b| {
            let mir::StatementKind::Assign(pair) = &statement(b, stage).kind else {
                panic!("stage")
            };
            let local = pair.0.local;
            b.local_decls[local].ty = tcx.types.u32;
        });
        reject("duplicate call destination", &|b| {
            let TerminatorKind::Call { destination, .. } = &mut b.basic_blocks.as_mut()
                [construction.call().block]
                .terminator_mut()
                .kind
            else {
                panic!("call")
            };
            *destination = other.binding().1.into();
        });
        reject("constructor unwind", &|b| {
            let TerminatorKind::Call { unwind, .. } = &mut b.basic_blocks.as_mut()
                [construction.call().block]
                .terminator_mut()
                .kind
            else {
                panic!("call")
            };
            *unwind = mir::UnwindAction::Continue;
        });
        reject("constructor extra argument", &|b| {
            let TerminatorKind::Call { args, .. } = &mut b.basic_blocks.as_mut()
                [construction.call().block]
                .terminator_mut()
                .kind
            else {
                panic!("call")
            };
            *args = vec![args[0].clone(), args[0].clone()].into_boxed_slice();
        });
    }
    for (i, aggregate) in matched.aggregates.iter().enumerate() {
        let location = aggregate.location();
        let other_ty = original.local_decls[matched.aggregates[1 - i].destination().local].ty;
        let rustc_middle::ty::Adt(other, _) = other_ty.kind() else {
            panic!("record")
        };
        reject("aggregate destination type", &|b| {
            b.local_decls[aggregate.destination().local].ty = other_ty
        });
        reject("aggregate nominal identity", &|b| {
            let Rvalue::Aggregate(kind, _) = value(b, location) else {
                panic!("aggregate")
            };
            let mir::AggregateKind::Adt(def, ..) = &mut **kind else {
                panic!("adt")
            };
            *def = other.did();
        });
        reject("aggregate variant", &|b| {
            let Rvalue::Aggregate(kind, _) = value(b, location) else {
                panic!("aggregate")
            };
            let mir::AggregateKind::Adt(_, variant, ..) = &mut **kind else {
                panic!("adt")
            };
            assert_eq!(variant.as_usize(), 0);
            *variant = VariantIdx::from_usize(1);
        });
        reject("aggregate generic arguments", &|b| {
            let Rvalue::Aggregate(kind, _) = value(b, location) else {
                panic!("aggregate")
            };
            let mir::AggregateKind::Adt(_, _, args, ..) = &mut **kind else {
                panic!("adt")
            };
            *args = tcx.mk_args(&[tcx.types.i32.into()]);
        });
        reject("aggregate union field", &|b| {
            let Rvalue::Aggregate(kind, _) = value(b, location) else {
                panic!("aggregate")
            };
            let mir::AggregateKind::Adt(_, _, _, _, active) = &mut **kind else {
                panic!("adt")
            };
            *active = Some(FieldIdx::from_usize(0));
        });
        reject("aggregate missing operand", &|b| {
            let Rvalue::Aggregate(_, operands) = value(b, location) else {
                panic!("aggregate")
            };
            operands.pop();
        });
        reject("aggregate duplicate operand", &|b| {
            let Rvalue::Aggregate(_, operands) = value(b, location) else {
                panic!("aggregate")
            };
            operands[FieldIdx::from_usize(1)] = operands[FieldIdx::from_usize(0)].clone();
        });
        reject("aggregate swapped operands", &|b| {
            let Rvalue::Aggregate(_, operands) = value(b, location) else {
                panic!("aggregate")
            };
            operands.raw.swap(0, 1);
        });
        for (j, field) in aggregate.fields().iter().enumerate() {
            reject("aggregate copied operand", &|b| {
                let Rvalue::Aggregate(_, operands) = value(b, location) else {
                    panic!("aggregate")
                };
                operands[field.index()] = Operand::Copy(field.staging());
            });
            reject("staging copy", &|b| {
                let Rvalue::Use(operand, _) = value(b, field.movement()) else {
                    panic!("stage")
                };
                *operand = Operand::Copy(field.source().1);
            });
            reject("staging wrong origin", &|b| {
                let Rvalue::Use(operand, _) = value(b, field.movement()) else {
                    panic!("stage")
                };
                *operand = Operand::Move(aggregate.fields()[1 - j].source().1);
            });
            reject("staging wrong type", &|b| {
                b.local_decls[field.staging().local].ty = tcx.types.u32
            });
        }
        reject("initializer evaluation order", &|b| {
            let a = aggregate.fields()[0].movement();
            let c = aggregate.fields()[1].movement();
            assert_eq!(a.block, c.block);
            b.basic_blocks.as_mut()[a.block]
                .statements
                .swap(a.statement_index, c.statement_index);
        });
    }
    for movement in &matched.movements {
        let actual = movement.actual_source();
        reject("move becomes copy", &|b| {
            let Rvalue::Use(operand, _) = value(b, movement.location()) else {
                panic!("move")
            };
            *operand = Operand::Copy(actual);
        });
        reject("move destination type", &|b| {
            b.local_decls[movement.destination().1].ty = tcx.types.u32
        });
        reject("stale original owner", &|b| {
            let Rvalue::Use(operand, _) = value(b, movement.location()) else {
                panic!("move")
            };
            *operand = Operand::Move(matched.constructions[0].binding().1.into());
        });
        for index in 0..actual.projection.len() {
            for wrong_type in [false, true] {
                reject("complete nested projection", &|b| {
                    let mut projections = actual.projection.to_vec();
                    let ProjectionElem::Field(field, ty) = projections[index] else {
                        panic!("field")
                    };
                    projections[index] = if wrong_type {
                        ProjectionElem::Field(field, tcx.types.u32)
                    } else {
                        ProjectionElem::Field(FieldIdx::from_usize(1 - field.as_usize()), ty)
                    };
                    let changed = mir::Place::from(actual.local).project_deeper(&projections, tcx);
                    let Rvalue::Use(operand, _) = value(b, movement.location()) else {
                        panic!("move")
                    };
                    *operand = Operand::Move(changed);
                });
            }
        }
        if !actual.projection.is_empty() {
            reject("nested projection depth", &|b| {
                let changed = mir::Place::from(actual.local)
                    .project_deeper(&actual.projection[..actual.projection.len() - 1], tcx);
                let Rvalue::Use(operand, _) = value(b, movement.location()) else {
                    panic!("move")
                };
                *operand = Operand::Move(changed);
            });
        }
    }
}
