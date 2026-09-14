//! Typed query observation, deliberately separate from target lowering.
use rustc_hir::def::DefKind;
use rustc_middle::{
    mir::{self, visit::Visitor},
    ty::{self, Ty, TyCtxt},
};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Observation {
    pub box_locals: usize,
    pub box_moves: usize,
    pub box_drops: usize,
    pub projected_drops: usize,
    pub switches: usize,
    pub returns: usize,
    pub calls: usize,
}

struct Observer<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    body: &'a mir::Body<'tcx>,
    observation: Observation,
}

impl<'tcx> Observer<'_, 'tcx> {
    fn is_box(&self, ty: Ty<'tcx>) -> bool {
        matches!(ty.kind(), ty::Adt(def, _) if Some(def.did()) == self.tcx.lang_items().owned_box())
    }
}

impl<'tcx> Visitor<'tcx> for Observer<'_, 'tcx> {
    fn visit_operand(&mut self, operand: &mir::Operand<'tcx>, location: mir::Location) {
        if let mir::Operand::Move(place) = operand
            && self.is_box(place.ty(&self.body.local_decls, self.tcx).ty)
        {
            self.observation.box_moves += 1;
        }
        self.super_operand(operand, location);
    }
}

pub(super) fn check(tcx: TyCtxt<'_>) {
    let mut seen = std::collections::BTreeSet::new();
    for owner in tcx.hir_body_owners() {
        if tcx.def_kind(owner) != DefKind::Fn {
            continue;
        }
        // Borrow before optimized_mir: that query may steal this body.
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        assert_eq!(body.source.def_id(), owner.to_def_id());
        assert_eq!(
            body.phase,
            mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup)
        );
        assert_eq!(
            tcx.hir_body_owned_by(owner).value.hir_id.owner.def_id,
            owner
        );
        let mut observer = Observer {
            tcx,
            body: &body,
            observation: Observation::default(),
        };
        observer.observation.box_locals = body
            .local_decls
            .iter()
            .filter(|local| observer.is_box(local.ty))
            .count();
        observer.visit_body(&body);
        for block in body.basic_blocks.iter() {
            assert!(
                !block.is_cleanup,
                "abort-mode fixtures have no unwind cleanup blocks"
            );
            match &block.terminator().kind {
                mir::TerminatorKind::Drop { place, unwind, .. } => {
                    assert!(observer.is_box(place.ty(&body.local_decls, tcx).ty));
                    assert_eq!(*unwind, mir::UnwindAction::Unreachable);
                    observer.observation.box_drops += 1;
                    observer.observation.projected_drops +=
                        usize::from(!place.projection.is_empty());
                }
                mir::TerminatorKind::SwitchInt { .. } => observer.observation.switches += 1,
                mir::TerminatorKind::Return => observer.observation.returns += 1,
                mir::TerminatorKind::Call { func, unwind, .. } => {
                    assert!(matches!(
                        func.ty(&body.local_decls, tcx).kind(),
                        ty::FnDef(..)
                    ));
                    assert_eq!(*unwind, mir::UnwindAction::Unreachable);
                    observer.observation.calls += 1;
                }
                _ => {}
            }
        }
        let name = tcx.def_path_str(owner);
        super::assertions::check(tcx, &name, &body, &observer.observation);
        assert!(seen.insert(name.clone()));
        println!("{} {:?} {:?}", name, body.phase, observer.observation);
    }
    assert_eq!(seen, super::assertions::names());
}
