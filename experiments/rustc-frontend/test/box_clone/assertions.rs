//! An ordinary consumer checks identity, canonical source and executable slots.
use crate::{
    mapping,
    owned_source::{
        BoxConstructionInput, Builder, OwnedBoxConstruction,
        cloning::{BoxCloneInput, CloneError, CloneForm, OwnedBoxClone},
    },
    source_capabilities::{Mapping, Supports},
};
use rustc_hir::{self as hir, intravisit::Visitor};
use rustc_middle::ty::{self, TyCtxt};
use std::collections::BTreeMap;
struct Calls<'tcx> {
    tcx: TyCtxt<'tcx>,
    owner: hir::def_id::LocalDefId,
    accepted: Vec<BoxCloneInput<'tcx>>,
    rejected: Vec<CloneError>,
    boxes: Vec<BoxConstructionInput<'tcx>>,
}
impl<'tcx> Visitor<'tcx> for Calls<'tcx> {
    fn visit_expr(&mut self, expression: &'tcx hir::Expr<'tcx>) {
        if matches!(
            expression.kind,
            hir::ExprKind::Call(..) | hir::ExprKind::MethodCall(..)
        ) {
            match BoxCloneInput::read(self.tcx, self.owner, expression) {
                Ok(input) => self.accepted.push(input),
                Err(error) => self.rejected.push(error),
            }
            if let Ok(input) = BoxConstructionInput::read(self.tcx, self.owner, expression) {
                self.boxes.push(input);
            }
        }
        hir::intravisit::walk_expr(self, expression);
    }
}
pub(super) fn check(tcx: TyCtxt<'_>) {
    let owners: Vec<_> = tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == hir::def::DefKind::Fn)
        .collect();
    let mut inventory = BTreeMap::new();
    let mut context = mapping::Context {
        tcx,
        clones: 0,
        boxes: 0,
    };
    for &owner in &owners {
        let name = tcx.def_path_str(owner);
        let mut calls = Calls {
            tcx,
            owner,
            accepted: Vec::new(),
            rejected: Vec::new(),
            boxes: Vec::new(),
        };
        calls.visit_expr(tcx.hir_body_owned_by(owner).value);
        assert!(
            inventory
                .insert(
                    name.clone(),
                    (
                        calls.accepted.len(),
                        calls.rejected.len(),
                        calls.boxes.len()
                    )
                )
                .is_none()
        );
        for input in calls.accepted {
            assert_eq!(input.owner(), owner);
            assert_eq!(Some(input.definition()), tcx.lang_items().clone_fn());
            assert_eq!(input.arguments().type_at(0), input.box_ty());
            assert_eq!(
                input.signature(),
                tcx.fn_sig(input.definition())
                    .instantiate(tcx, input.arguments())
                    .skip_binder()
            );
            let expected = ty::Instance::try_resolve(
                tcx,
                ty::TypingEnv::post_analysis(tcx, owner),
                input.definition(),
                input.arguments(),
            )
            .unwrap()
            .unwrap();
            assert_eq!(input.instance(), expected);
            assert_eq!(
                tcx.associated_item(expected.def_id()).trait_item_def_id(),
                Some(input.definition())
            );
            let hir::ExprKind::Path(path) = &input.receiver().kind else {
                panic!("local receiver")
            };
            assert!(
                matches!(tcx.typeck(owner).qpath_res(path, input.receiver().hir_id), hir::def::Res::Local(binding) if binding == input.binding())
            );
            for node in [input.call(), input.receiver()] {
                assert_eq!(node.hir_id.owner.def_id, owner);
                assert!(
                    matches!(tcx.hir_node(node.hir_id), hir::Node::Expr(actual) if std::ptr::eq(actual, node))
                );
            }
            match (name.as_str(), input.form()) {
                ("method" | "alias", CloneForm::Method) => {}
                ("qualified" | "inferred", CloneForm::ExplicitBorrow(borrow)) => {
                    assert!(
                        matches!(borrow.kind, hir::ExprKind::AddrOf(hir::BorrowKind::Ref, hir::Mutability::Not, receiver) if std::ptr::eq(receiver, input.receiver()))
                    );
                    assert!(
                        matches!(tcx.hir_node(borrow.hir_id), hir::Node::Expr(actual) if std::ptr::eq(actual, borrow))
                    );
                }
                _ => panic!("unexpected clone form in {name}"),
            }
            let other = owners
                .iter()
                .copied()
                .find(|other| *other != owner)
                .unwrap();
            assert!(matches!(
                BoxCloneInput::read(tcx, other, input.call()),
                Err(CloneError::WrongOwner)
            ));
            let copied = hir::Expr {
                hir_id: input.call().hir_id,
                kind: input.call().kind,
                span: input.call().span,
            };
            assert!(matches!(
                BoxCloneInput::read(tcx, owner, &copied),
                Err(CloneError::NonCanonicalNode)
            ));
            let again = BoxCloneInput::read(tcx, owner, input.call()).unwrap();
            let first = mapping::bindings(mapping::Observe);
            assert_eq!(
                Supports::<OwnedBoxClone>::mapping(&first)
                    .lower(&mut context, input)
                    .unwrap(),
                expected.def_id()
            );
            let reverse = Builder::new()
                .construction(mapping::BoxMapping)
                .box_clone(mapping::Observe)
                .build();
            assert_eq!(
                Supports::<OwnedBoxClone>::mapping(&reverse)
                    .lower(&mut context, again)
                    .unwrap(),
                expected.def_id()
            );
        }
        for input in calls.boxes {
            let bindings = Builder::new().construction(mapping::BoxMapping).build();
            let expected = input.constructor();
            assert_eq!(
                Supports::<OwnedBoxConstruction>::mapping(&bindings)
                    .lower(&mut context, input)
                    .unwrap(),
                expected
            );
        }
    }
    assert_eq!(
        inventory,
        [
            ("method", (1, 1, 1)),
            ("qualified", (1, 1, 1)),
            ("inferred", (1, 1, 1)),
            ("reference", (0, 2, 1)),
            ("wrong_payload", (0, 2, 0)),
            ("replacement", (0, 3, 2)),
            ("custom", (0, 2, 0)),
            ("alias", (1, 0, 0)),
            ("shared", (0, 1, 0)),
            ("temporary", (0, 2, 1)),
            ("adjusted", (0, 1, 0)),
            ("generic", (0, 1, 0)),
            ("indirect", (0, 1, 0)),
            ("field", (0, 1, 0)),
            ("preborrow", (0, 1, 0)),
            ("custom_trait", (0, 1, 0)),
            ("same_signature", (0, 1, 0)),
        ]
        .into_iter()
        .map(|(name, counts)| (name.to_owned(), counts))
        .collect()
    );
    assert_eq!((context.clones, context.boxes), (8, 7));
    println!("four scalar Box clones and both registration orders passed");
}
