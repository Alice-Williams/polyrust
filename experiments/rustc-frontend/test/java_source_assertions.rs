//! Read-only assertions over actual compiler facts and production mapping output.
use super::{Reader, TypePlan, Value};
use crate::source_origin::identity;
use portable_backend_java::{ast::*, dialect::JavaDialect};
use portable_codegen::{GeneratedOrigin, RustSourceOrigin, RustVisibility, TargetAstPackage};
use rustc_hir::{
    self as hir,
    def::DefKind,
    def_id::{DefId, LocalDefId},
};
use rustc_middle::ty::{self, TyCtxt, Visibility};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn origin(tcx: TyCtxt<'_>, id: DefId, source: &RustSourceOrigin) {
    assert_eq!(identity(tcx, id), source.declaration);
    assert_eq!(
        source.visibility,
        match tcx.visibility(id) {
            Visibility::Public => RustVisibility::Public,
            Visibility::Restricted(owner) => RustVisibility::RestrictedTo(identity(tcx, owner)),
        }
    );
    assert_eq!(
        source.externally_reachable,
        tcx.effective_visibilities(())
            .is_exported(id.expect_local())
    );
    let attrs: Vec<_> = tcx
        .hir_attrs(tcx.local_def_id_to_hir_id(id.expect_local()))
        .iter()
        .filter_map(|attribute| attribute.doc_str().map(|text| text.as_str().to_owned()))
        .collect();
    assert_eq!(source.documentation, attrs);
    let mut module = id;
    while tcx.def_kind(module) != DefKind::Mod {
        module = tcx.parent(module);
    }
    assert_eq!(source.module, identity(tcx, module));
    assert_eq!(
        source.module_ancestors.last().unwrap().declaration,
        source.module
    );
    for ancestor in source.module_ancestors.iter().rev() {
        assert_eq!(ancestor.declaration, identity(tcx, module));
        let attrs: Vec<_> = tcx
            .hir_attrs(tcx.local_def_id_to_hir_id(module.expect_local()))
            .iter()
            .filter_map(|attribute| attribute.doc_str().map(|text| text.as_str().to_owned()))
            .collect();
        assert_eq!(ancestor.documentation, attrs);
        if let Some(parent) = tcx.opt_parent(module) {
            module = parent;
        }
    }
}

pub(super) fn package(tcx: TyCtxt<'_>, package: &TargetAstPackage<JavaDialect>) {
    assert_eq!(package.files().len(), 1);
    let file = package.files().next().unwrap();
    let [JavaFileItem::Type { declaration, .. }] = file.items() else {
        panic!("one source facade")
    };
    let constructors: Vec<_> = declaration
        .members
        .iter()
        .filter_map(|member| {
            if let JavaMember::Constructor(value) = member {
                Some(value)
            } else {
                None
            }
        })
        .collect();
    let [constructor] = constructors.as_slice() else {
        panic!("one explicit facade constructor")
    };
    assert_eq!(constructor.modifiers, vec![JavaModifier::Private]);
    assert!(constructor.parameters.is_empty());
    assert!(constructor.body.statements.is_empty());
    assert_eq!(constructor.name, declaration.name);
    let functions: BTreeMap<_, _> = tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
        .map(|id| (identity(tcx, id.to_def_id()), id))
        .collect();
    for callable in package.callables() {
        let GeneratedOrigin::RustSource(source) = &callable.origin else {
            panic!("source callable lost identity")
        };
        let id = functions[&source.declaration];
        origin(tcx, id.to_def_id(), source);
        assert_eq!(
            callable.visibility,
            if source.externally_reachable {
                JavaVisibility::Public
            } else {
                JavaVisibility::Private
            }
        );
        println!(
            "CALL\t{}\t{}\t{}",
            tcx.def_path_str(id.to_def_id()),
            callable.name,
            if source.externally_reachable {
                "public"
            } else {
                "private"
            }
        );
    }
    println!("AST_PACKAGE_CHECKED");
}

pub(super) fn record(tcx: TyCtxt<'_>, definition: ty::AdtDef<'_>, record: &JavaTypeDeclaration) {
    assert_eq!(record.kind, JavaDeclarationKind::Record);
    assert_eq!(record.visibility, JavaVisibility::Private);
    assert_eq!(
        record.record_components.len(),
        definition.non_enum_variant().fields.len()
    );
    for (actual, field) in record
        .record_components
        .iter()
        .zip(definition.non_enum_variant().fields.iter())
    {
        let JavaRecordComponentOrigin::RustSource(source) = &actual.origin else {
            panic!("source field identity")
        };
        assert_eq!(source.owner, identity(tcx, definition.did()));
        origin(tcx, field.did, &source.origin);
        let compiler_type = tcx
            .try_normalize_erasing_regions(
                ty::TypingEnv::fully_monomorphized(),
                tcx.type_of(field.did).instantiate_identity(),
            )
            .unwrap();
        let expected = match compiler_type.kind() {
            ty::Int(ty::IntTy::I32) => TypePlan::I32.java_type(),
            ty::Bool => TypePlan::Bool.java_type(),
            _ => panic!("unadmitted field type"),
        };
        assert_eq!(actual.ty, expected);
    }
    println!(
        "RECORD\t{}\t{}",
        tcx.def_path_str(definition.did()),
        record.name.as_str()
    );
}

pub(super) fn borrow(reader: &Reader<'_>, source: ty::Ty<'_>, value: &Value) {
    let mut source = source;
    let mut plan = value.plan();
    let mut depth = 0;
    while let ty::Ref(_, referent, hir::Mutability::Not) = source.kind() {
        let TypePlan::Shared(target) = plan else {
            panic!("erased shared-reference plan")
        };
        source = *referent;
        plan = target;
        depth += 1;
    }
    match (source.kind(), plan) {
        (ty::Int(ty::IntTy::I32), TypePlan::I32) | (ty::Bool, TypePlan::Bool) => {}
        (ty::Adt(definition, _), TypePlan::Record(id)) => {
            assert_eq!(reader.records[&definition.did()].id, *id)
        }
        _ => panic!("incorrect reference referent representation"),
    }
    assert!(depth > 0);
    println!("BORROW_DEPTH\t{depth}");
}

pub(super) fn field_order(
    reader: &Reader<'_>,
    fields: &[hir::ExprField<'_>],
    arguments: &[JavaExpr],
) {
    let mut positions = Vec::new();
    for field in fields {
        let index = reader.checked.field_index(field.hir_id).as_usize();
        let JavaExprKind::Value(JavaValueRef::Local(name)) = &arguments[index].kind else {
            panic!("field was not materialized")
        };
        let position = reader.prelude.iter().position(|statement| matches!(statement, JavaStmt::Local { name: local, .. } if local == name)).unwrap();
        positions.push(position);
    }
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    println!("FIELD_ORDER\t{}", fields.len());
}

pub(super) fn function(reader: &Reader<'_>, id: LocalDefId, body: &JavaBlock) {
    assert!(!reader.scopes.is_empty());
    for (scope, parent) in &reader.scopes {
        assert_eq!(scope.owner.def_id, id);
        if let Some(parent) = parent {
            assert!(reader.scopes.contains_key(parent));
        }
    }
    let mut names: BTreeSet<_> = reader
        .bindings
        .values()
        .map(|place| {
            let expression = place.value().into_expression();
            let JavaExprKind::Value(JavaValueRef::Local(name)) = expression.kind else {
                panic!("parameter binding")
            };
            name.as_str().to_owned()
        })
        .collect();
    fn locals(block: &JavaBlock, names: &mut BTreeSet<String>) {
        for statement in &block.statements {
            match statement {
                JavaStmt::Local { name, finality, .. } => {
                    assert_eq!(*finality, JavaLocalFinality::Final);
                    assert!(
                        names.insert(name.as_str().to_owned()),
                        "duplicate target local"
                    );
                }
                JavaStmt::If {
                    then_block,
                    else_block,
                    ..
                } => {
                    locals(then_block, names);
                    locals(else_block.as_ref().unwrap(), names);
                }
                JavaStmt::Return(Some(_)) => {}
                _ => panic!("unadmitted Java source statement"),
            }
        }
    }
    locals(body, &mut names);
    println!("SCOPES_CHECKED\t{}", reader.scopes.len());
}
