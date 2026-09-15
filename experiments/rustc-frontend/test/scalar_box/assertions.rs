//! Exact source inventories and compiler nominal/scalar-kind identities.
use super::{
    mapping::{self, Observe},
    previous,
};
use crate::{
    owned_source::{
        BoxConstructionInput, Builder, ConstructionError, OwnedBoxConstruction,
        boxed_record::{
            BoxedRecordError as Error, OwnedScalarRecordBoxConstruction, ScalarRecordBoxInput,
        },
        record::{OwnedRecordConstruction, RecordConstructionInput},
        scalar_record::{PayloadError, ScalarKind},
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
use std::collections::{BTreeSet, HashMap};

struct Calls<'tcx> {
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    accepted: Vec<ScalarRecordBoxInput<'tcx>>,
    rejected: Vec<Error>,
    old_boxes: Vec<BoxConstructionInput<'tcx>>,
    old_records: Vec<RecordConstructionInput<'tcx>>,
}
impl<'tcx> Visitor<'tcx> for Calls<'tcx> {
    fn visit_expr(&mut self, expression: &'tcx hir::Expr<'tcx>) {
        if matches!(expression.kind, hir::ExprKind::Call(..)) {
            match ScalarRecordBoxInput::read(self.tcx, self.owner, expression) {
                Ok(input) => self.accepted.push(input),
                Err(error) => self.rejected.push(error),
            }
            if let Ok(input) = BoxConstructionInput::read(self.tcx, self.owner, expression) {
                self.old_boxes.push(input);
            }
        }
        if matches!(expression.kind, hir::ExprKind::Struct(..))
            && let Ok(input) = RecordConstructionInput::read(self.tcx, self.owner, expression)
        {
            self.old_records.push(input);
        }
        intravisit::walk_expr(self, expression);
    }
}
pub(super) fn check(tcx: TyCtxt<'_>) {
    let owners: Vec<_> = tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
        .collect();
    let mut seen = BTreeSet::new();
    let mut nominal = HashMap::new();
    for &owner in &owners {
        let name = tcx.def_path_str(owner);
        let expected = match name.as_str() {
            "mixed" | "alias" | "renamed" | "parameter" | "qualified" | "renamed_box" | "new" => {
                Ok(vec![ScalarKind::I32, ScalarKind::Bool])
            }
            "left::wrap" | "right::wrap" => Ok(vec![ScalarKind::I32]),
            "boolean" => Ok(vec![ScalarKind::Bool]),
            "scalar_box_limits::limit128::wrap" => Ok(vec![ScalarKind::I32; 128]),
            "scalar" | "old_record" => Err(Error::Payload(PayloadError::NotRecord)),
            "owned" | "nested" | "unsigned" => Err(Error::Payload(PayloadError::UnsupportedField)),
            "generic"
            | "empty"
            | "unit"
            | "tuple"
            | "enumeration"
            | "union_record"
            | "external"
            | "custom"
            | "representation"
            | "scalar_box_limits::limit129::wrap" => {
                Err(Error::Payload(PayloadError::UnsupportedRecord))
            }
            "indirect" | "block_callee" => {
                Err(Error::Constructor(ConstructionError::NotDirectCall))
            }
            "wrapper" | "counterfeit" => Err(Error::Constructor(
                ConstructionError::NotStandardConstructor,
            )),
            "adjusted" => Err(Error::Adjustment),
            _ => panic!("unasserted scalar Box fixture {name}"),
        };
        let mut calls = Calls {
            tcx,
            owner,
            accepted: vec![],
            rejected: vec![],
            old_boxes: vec![],
            old_records: vec![],
        };
        calls.visit_body(tcx.hir_body_owned_by(owner));
        let bindings = mapping::bindings(Observe);
        let mut context = mapping::Context { tcx, calls: 0 };
        match expected {
            Ok(kinds) => {
                assert!(calls.rejected.is_empty(), "{name}: {:?}", calls.rejected);
                assert_eq!(calls.accepted.len(), 1, "{name}");
                let input = calls.accepted.pop().unwrap();
                assert!(matches!(
                    BoxConstructionInput::read(tcx, owner, input.call()),
                    Err(ConstructionError::UnsupportedPayload)
                ));
                assert_eq!(
                    input
                        .payload()
                        .fields()
                        .iter()
                        .map(|f| f.kind())
                        .collect::<Vec<_>>(),
                    kinds
                );
                assert!(input.payload().arguments().is_empty());
                assert_eq!(
                    tcx.typeck(owner).expr_ty(input.argument()),
                    input.payload().ty()
                );
                let ty::Adt(box_def, args) = input.result().kind() else {
                    panic!("Box result");
                };
                assert_eq!(Some(box_def.did()), tcx.lang_items().owned_box());
                assert_eq!(args.type_at(0), input.payload().ty());
                for (index, field) in input.payload().fields().iter().enumerate() {
                    assert_eq!(field.index().as_usize(), index);
                    let declaration =
                        &input.payload().definition().non_enum_variant().fields[field.index()];
                    assert_eq!(field.declaration(), declaration.did);
                    assert_eq!(
                        field.ty(),
                        tcx.try_normalize_erasing_regions(
                            ty::TypingEnv::fully_monomorphized(),
                            declaration.ty(tcx, input.payload().arguments())
                        )
                        .unwrap()
                    );
                    assert_eq!(
                        field.ty(),
                        match field.kind() {
                            ScalarKind::I32 => tcx.types.i32,
                            ScalarKind::Bool => tcx.types.bool,
                        }
                    );
                }
                let copied = hir::Expr {
                    hir_id: input.call().hir_id,
                    kind: input.call().kind,
                    span: input.call().span,
                };
                assert!(matches!(
                    ScalarRecordBoxInput::read(tcx, owner, &copied),
                    Err(Error::Constructor(ConstructionError::NonCanonicalNode))
                ));
                let other = *owners.iter().find(|id| **id != owner).unwrap();
                assert!(matches!(
                    ScalarRecordBoxInput::read(tcx, other, input.call()),
                    Err(Error::Constructor(ConstructionError::WrongOwner))
                ));
                let call = input.call();
                let definition = input.payload().definition().did();
                nominal.insert(name.clone(), definition);
                assert_eq!(
                    Supports::<OwnedScalarRecordBoxConstruction>::mapping(&bindings)
                        .lower(&mut context, input)
                        .unwrap(),
                    definition
                );
                let reversed = Builder::new()
                    .construction(previous::BoxMapping)
                    .record_construction(previous::RecordMapping)
                    .scalar_record_box(Observe)
                    .build();
                assert_eq!(
                    Supports::<OwnedScalarRecordBoxConstruction>::mapping(&reversed)
                        .lower(
                            &mut context,
                            ScalarRecordBoxInput::read(tcx, owner, call).unwrap()
                        )
                        .unwrap(),
                    definition
                );
                assert_eq!(context.calls, 2);
            }
            Err(error) => {
                assert!(calls.accepted.is_empty(), "admitted {name}");
                assert_eq!(calls.rejected, [error], "{name}");
                assert_eq!(context.calls, 0);
            }
        }
        let mut old = previous::Context {
            tcx,
            boxes: 0,
            records: 0,
        };
        for input in calls.old_boxes {
            Supports::<OwnedBoxConstruction>::mapping(&bindings)
                .lower(&mut old, input)
                .unwrap();
        }
        for input in calls.old_records {
            let expression = input.expression();
            Supports::<OwnedRecordConstruction>::mapping(&bindings)
                .lower(&mut old, input)
                .unwrap();
            let prior = previous::bindings(previous::RecordMapping);
            Supports::<OwnedRecordConstruction>::mapping(&prior)
                .lower(
                    &mut old,
                    RecordConstructionInput::read(tcx, owner, expression).unwrap(),
                )
                .unwrap();
        }
        assert_eq!(
            old.boxes,
            usize::from(matches!(name.as_str(), "scalar" | "old_record")),
            "{name}"
        );
        assert_eq!(old.records, if name == "old_record" { 2 } else { 0 });
        assert!(seen.insert(name));
    }
    assert_eq!(seen.len(), 31);
    assert_eq!(nominal["mixed"], nominal["alias"]);
    assert_eq!(nominal["mixed"], nominal["renamed"]);
    assert_ne!(nominal["left::wrap"], nominal["right::wrap"]);
    assert_eq!(
        tcx.item_name(nominal["left::wrap"]),
        tcx.item_name(nominal["right::wrap"])
    );
    println!("scalar-record identities, closed kinds and three executable slots passed");
}
