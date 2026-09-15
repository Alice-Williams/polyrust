//! Each fixture has exactly one expected root operation; no vacuous inventory.
use super::mapping::{self, BoxMapping, Context, RecordMapping};
use crate::{
    owned_source::{
        Builder,
        nested_record::{
            NestedError as E, NestedRecordConstructionInput as Input,
            OwnedNestedRecordConstruction as Capability,
        },
    },
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::{self as hir, def::DefKind};
use rustc_middle::ty::TyCtxt;
use std::collections::HashMap;

pub(super) fn check(tcx: TyCtxt<'_>) {
    #[cfg(nested_record_proof)]
    crate::owned_source::nested_record::layout_oracles::check(tcx);
    let owners: Vec<_> = tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
        .collect();
    let mut nominal = HashMap::new();
    let mut seen = 0;
    for &owner in &owners {
        let name = tcx.def_path_str(owner);
        assert!(matches!(
            crate::owned_source::BoxConstructionInput::read(
                tcx,
                owner,
                tcx.hir_body_owned_by(owner).value
            ),
            Err(crate::owned_source::ConstructionError::NotDirectCall)
        ));
        let expected = match name.as_str() {
            "repeated" | "alias" => Ok((vec![0, 1], 2, 4)),
            "reversed" => Ok((vec![1, 0], 2, 4)),
            "mixed" => Ok((vec![0, 1], 2, 3)),
            "deeper" => Ok((vec![0], 3, 5)),
            "distinct" => Ok((vec![0], 2, 2)),
            "explicit_rust" => Ok((vec![0], 2, 2)),
            "static_reference" | "pointer" => Err(E::UnsupportedField),
            "external_nongeneric" | "packed" | "aligned" => Err(E::UnsupportedRecord),
            "nested_budgets::depth_eight" => Ok((vec![0], 8, 8)),
            "nested_budgets::depth_nine" => Err(E::DepthBudget),
            "nested_budgets::fields_128" => Ok(((0..64).collect(), 2, 128)),
            "nested_budgets::fields_129" => Err(E::FieldBudget),
            "flat" => Err(E::FlatRecord),
            "union_record" => Err(E::UnsupportedRecord),
            "counterfeit" => Err(E::UnsupportedField),
            "diverging" => Err(E::Adjustment),
            "update" => Err(E::NotCompleteStruct),
            "generic" | "generic_root" | "reference" | "tuple" | "unit" | "empty"
            | "enumeration" | "custom" | "representation" | "external" => Err(E::UnsupportedRecord),
            "scalar" | "payload" | "boxed_box" | "array" => Err(E::UnsupportedField),
            "allocator_type" => {
                continue;
            }
            _ => panic!("unasserted nested fixture {name}"),
        };
        seen += 1;
        let hir::ExprKind::Block(block, None) = tcx.hir_body_owned_by(owner).value.kind else {
            panic!("root block");
        };
        let expression = block.expr.expect("one tail constructor");
        let input = Input::read(tcx, owner, expression);
        match expected {
            Err(error) => assert!(
                matches!(input, Err(ref actual) if *actual == error),
                "{name}: expected {error:?}"
            ),
            Ok((indices, depth, count)) => {
                let input = input.unwrap_or_else(|e| panic!("{name}: {e:?}"));
                assert_eq!(
                    input
                        .initializers()
                        .iter()
                        .map(|i| i.index().as_usize())
                        .collect::<Vec<_>>(),
                    indices,
                    "initializer source order"
                );
                let hir::ExprKind::Struct(_, fields, hir::StructTailExpr::None) = expression.kind
                else {
                    panic!("complete struct");
                };
                for (init, source) in input.initializers().iter().zip(fields) {
                    assert!(std::ptr::eq(init.expression(), source.expr));
                    assert_eq!(init.index(), tcx.typeck(owner).field_index(source.hir_id));
                    assert_eq!(
                        input.field(init.index()).unwrap().ty(),
                        tcx.typeck(owner).expr_ty(source.expr)
                    );
                }
                assert!(
                    input
                        .field(rustc_abi::FieldIdx::from_usize(
                            input.layout().fields().len()
                        ))
                        .is_none()
                );
                let constructor = tcx.get_diagnostic_item(rustc_span::sym::box_new).unwrap();
                let standard = tcx
                    .fn_sig(constructor)
                    .instantiate(tcx, tcx.mk_args(&[tcx.types.i32.into()]))
                    .skip_binder()
                    .output();
                assert_eq!(
                    mapping::inspect(tcx, input.layout(), standard),
                    (depth, count)
                );
                let copied = hir::Expr {
                    hir_id: expression.hir_id,
                    kind: expression.kind,
                    span: expression.span,
                };
                assert!(matches!(
                    Input::read(tcx, owner, &copied),
                    Err(E::NonCanonicalNode)
                ));
                let other = *owners.iter().find(|id| **id != owner).unwrap();
                assert!(matches!(
                    Input::read(tcx, other, expression),
                    Err(E::WrongOwner)
                ));
                let definition = input.layout().definition().did();
                nominal.insert(name, definition);
                let mut context = Context { tcx, records: 0 };
                assert_eq!(
                    Supports::<Capability>::mapping(&mapping::bindings(RecordMapping))
                        .lower(&mut context, input)
                        .unwrap(),
                    definition
                );
                let reversed = Builder::new()
                    .nested_record(RecordMapping)
                    .construction(BoxMapping)
                    .build();
                assert_eq!(
                    Supports::<Capability>::mapping(&reversed)
                        .lower(&mut context, Input::read(tcx, owner, expression).unwrap())
                        .unwrap(),
                    definition
                );
                assert_eq!(context.records, 2);
            }
        }
        assert!(matches!(
            Input::read(tcx, owner, tcx.hir_body_owned_by(owner).value),
            Err(E::NotCompleteStruct)
        ));
    }
    assert_eq!(seen, 35);
    assert_eq!(nominal["repeated"], nominal["alias"]);
    assert_eq!(nominal["repeated"], nominal["reversed"]);
    assert_ne!(nominal["repeated"], nominal["distinct"]);
    assert_eq!(
        tcx.item_name(nominal["repeated"]),
        tcx.item_name(nominal["distinct"])
    );
    println!("nested identities, budgets and executable bindings passed");
}
