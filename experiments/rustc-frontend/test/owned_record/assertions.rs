//! Nominal identities and source/declaration field order are checked separately.
use super::mapping::{self, BoxMapping, Context, RecordMapping};
use crate::{
    owned_source::{
        BoxConstructionInput, Builder, ConstructionError, OwnedBoxConstruction,
        record::{OwnedRecordConstruction, RecordConstructionInput, RecordError},
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

struct Records<'tcx> {
    tcx: TyCtxt<'tcx>,
    owner: LocalDefId,
    accepted: Vec<RecordConstructionInput<'tcx>>,
    rejected: Vec<RecordError>,
    boxes: Vec<BoxConstructionInput<'tcx>>,
}
impl<'tcx> Visitor<'tcx> for Records<'tcx> {
    fn visit_expr(&mut self, expression: &'tcx hir::Expr<'tcx>) {
        if matches!(expression.kind, hir::ExprKind::Struct(..)) {
            match RecordConstructionInput::read(self.tcx, self.owner, expression) {
                Ok(input) => self.accepted.push(input),
                Err(error) => self.rejected.push(error),
            }
        }
        if matches!(expression.kind, hir::ExprKind::Call(..))
            && let Ok(input) = BoxConstructionInput::read(self.tcx, self.owner, expression)
        {
            self.boxes.push(input);
        }
        intravisit::walk_expr(self, expression);
    }
}

pub(super) fn check(tcx: TyCtxt<'_>) {
    #[cfg(owned_record_proof)]
    crate::owned_source::record::allocator::check(tcx);
    let owners: Vec<_> = tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
        .collect();
    let mut seen = BTreeSet::new();
    let mut nominal = HashMap::new();
    for &owner in &owners {
        let name = tcx.def_path_str(owner);
        let expected = match name.as_str() {
            "forward" | "alias" | "renamed" | "left::make" | "right::make" => Ok(vec![0, 1]),
            "reversed" | "locals" => Ok(vec![1, 0]),
            "single" => Ok(vec![0]),
            "record_limit_128::make" => Ok((0..128).collect()),
            "record_limit_129::make" => Err(Some(RecordError::UnsupportedRecord)),
            "generic" | "empty" | "tuple" | "enumeration" | "custom" | "representation"
            | "external" | "unit" | "union_record" => Err(Some(RecordError::UnsupportedRecord)),
            "scalar" | "unsigned" | "counterfeit" => Err(Some(RecordError::UnsupportedField)),
            "update" => Err(Some(RecordError::NotCompleteStruct)),
            "no_record" | "allocator_type" => Err(None),
            _ => panic!("unasserted record fixture {name}"),
        };
        let mut records = Records {
            tcx,
            owner,
            accepted: vec![],
            rejected: vec![],
            boxes: vec![],
        };
        records.visit_body(tcx.hir_body_owned_by(owner));
        let mut context = Context {
            tcx,
            boxes: 0,
            records: 0,
        };
        let bindings = mapping::bindings(RecordMapping);
        let box_count = records.boxes.len();
        for input in records.boxes {
            assert_eq!(
                Supports::<OwnedBoxConstruction>::mapping(&bindings)
                    .lower(&mut context, input)
                    .unwrap(),
                tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap()
            );
        }
        assert_eq!(context.boxes, box_count);
        match expected {
            Ok(indices) => {
                assert!(
                    records.rejected.is_empty(),
                    "{name}: {:?}",
                    records.rejected
                );
                assert_eq!(records.accepted.len(), 1, "{name}");
                let input = records.accepted.pop().unwrap();
                assert_eq!(box_count, indices.len());
                assert_eq!(
                    input
                        .fields()
                        .iter()
                        .map(|f| f.index().as_usize())
                        .collect::<Vec<_>>(),
                    indices
                );
                let hir::ExprKind::Struct(_, fields, hir::StructTailExpr::None) =
                    input.expression().kind
                else {
                    panic!("record")
                };
                for (field, source) in input.fields().iter().zip(fields) {
                    assert_eq!(field.index(), tcx.typeck(owner).field_index(source.hir_id));
                    assert!(std::ptr::eq(field.initializer(), source.expr));
                    let declaration = &input.definition().non_enum_variant().fields[field.index()];
                    assert_eq!(field.declaration(), declaration.did);
                    assert_eq!(
                        field.ty(),
                        tcx.try_normalize_erasing_regions(
                            ty::TypingEnv::fully_monomorphized(),
                            declaration.ty(tcx, input.arguments())
                        )
                        .unwrap()
                    );
                    assert_eq!(field.ty(), tcx.typeck(owner).expr_ty(source.expr));
                    let ty::Adt(standard, args) = field.ty().kind() else {
                        panic!("Box")
                    };
                    assert_eq!(Some(standard.did()), tcx.lang_items().owned_box());
                    assert_eq!(args.type_at(0), tcx.types.i32);
                }
                let copied = hir::Expr {
                    hir_id: input.expression().hir_id,
                    kind: input.expression().kind,
                    span: input.expression().span,
                };
                assert!(matches!(
                    RecordConstructionInput::read(tcx, owner, &copied),
                    Err(RecordError::NonCanonicalNode)
                ));
                let other = *owners.iter().find(|id| **id != owner).unwrap();
                assert!(matches!(
                    RecordConstructionInput::read(tcx, other, input.expression()),
                    Err(RecordError::WrongOwner)
                ));
                let expression = input.expression();
                let definition = input.definition().did();
                nominal.insert(name.clone(), definition);
                assert_eq!(
                    Supports::<OwnedRecordConstruction>::mapping(&bindings)
                        .lower(&mut context, input)
                        .unwrap(),
                    definition
                );
                // Register in the opposite order and prove the same two slots.
                let reversed = Builder::new()
                    .record_construction(RecordMapping)
                    .construction(BoxMapping)
                    .build();
                assert_eq!(
                    Supports::<OwnedRecordConstruction>::mapping(&reversed)
                        .lower(
                            &mut context,
                            RecordConstructionInput::read(tcx, owner, expression).unwrap()
                        )
                        .unwrap(),
                    definition
                );
                assert_eq!(context.records, 2);
            }
            Err(error) => {
                assert!(records.accepted.is_empty(), "admitted {name}");
                assert_eq!(
                    records.rejected,
                    error.into_iter().collect::<Vec<_>>(),
                    "{name}"
                );
                assert_eq!(context.records, 0);
            }
        }
        assert!(matches!(
            RecordConstructionInput::read(tcx, owner, tcx.hir_body_owned_by(owner).value),
            Err(RecordError::NotCompleteStruct)
        ));
        assert!(matches!(
            BoxConstructionInput::read(tcx, owner, tcx.hir_body_owned_by(owner).value),
            Err(ConstructionError::NotDirectCall)
        ));
        assert!(seen.insert(name));
    }
    assert_eq!(seen.len(), 25);
    assert_eq!(nominal["forward"], nominal["alias"]);
    assert_eq!(nominal["forward"], nominal["renamed"]);
    assert_ne!(nominal["left::make"], nominal["right::make"]);
    assert_eq!(
        tcx.item_name(nominal["left::make"]),
        tcx.item_name(nominal["right::make"])
    );
    println!("record identities, field ordering and executable bindings passed");
}
