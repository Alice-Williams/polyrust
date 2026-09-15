//! Independent fixture expectations for the actual source/producer correspondence.
use crate::source_capabilities::{Mapping, Supports};
use crate::{
    compatibility, mapping,
    owned_linear::multiple::boxed_records::{BoxedRecordBody, PayloadTransfer, SourceExit},
};
use rustc_hir::{self as hir, def::DefKind};
use rustc_middle::{mir, ty::TyCtxt};

pub(super) fn check(tcx: TyCtxt<'_>) {
    let mut accepted = std::collections::BTreeSet::new();
    let mut rejected = std::collections::BTreeSet::new();
    let mut nominal = std::collections::BTreeMap::new();
    let mut historical = 0;
    for owner in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        let name = tcx.def_path_str(owner);
        #[cfg(boxed_record_flow_proof)]
        if name == "moved" {
            crate::owned_linear::multiple::boxed_records::mutations::check(tcx, owner);
        }
        if name == "tail" {
            historical += 1;
            compatibility::tail(tcx, owner);
            let bindings = crate::previous::bindings(crate::previous::RecordMapping);
            let input = crate::owned_linear::LinearOwnedBody::read(tcx, owner)
                .unwrap()
                .into_construction();
            let expected = input.constructor();
            let mut context = crate::previous::Context {
                tcx,
                boxes: 0,
                records: 0,
            };
            assert_eq!(
                <_ as Supports<crate::owned_source::OwnedBoxConstruction>>::mapping(&bindings)
                    .lower(&mut context, input)
                    .unwrap(),
                expected
            );
            assert_eq!((context.boxes, context.records), (1, 0));
            assert!(BoxedRecordBody::read(tcx, owner).is_err());
            continue;
        }
        if name.starts_with("rejected::") {
            assert!(
                BoxedRecordBody::read(tcx, owner).is_err(),
                "unexpected admission: {name}"
            );
            rejected.insert(name);
            continue;
        }
        if name == "implicit" {
            assert!(matches!(
                BoxedRecordBody::read(tcx, owner),
                Err(crate::owned_linear::LinearError::Read)
            ));
            rejected.insert(name);
            continue;
        }
        let evidence =
            BoxedRecordBody::read(tcx, owner).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        crate::exit_consumer::check(
            tcx,
            owner,
            evidence.scopes().read_scope(),
            evidence.exit(),
            evidence.returning(),
        );
        let body = tcx.mir_drops_elaborated_and_const_checked(owner).borrow();
        let checked = tcx.typeck(owner);
        let hir::ExprKind::Block(root, None) = tcx.hir_body_owned_by(owner).value.kind else {
            panic!("root");
        };
        match evidence.exit() {
            SourceExit::Tail(value) => {
                assert_ne!(name, "explicit");
                assert!(std::ptr::eq(value, root.expr.unwrap()));
            }
            SourceExit::Return { expression, value } => {
                assert_eq!(name, "explicit");
                let hir::StmtKind::Semi(canonical) = root.stmts.last().unwrap().kind else {
                    panic!("return statement")
                };
                assert!(std::ptr::eq(expression, canonical));
                let hir::ExprKind::Ret(Some(canonical_value)) = canonical.kind else {
                    panic!("return")
                };
                assert!(std::ptr::eq(value, canonical_value));
            }
        }
        let terminal = &body.basic_blocks[evidence.returning().block];
        assert_eq!(
            evidence.returning().statement_index,
            terminal.statements.len()
        );
        assert!(matches!(
            terminal.terminator().kind,
            mir::TerminatorKind::Return
        ));
        let hir::ExprKind::Struct(_, initializers, hir::StructTailExpr::None) =
            evidence.literal().kind
        else {
            panic!("literal");
        };
        assert_eq!(evidence.fields().len(), 2);
        assert_eq!(
            evidence.transfer(),
            if name == "copied" {
                PayloadTransfer::Copy
            } else {
                PayloadTransfer::Move
            }
        );
        assert_ne!(evidence.record().1, evidence.argument());
        assert_eq!(
            checked.node_type(evidence.record().0),
            body.local_decls[evidence.record().1].ty
        );
        assert_eq!(evidence.aggregate().block, mir::START_BLOCK);
        assert!(evidence.aggregate().statement_index < evidence.movement().statement_index);
        let reversed = matches!(name.as_str(), "boolean" | "moved");
        for (position, producer) in evidence.fields().iter().enumerate() {
            assert_eq!(
                producer.index().as_usize(),
                if reversed { 1 - position } else { position }
            );
            assert!(std::ptr::eq(
                producer.initializer(),
                initializers[position].expr
            ));
            assert_eq!(
                producer.parameter().1.as_usize(),
                producer.index().as_usize() + if name == "same_type" { 2 } else { 1 }
            );
            assert_eq!(
                checked.node_type(producer.parameter().0),
                body.local_decls[producer.parameter().1].ty
            );
            assert_eq!(producer.staging().1.block, mir::START_BLOCK);
            assert!(producer.staging().1.statement_index < evidence.aggregate().statement_index);
            assert_eq!(
                body.local_decls[producer.staging().0].ty,
                checked.expr_ty(producer.initializer())
            );
        }
        assert_eq!(
            evidence.owners().len(),
            match name.as_str() {
                "moved" => 3,
                "shadow" => 2,
                _ => 1,
            }
        );
        assert_eq!(evidence.moves().len() + 1, evidence.owners().len());
        assert_eq!(
            evidence.selected().index().as_usize(),
            usize::from(name == "boolean")
        );
        assert_eq!(
            tcx.item_name(evidence.selected().declaration()).as_str(),
            if name == "boolean" {
                "enabled"
            } else {
                "number"
            }
        );
        assert_eq!(
            evidence.selected().ty(),
            if name == "boolean" {
                tcx.types.bool
            } else {
                tcx.types.i32
            }
        );
        assert_eq!(
            evidence.selected().kind(),
            if name == "boolean" {
                crate::owned_source::scalar_record::ScalarKind::Bool
            } else {
                crate::owned_source::scalar_record::ScalarKind::I32
            }
        );
        assert!(matches!(
            body.local_decls[evidence.pointer()].ty.kind(),
            rustc_middle::ty::RawPtr(..)
        ));
        assert_eq!(evidence.scalar_read().block, evidence.drop_location().block);
        assert!(evidence.scalar_read().statement_index < evidence.drop_location().statement_index);
        assert_eq!(evidence.scopes().blocks().count(), 1);
        assert_eq!(
            evidence.scopes().bindings().len(),
            evidence.owners().len() + 1
        );
        assert_eq!(
            evidence.scopes().bindings().last().unwrap().1,
            evidence.scopes().read_scope()
        );
        let input = evidence.into_construction();
        let expected = input.payload().definition().did();
        assert!(nominal.insert(name.clone(), expected).is_none());
        let mut context = mapping::Context { tcx, calls: 0 };
        let bindings = mapping::bindings(mapping::Observe);
        assert_eq!(< _ as Supports<crate::owned_source::boxed_record::OwnedScalarRecordBoxConstruction>>::mapping(&bindings).lower(&mut context, input).unwrap(), expected);
        assert_eq!(context.calls, 1);
        accepted.insert(name);
    }
    assert_eq!(historical, 1);
    assert_eq!(nominal["alias"], nominal["number"]);
    assert_eq!(nominal["shadow"], nominal["number"]);
    assert_ne!(nominal["left::same"], nominal["right::same"]);
    assert_ne!(nominal["left::same"], nominal["number"]);
    assert_ne!(nominal["right::same"], nominal["number"]);
    assert_eq!(
        accepted,
        [
            "number",
            "boolean",
            "moved",
            "explicit",
            "copied",
            "alias",
            "shadow",
            "same_type",
            "left::same",
            "right::same"
        ]
        .into_iter()
        .map(String::from)
        .collect()
    );
    assert_eq!(
        rejected,
        [
            "implicit",
            "rejected::constant",
            "rejected::make",
            "rejected::wrapper",
            "rejected::expression",
            "rejected::mutable",
            "rejected::mutation",
            "rejected::borrowed",
            "rejected::direct",
            "rejected::nested",
            "rejected::conditional",
            "rejected::update",
            "rejected::parameter",
            "rejected::extra",
            "rejected::indirect",
            "rejected::generic"
        ]
        .into_iter()
        .map(String::from)
        .collect()
    );
}
