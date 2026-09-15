//! Inspect real method resolution and borrow staging before defining admission.
use rustc_hir::{self as hir, intravisit::Visitor};
use rustc_middle::{
    mir,
    ty::{self, TyCtxt},
};

struct Calls<'tcx> {
    tcx: TyCtxt<'tcx>,
    owner: hir::def_id::LocalDefId,
    clones: Vec<(ty::Ty<'tcx>, ty::Instance<'tcx>)>,
}
impl<'tcx> Visitor<'tcx> for Calls<'tcx> {
    fn visit_expr(&mut self, expression: &'tcx hir::Expr<'tcx>) {
        let checked = self.tcx.typeck(self.owner);
        let callable = match expression.kind {
            hir::ExprKind::MethodCall(_, receiver, _, _) => {
                println!(
                    "METHOD receiver {:?} adjustments {:?}",
                    checked.expr_ty(receiver),
                    checked.expr_adjustments(receiver)
                );
                checked
                    .type_dependent_def_id(expression.hir_id)
                    .map(|def| (def, checked.node_args(expression.hir_id)))
            }
            hir::ExprKind::Call(callee, _) => match *checked.expr_ty(callee).kind() {
                ty::FnDef(def, args) => Some((def, args)),
                _ => None,
            },
            _ => None,
        };
        if let Some((def, args)) = callable {
            let instance = ty::Instance::try_resolve(
                self.tcx,
                ty::TypingEnv::post_analysis(self.tcx, self.owner),
                def,
                args,
            )
            .expect("resolution query");
            if Some(def) == self.tcx.lang_items().clone_fn() {
                assert_eq!(args.len(), 1);
                crate::adjustments::check(self.tcx, self.owner, expression, args.type_at(0));
                self.clones
                    .push((args.type_at(0), instance.expect("concrete clone")));
                if let hir::ExprKind::Call(_, arguments) = expression.kind {
                    for argument in arguments {
                        println!(
                            "EXPLICIT {:?} adjustments {:?}",
                            argument.kind,
                            checked.expr_adjustments(argument)
                        );
                    }
                }
            }
            println!(
                "CALL {:?} args {:?} instance {:?} signature {:?}",
                def,
                args,
                instance,
                self.tcx.fn_sig(def).instantiate(self.tcx, args)
            );
        }
        hir::intravisit::walk_expr(self, expression);
    }
}
pub(super) fn check(tcx: TyCtxt<'_>) {
    println!(
        "CLONE trait {:?} method {:?}",
        tcx.lang_items().clone_trait(),
        tcx.lang_items().clone_fn()
    );
    let mut seen = std::collections::BTreeSet::new();
    for owner in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == hir::def::DefKind::Fn)
    {
        // This observation pins seven selected representations. The separate
        // capability test checks the complete fixture and its rejection cases.
        if ![
            "method",
            "qualified",
            "inferred",
            "reference",
            "wrong_payload",
            "replacement",
            "custom",
        ]
        .contains(&tcx.def_path_str(owner).as_str())
        {
            continue;
        }
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        assert_eq!(body.source.def_id(), owner.to_def_id());
        assert_eq!(
            body.phase,
            mir::MirPhase::Runtime(mir::RuntimePhase::PostCleanup)
        );
        let name = tcx.def_path_str(owner);
        println!("FUNCTION {name}");
        if matches!(
            name.as_str(),
            "method" | "qualified" | "inferred" | "wrong_payload"
        ) {
            crate::loans::check(
                tcx,
                &body,
                matches!(name.as_str(), "qualified" | "inferred"),
            );
        }
        let mut calls = Calls {
            tcx,
            owner,
            clones: Vec::new(),
        };
        calls.visit_expr(tcx.hir_body_owned_by(owner).value);
        match name.as_str() {
            "replacement" | "custom" => assert!(calls.clones.is_empty()),
            "reference" => {
                assert_eq!(calls.clones.len(), 1);
                let (self_ty, instance) = calls.clones[0];
                let ty::Ref(_, inner, hir::Mutability::Not) = *self_ty.kind() else {
                    panic!("shared reference clone")
                };
                let constructor = tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap();
                let full_box = tcx
                    .fn_sig(constructor)
                    .instantiate(tcx, tcx.mk_args(&[tcx.types.i32.into()]))
                    .skip_binder()
                    .output();
                assert_eq!(inner, full_box);
                assert_eq!(
                    tcx.associated_item(instance.def_id()).trait_item_def_id(),
                    tcx.lang_items().clone_fn()
                );
                let implementation = tcx.parent(instance.def_id());
                let environment = ty::TypingEnv::post_analysis(tcx, owner);
                let trait_ref = tcx
                    .try_normalize_erasing_regions(
                        environment,
                        tcx.impl_trait_ref(implementation)
                            .instantiate(tcx, instance.args),
                    )
                    .unwrap();
                assert_eq!(Some(trait_ref.def_id), tcx.lang_items().clone_trait());
                assert_eq!(trait_ref.self_ty(), self_ty);
                let box_instance = ty::Instance::try_resolve(
                    tcx,
                    environment,
                    tcx.lang_items().clone_fn().unwrap(),
                    tcx.mk_args(&[full_box.into()]),
                )
                .unwrap()
                .unwrap();
                assert_ne!(instance.def_id(), box_instance.def_id());
            }
            _ => {
                assert_eq!(calls.clones.len(), 1);
                let (box_ty, instance) = calls.clones[0];
                let ty::Adt(def, args) = box_ty.kind() else {
                    panic!("Box clone")
                };
                assert_eq!(Some(def.did()), tcx.lang_items().owned_box());
                assert_eq!(
                    args.type_at(0),
                    if name == "wrong_payload" {
                        tcx.types.bool
                    } else {
                        tcx.types.i32
                    }
                );
                assert_eq!(instance.def_id().krate, def.did().krate);
                let implementation = tcx.parent(instance.def_id());
                let trait_ref = tcx
                    .impl_trait_ref(implementation)
                    .instantiate(tcx, instance.args);
                let trait_ref = tcx
                    .try_normalize_erasing_regions(
                        ty::TypingEnv::post_analysis(tcx, owner),
                        trait_ref,
                    )
                    .expect("normalized concrete trait");
                assert_eq!(Some(trait_ref.def_id), tcx.lang_items().clone_trait());
                assert_eq!(trait_ref.self_ty(), box_ty);
                assert_eq!(
                    tcx.associated_item(instance.def_id()).trait_item_def_id(),
                    tcx.lang_items().clone_fn()
                );
            }
        }
        let drops = body
            .basic_blocks
            .iter()
            .filter(|block| matches!(block.terminator().kind, mir::TerminatorKind::Drop { .. }))
            .count();
        assert_eq!(
            drops,
            if matches!(name.as_str(), "reference" | "custom") {
                1
            } else {
                2
            }
        );
        for (block, data) in body.basic_blocks.iter_enumerated() {
            println!(
                "{block:?}: {:?} {:?}",
                data.statements,
                data.terminator().kind
            );
        }
        assert!(seen.insert(name));
    }
    assert_eq!(
        seen,
        [
            "method",
            "qualified",
            "inferred",
            "reference",
            "wrong_payload",
            "replacement",
            "custom"
        ]
        .into_iter()
        .map(String::from)
        .collect()
    );
}
