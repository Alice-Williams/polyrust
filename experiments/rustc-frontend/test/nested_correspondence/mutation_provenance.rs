//! Coherent nominal/owner substitutions and a semantics-preserving local bijection.
use super::{Reject, relations, source, value};
use rustc_hir::def_id::LocalDefId;
use rustc_middle::{
    mir::{
        self, Operand, Rvalue, TerminatorKind,
        visit::{MutVisitor, PlaceContext},
    },
    ty::{self, Ty, TyCtxt},
};

pub(super) fn check<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    original: &mir::Body<'tcx>,
    matched: &relations::Matched<'tcx>,
    reject: &mut Reject<'_, 'tcx>,
) {
    let alternate = tcx
        .hir_body_owners()
        .find(|id| {
            tcx.def_path_str(*id)
                == if tcx.def_path_str(owner) == "alternate::same" {
                    "first"
                } else {
                    "alternate::same"
                }
        })
        .unwrap();
    let other = source::read(tcx, alternate).unwrap();
    let replacements = [other.inner.1.result(), other.outer.1.result()];
    for (index, aggregate) in matched.aggregates.iter().enumerate() {
        let before = original.local_decls[aggregate.destination().local].ty;
        let after = replacements[index];
        assert_ne!(before, after);
        let ty::Adt(definition, _) = after.kind() else {
            panic!("record")
        };
        reject("same-shape nominal definition", &|b| {
            let Rvalue::Aggregate(kind, _) = value(b, aggregate.location()) else {
                panic!("aggregate")
            };
            let mir::AggregateKind::Adt(def, ..) = &mut **kind else {
                panic!("adt")
            };
            *def = definition.did();
        });
    }
    reject("coherent replacement record tree", &|b| {
        let pairs = [
            (
                original.local_decls[matched.aggregates[0].destination().local].ty,
                replacements[0],
            ),
            (
                original.local_decls[matched.aggregates[1].destination().local].ty,
                replacements[1],
            ),
        ];
        for declaration in b.local_decls.iter_mut() {
            if let Some((_, replacement)) =
                pairs.iter().find(|(before, _)| *before == declaration.ty)
            {
                declaration.ty = *replacement;
            }
        }
        for block in b.basic_blocks.as_mut() {
            for statement in &mut block.statements {
                if let mir::StatementKind::Assign(pair) = &mut statement.kind {
                    match &mut pair.1 {
                        Rvalue::Aggregate(kind, _) => {
                            let mir::AggregateKind::Adt(def, ..) = &mut **kind else {
                                panic!("adt")
                            };
                            for (before, after) in pairs {
                                let (ty::Adt(old, _), ty::Adt(new, _)) =
                                    (before.kind(), after.kind())
                                else {
                                    panic!("record")
                                };
                                if *def == old.did() {
                                    *def = new.did();
                                    break;
                                }
                            }
                        }
                        Rvalue::Use(Operand::Move(place) | Operand::Copy(place), _) => {
                            rewrite(tcx, place, &pairs)
                        }
                        _ => {}
                    }
                }
            }
            if let TerminatorKind::Drop { place, .. } = &mut block.terminator_mut().kind {
                rewrite(tcx, place, &pairs);
            }
        }
    });
    for construction in &matched.constructions {
        for change_definition in [false, true] {
            reject("constructor full identity", &|b| {
                let TerminatorKind::Call {
                    func: Operand::Constant(function),
                    ..
                } = &mut b.basic_blocks.as_mut()[construction.call().block]
                    .terminator_mut()
                    .kind
                else {
                    panic!("call")
                };
                let ty::FnDef(definition, _) = function.const_.ty().kind() else {
                    panic!("function")
                };
                let (definition, arguments) = if change_definition {
                    (owner.to_def_id(), tcx.mk_args(&[]))
                } else {
                    (*definition, tcx.mk_args(&[tcx.types.u32.into()]))
                };
                let changed = Ty::new_fn_def(tcx, definition, arguments);
                assert_ne!(changed, function.const_.ty());
                function.const_ = mir::Const::Val(mir::ConstValue::ZeroSized, changed);
            });
        }
    }
    let a = matched.aggregates[0].fields()[0].movement();
    let c = matched.aggregates[0].fields()[1].movement();
    reject(
        "coherent swapped leaves with unchanged ownership counts",
        &|b| {
            let Rvalue::Use(Operand::Move(first), _) = value(b, a) else {
                panic!("stage")
            };
            let saved = *first;
            let Rvalue::Use(Operand::Move(second), _) = value(b, c) else {
                panic!("stage")
            };
            let replacement = *second;
            *second = saved;
            let Rvalue::Use(Operand::Move(first), _) = value(b, a) else {
                panic!("stage")
            };
            *first = replacement;
        },
    );
    let mut wrong_owner = source::read(tcx, owner).unwrap();
    wrong_owner.frame.owner = alternate;
    assert!(relations::validate(tcx, owner, &wrong_owner, original).is_err());
    println!("nested canonical source owner substitution rejected");
}

fn rewrite<'tcx>(tcx: TyCtxt<'tcx>, place: &mut mir::Place<'tcx>, pairs: &[(Ty<'tcx>, Ty<'tcx>)]) {
    let mut projections = place.projection.to_vec();
    for projection in &mut projections {
        if let mir::ProjectionElem::Field(_, ty) = projection
            && let Some((_, replacement)) = pairs.iter().find(|(before, _)| before == ty)
        {
            *ty = *replacement;
        }
    }
    *place = mir::Place::from(place.local).project_deeper(&projections, tcx);
}

struct Rename<'tcx> {
    tcx: TyCtxt<'tcx>,
    first: mir::Local,
    second: mir::Local,
}
impl<'tcx> MutVisitor<'tcx> for Rename<'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
    fn visit_local(&mut self, local: &mut mir::Local, _: PlaceContext, _: mir::Location) {
        if *local == self.first {
            *local = self.second;
        } else if *local == self.second {
            *local = self.first;
        }
    }
}
pub(super) fn renaming<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    original: &mir::Body<'tcx>,
    before: &relations::Matched<'tcx>,
) {
    let first = before.constructions[0].binding().1;
    let second = before.constructions[1].binding().1;
    assert_ne!(first, second);
    assert_eq!(
        original.local_decls[first].ty,
        original.local_decls[second].ty
    );
    let mut changed = original.clone();
    Rename { tcx, first, second }.visit_body(&mut changed);
    changed.local_decls[first] = original.local_decls[second].clone();
    changed.local_decls[second] = original.local_decls[first].clone();
    let plan = source::read(tcx, owner).unwrap();
    let after = relations::validate(tcx, owner, &plan, &changed).unwrap();
    assert_eq!(after.constructions[0].binding().1, second);
    assert_eq!(after.constructions[1].binding().1, first);
    assert_eq!(after.read, before.read);
    assert_eq!(after.returning, before.returning);
    assert_eq!(
        after
            .leaves
            .iter()
            .map(|l| (l.constructor(), l.actual(), l.cleanup_location()))
            .collect::<Vec<_>>(),
        before
            .leaves
            .iter()
            .map(|l| (l.constructor(), l.actual(), l.cleanup_location()))
            .collect::<Vec<_>>()
    );
    println!("nested complete owner-local bijection accepted");
}
