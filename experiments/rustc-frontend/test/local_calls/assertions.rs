//! Non-vacuous canonical call/role and executable-mapping assertions.
use crate::{
    mapping,
    owned_source::{
        BoxConstructionInput, Builder, OwnedBoxConstruction,
        local_call::{CallError, CallRole, LocalCallInput, OwnedLocalCall},
    },
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::{
    self as hir,
    def::DefKind,
    def_id::LocalDefId,
    intravisit::{self, Visitor},
};
use rustc_middle::ty::{self, TyCtxt};
use std::collections::BTreeMap;
struct Calls<'tcx> {
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    accepted: Vec<LocalCallInput<'tcx>>,
    rejected: Vec<CallError>,
    boxes: Vec<BoxConstructionInput<'tcx>>,
}
impl<'tcx> Visitor<'tcx> for Calls<'tcx> {
    fn visit_expr(&mut self, e: &'tcx hir::Expr<'tcx>) {
        if matches!(
            e.kind,
            hir::ExprKind::Call(..) | hir::ExprKind::MethodCall(..)
        ) {
            match LocalCallInput::read(self.tcx, self.owner, e) {
                Ok(input) => self.accepted.push(input),
                Err(error) => self.rejected.push(error),
            }
            if let Ok(input) = BoxConstructionInput::read(self.tcx, self.owner, e) {
                self.boxes.push(input);
            }
        }
        intravisit::walk_expr(self, e);
    }
}
pub(super) fn check(tcx: TyCtxt<'_>) {
    let owners: Vec<_> = tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
        .collect();
    let mut inventory = BTreeMap::new();
    let mut context = mapping::Context {
        tcx,
        calls: 0,
        boxes: 0,
    };
    let mut identities = BTreeMap::new();
    for &owner in &owners {
        let name = tcx.def_path_str(owner);
        let mut calls = Calls {
            tcx,
            owner,
            accepted: vec![],
            rejected: vec![],
            boxes: vec![],
        };
        calls.visit_expr(tcx.hir_body_owned_by(owner).value);
        let shape = (
            calls.accepted.len(),
            calls.rejected.len(),
            calls.boxes.len(),
        );
        assert!(inventory.insert(name.clone(), shape).is_none());
        let rejection = match name.as_str() {
            "result_adjusted" | "argument_adjusted" => CallError::Adjustment,
            "computed" | "closure" | "wrong_arity" | "method" | "pointer" => {
                CallError::NotDirectCall
            }
            "call_generic" | "wrong_result" | "call_foreign" | "wrong_payload" => {
                CallError::UnsupportedSignature
            }
            _ => CallError::NotLocalFunction,
        };
        assert!(
            calls.rejected.iter().all(|e| *e == rejection),
            "{name}: {:?}",
            calls.rejected
        );
        for input in calls.accepted {
            let (role, callee) = match name.as_str() {
                "via_producer" | "aliased" | "qualified" => (CallRole::Producer, "produce"),
                "via_consumer" => (CallRole::Consumer, "consume"),
                "via_relay" => (CallRole::Relay, "relay"),
                "same_named_left" => (CallRole::Producer, "left::produce"),
                "same_named_right" => (CallRole::Producer, "right::produce"),
                "unproven" => (CallRole::Relay, "replacement"),
                _ => panic!("unexpected local call {name}"),
            };
            assert_eq!(input.role(), role);
            assert_eq!(tcx.def_path_str(input.callee()), callee);
            assert_eq!(input.definition(), input.callee().to_def_id());
            assert_eq!(input.owner(), owner);
            assert_eq!(input.call().hir_id.owner.def_id, owner);
            assert!(input.arguments().is_empty());
            assert_eq!(
                input.signature(),
                tcx.fn_sig(input.callee())
                    .instantiate(tcx, input.arguments())
                    .skip_binder()
            );
            let ty::Adt(definition, arguments) = input.box_ty().kind() else {
                panic!("standard Box")
            };
            assert_eq!(Some(definition.did()), tcx.lang_items().owned_box());
            assert_eq!(arguments.type_at(0), tcx.types.i32);
            let other = owners.iter().copied().find(|id| *id != owner).unwrap();
            assert!(matches!(
                LocalCallInput::read(tcx, other, input.call()),
                Err(CallError::WrongOwner)
            ));
            let copied = hir::Expr {
                hir_id: input.call().hir_id,
                kind: input.call().kind,
                span: input.call().span,
            };
            assert!(matches!(
                LocalCallInput::read(tcx, owner, &copied),
                Err(CallError::NonCanonicalNode)
            ));
            let second = LocalCallInput::read(tcx, owner, input.call()).unwrap();
            let expected = input.definition();
            let binding = mapping::bindings(mapping::Observe);
            assert_eq!(
                <_ as Supports<OwnedLocalCall>>::mapping(&binding)
                    .lower(&mut context, input)
                    .unwrap(),
                expected
            );
            let reversed = Builder::new()
                .construction(mapping::BoxMapping)
                .local_call(mapping::Observe)
                .build();
            assert_eq!(
                <_ as Supports<OwnedLocalCall>>::mapping(&reversed)
                    .lower(&mut context, second)
                    .unwrap(),
                expected
            );
            identities.insert(name.clone(), expected);
        }
        for input in calls.boxes {
            let expected = input.constructor();
            let binding = mapping::bindings(mapping::Observe);
            assert_eq!(
                <_ as Supports<OwnedBoxConstruction>>::mapping(&binding)
                    .lower(&mut context, input)
                    .unwrap(),
                expected
            );
        }
    }
    assert_eq!(
        inventory,
        BTreeMap::from([
            ("produce".into(), (0, 1, 1)),
            ("produce_return".into(), (0, 1, 1)),
            ("consume".into(), (0, 0, 0)),
            ("relay".into(), (0, 0, 0)),
            ("via_producer".into(), (1, 0, 0)),
            ("via_consumer".into(), (1, 1, 1)),
            ("via_relay".into(), (1, 1, 1)),
            ("aliased".into(), (1, 0, 0)),
            ("qualified".into(), (1, 0, 0)),
            ("result_adjusted".into(), (0, 1, 0)),
            ("reads_ref".into(), (0, 0, 0)),
            ("argument_adjusted".into(), (0, 1, 0)),
            ("indirect".into(), (0, 1, 0)),
            ("computed".into(), (0, 1, 0)),
            ("closure".into(), (0, 1, 0)),
            ("external".into(), (0, 1, 0)),
            ("generic".into(), (0, 0, 0)),
            ("call_generic".into(), (0, 1, 0)),
            ("pair".into(), (0, 1, 1)),
            ("wrong_arity".into(), (0, 1, 0)),
            ("plain".into(), (0, 0, 0)),
            ("wrong_result".into(), (0, 1, 0)),
            ("foreign".into(), (0, 1, 1)),
            ("call_foreign".into(), (0, 1, 0)),
            ("unsigned".into(), (0, 1, 0)),
            ("wrong_payload".into(), (0, 1, 0)),
            ("left::produce".into(), (0, 1, 1)),
            ("right::produce".into(), (0, 1, 1)),
            ("same_named_left".into(), (1, 0, 0)),
            ("same_named_right".into(), (1, 0, 0)),
            ("replacement".into(), (0, 1, 1)),
            ("unproven".into(), (1, 0, 0)),
            ("associated".into(), (0, 1, 0)),
            ("method".into(), (0, 1, 0)),
            ("pointer".into(), (0, 1, 0)),
        ])
    );
    assert_eq!(identities["via_producer"], identities["aliased"]);
    assert_ne!(identities["via_consumer"], identities["via_relay"]);
    assert_eq!(identities["qualified"], identities["aliased"]);
    assert_ne!(
        identities["same_named_left"],
        identities["same_named_right"]
    );
    assert_ne!(identities["same_named_left"], identities["via_producer"]);
    assert_ne!(identities["unproven"], identities["via_relay"]);
    assert_eq!((context.calls, context.boxes), (16, 9));
    println!("three local roles and both registration orders passed");
}
