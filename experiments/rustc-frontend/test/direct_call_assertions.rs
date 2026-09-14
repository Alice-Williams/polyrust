//! Independent fixture expectations use compiler origins, never generated names.
use portable_backend_c::ast::*;
use portable_codegen::{RustDeclarationId, RustSourceNode, RustSourceOrigin};
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use std::collections::{BTreeMap, BTreeSet};

fn origin(function: &CFunctionRef) -> &RustSourceOrigin {
    let CGeneratedOrigin::RustSource(origin) = &function.key().origin else {
        panic!("source callable")
    };
    origin
}

fn calls(block: &CBlock) -> Vec<RustDeclarationId> {
    let mut available = BTreeSet::new();
    let mut result = Vec::new();
    for statement in block.statements() {
        if let CStatementKind::Declare(local) = statement.kind() {
            assert_eq!(local.local().scope(), block.scope());
            if let Some(initializer) = local.initializer()
                && let CInitializerKind::Expression(value) = initializer.kind()
                && let CValueKind::Call(call) = value.kind()
            {
                assert_eq!(
                    local.local().key().origin,
                    CGeneratedOrigin::Synthesized(CSynthesisReason::EvaluationTemporary)
                );
                for argument in call.arguments() {
                    let CValueKind::Read(place) = argument.kind() else {
                        panic!("sequenced argument")
                    };
                    let CPlaceKind::Local(temporary) = place.kind() else {
                        panic!("argument temporary")
                    };
                    assert!(available.contains(temporary));
                    assert_eq!(
                        temporary.key().origin,
                        CGeneratedOrigin::Synthesized(CSynthesisReason::EvaluationTemporary)
                    );
                }
                let CCallableKind::Direct(function) = call.callable().kind() else {
                    panic!("direct call")
                };
                result.push(origin(function).declaration);
            }
            available.insert(local.local().clone());
        }
    }
    result
}

pub(super) fn check(tcx: TyCtxt<'_>, source: &CSourceFile) {
    let mut names = BTreeMap::new();
    for id in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        let hash = tcx.def_path_hash(id.to_def_id());
        let identity = RustDeclarationId {
            crate_id: hash.stable_crate_id().as_u64(),
            definition_path_hash: hash.local_hash().as_u64(),
        };
        let name = tcx.item_name(id.to_def_id()).to_string();
        let name = if name == "select" {
            format!("{}::{name}", tcx.item_name(tcx.parent(id.to_def_id())))
        } else {
            name
        };
        assert!(names.insert(name, identity).is_none());
    }
    assert_eq!(names.len(), 6);
    assert_ne!(names["first::select"], names["second::select"]);
    let expected = |names_in_order: &[&str]| {
        names_in_order
            .iter()
            .map(|name| names[*name])
            .collect::<Vec<_>>()
    };
    let mut prototypes = BTreeSet::new();
    let mut definitions = BTreeSet::new();
    let mut body_started = false;
    for item in source.items() {
        match item {
            CFileItem::Declaration(declaration) => {
                if let CDeclarationKind::FunctionPrototype { function, linkage } =
                    declaration.kind()
                {
                    assert!(!body_started, "all signatures precede all bodies");
                    let public = origin(function).declaration == names["score"];
                    assert_eq!(
                        *linkage,
                        if public {
                            CLinkage::External
                        } else {
                            CLinkage::Internal
                        }
                    );
                    assert_eq!(origin(function).externally_reachable, public);
                    assert!(prototypes.insert(function.clone()));
                }
            }
            CFileItem::Definition(definition) => {
                body_started = true;
                let CDefinitionKind::Function {
                    function,
                    parameters,
                    body,
                    ..
                } = definition.kind()
                else {
                    panic!()
                };
                assert!(prototypes.contains(function));
                assert!(definitions.insert(function.clone()));
                assert_eq!(parameters.len(), function.signature().parameters().len());
                assert!(matches!(origin(function).node, RustSourceNode::Declaration));
                let id = origin(function).declaration;
                if id == names["score"] {
                    assert_eq!(
                        calls(body),
                        expected(&[
                            "echo",
                            "negative",
                            "zero",
                            "second::select",
                            "negative",
                            "zero",
                            "first::select",
                            "echo",
                            "negative"
                        ])
                    );
                    let CStatementKind::If {
                        then_block,
                        else_block,
                        ..
                    } = body.statements().last().unwrap().kind()
                    else {
                        panic!("root branch")
                    };
                    assert_eq!(calls(then_block), expected(&["echo", "first::select"]));
                    assert_eq!(
                        calls(else_block),
                        expected(&["zero", "first::select", "second::select"])
                    );
                    assert_ne!(then_block.scope(), else_block.scope());
                    assert_eq!(then_block.scope().parent(), Some(body.scope()));
                    assert_eq!(else_block.scope().parent(), Some(body.scope()));
                } else if id == names["negative"] {
                    assert_eq!(calls(body), expected(&["zero"]));
                } else {
                    assert!(calls(body).is_empty());
                }
                if id == names["echo"] || id == names["second::select"] {
                    assert!(
                        origin(function)
                            .location
                            .file
                            .ends_with("direct_calls_out.rs")
                    );
                }
            }
            _ => {}
        }
    }
    assert_eq!(prototypes.len(), 6);
    assert_eq!(prototypes, definitions);
}
