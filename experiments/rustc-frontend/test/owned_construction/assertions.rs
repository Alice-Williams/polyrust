//! Typed per-body oracle; fixture names do not grant capability authority.
use crate::{
    owned_source::{BoxConstructionInput, Builder, ConstructionError},
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::{
    self as hir,
    def::DefKind,
    def_id::{DefId, LocalDefId},
    intravisit::{self, Visitor},
};
use rustc_middle::{
    mir,
    ty::{self, TyCtxt},
};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub(super) struct Observe;
pub(super) struct Context<'tcx> {
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    calls: usize,
}
impl Mapping for Observe {
    type Capability = crate::owned_source::OwnedBoxConstruction;
    type Context<'tcx> = Context<'tcx>;
    type Output = DefId;
    fn lower<'tcx>(
        &self,
        context: &mut Context<'tcx>,
        input: BoxConstructionInput<'tcx>,
    ) -> Result<DefId, String> {
        assert_eq!(input.owner(), context.owner);
        assert_eq!(input.call().hir_id.owner.def_id, context.owner);
        let signature = context
            .tcx
            .fn_sig(input.constructor())
            .instantiate(context.tcx, input.arguments())
            .skip_binder();
        assert_eq!(signature.output(), input.result());
        assert_eq!(
            context.tcx.typeck(context.owner).expr_ty(input.argument()),
            context.tcx.types.i32
        );
        context.calls += 1;
        Ok(input.constructor())
    }
}

/// This proof consumer fixes the executable context and output; a real backend
/// supplies its own corresponding typed factory rather than support flags.
pub(super) fn bindings<M>(
    mapping: M,
) -> impl Supports<crate::owned_source::OwnedBoxConstruction, Mapping = M>
where
    M: for<'tcx> Mapping<
            Capability = crate::owned_source::OwnedBoxConstruction,
            Context<'tcx> = Context<'tcx>,
            Output = DefId,
        >,
{
    Builder::new().construction(mapping).build()
}

struct Calls<'tcx> {
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    accepted: Vec<BoxConstructionInput<'tcx>>,
    rejected: Vec<ConstructionError>,
}
impl<'tcx> Visitor<'tcx> for Calls<'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if matches!(expr.kind, hir::ExprKind::Call(..)) {
            match BoxConstructionInput::read(self.tcx, self.owner, expr) {
                Ok(input) => self.accepted.push(input),
                Err(error) => {
                    if error == ConstructionError::NotStandardConstructor {
                        // Fixture-only same-name proof; never constructor admission.
                        let hir::ExprKind::Call(callee, _) = expr.kind else {
                            unreachable!()
                        };
                        let checked = self.tcx.typeck(self.owner);
                        let ty::FnDef(def, _) = *checked.expr_ty(callee).kind() else {
                            panic!("expected fixture direct function")
                        };
                        let standard = self
                            .tcx
                            .get_diagnostic_item(rustc_span::sym::box_new)
                            .unwrap();
                        assert!(def.is_local());
                        assert_ne!(def, standard);
                        assert_eq!(self.tcx.item_name(def), self.tcx.item_name(standard));
                        let ty::Adt(result, _) = checked.expr_ty(expr).kind() else {
                            panic!("expected Box result")
                        };
                        if self.tcx.def_path_str(self.owner) == "wrapper" {
                            assert_eq!(Some(result.did()), self.tcx.lang_items().owned_box());
                        } else {
                            assert!(result.did().is_local());
                            assert_eq!(self.tcx.item_name(result.did()).as_str(), "Box");
                        }
                    }
                    self.rejected.push(error);
                }
            }
        }
        intravisit::walk_expr(self, expr);
    }
}

pub(super) fn check(tcx: TyCtxt<'_>) {
    let mut seen = BTreeSet::new();
    let owners: Vec<_> = tcx
        .hir_body_owners()
        .filter(|owner| tcx.def_kind(*owner) == DefKind::Fn)
        .collect();
    for &owner in &owners {
        let name = tcx.def_path_str(owner);
        let mut calls = Calls {
            tcx,
            owner,
            accepted: vec![],
            rejected: vec![],
        };
        calls.visit_body(tcx.hir_body_owned_by(owner));
        let (accepted, rejected): (usize, Vec<_>) = match name.as_str() {
            "genuine" | "qualified" | "renamed" | "imitation::new" => (1, vec![]),
            "unsupported" => (0, vec![ConstructionError::UnsupportedPayload]),
            "wrapper" | "counterfeit" => (0, vec![ConstructionError::NotStandardConstructor]),
            "function_item" | "callee_block" => (0, vec![ConstructionError::NotDirectCall]),
            _ => panic!("unasserted fixture {name}"),
        };
        assert_eq!(calls.accepted.len(), accepted, "{name}");
        assert_eq!(calls.rejected, rejected, "{name}");
        let bindings = bindings(Observe);
        let mut context = Context {
            tcx,
            owner,
            calls: 0,
        };
        let mut source = Vec::new();
        for input in calls.accepted {
            let copied = hir::Expr {
                hir_id: input.call().hir_id,
                kind: input.call().kind,
                span: input.call().span,
            };
            assert!(matches!(
                BoxConstructionInput::read(tcx, owner, &copied),
                Err(ConstructionError::NonCanonicalNode)
            ));
            let wrong_owner = *owners.iter().find(|other| **other != owner).unwrap();
            assert!(matches!(
                BoxConstructionInput::read(tcx, wrong_owner, input.call()),
                Err(ConstructionError::WrongOwner)
            ));
            source.push((input.constructor(), input.arguments(), input.result()));
            assert_eq!(
                bindings.mapping().lower(&mut context, input).unwrap(),
                tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap()
            );
        }
        assert_eq!(
            context.calls, accepted,
            "executable mapping was not invoked"
        );
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        assert_eq!(body.source.def_id(), owner.to_def_id());
        assert_eq!(
            body.phase,
            mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup)
        );
        let mir_calls: Vec<_> = body
            .basic_blocks
            .iter()
            .filter_map(|block| {
                let mir::TerminatorKind::Call {
                    func, destination, ..
                } = &block.terminator().kind
                else {
                    return None;
                };
                let ty::FnDef(def, args) = *func.ty(&body.local_decls, tcx).kind() else {
                    return None;
                };
                let signature = tcx.fn_sig(def).instantiate(tcx, args).skip_binder();
                if Some(def) != tcx.get_diagnostic_item(rustc_span::sym::box_new)
                    || signature.inputs() != [tcx.types.i32]
                {
                    return None;
                }
                Some((def, args, destination.ty(&body.local_decls, tcx).ty))
            })
            .collect();
        if matches!(name.as_str(), "function_item" | "callee_block") {
            assert!(source.is_empty());
            assert_eq!(
                mir_calls.len(),
                1,
                "authentic callee type alone must not admit unimplemented evaluation forms"
            );
        } else {
            assert_eq!(
                source, mir_calls,
                "constructor inventory mismatch in {name}"
            );
        }
        assert!(seen.insert(name));
    }
    assert_eq!(
        seen,
        [
            "genuine",
            "qualified",
            "renamed",
            "unsupported",
            "wrapper",
            "counterfeit",
            "imitation::new",
            "function_item",
            "callee_block"
        ]
        .into_iter()
        .map(String::from)
        .collect()
    );
}
