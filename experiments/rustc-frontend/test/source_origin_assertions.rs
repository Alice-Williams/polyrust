use crate::source_origin::{Cache, identity, read};
use portable_codegen::{
    CheckedRustDocumentation, RustSourceNode, RustSourceOrigin, RustVisibility,
};
use rustc_hir::{
    def::{DefKind, Res},
    def_id::CRATE_DEF_ID,
};
use rustc_middle::ty::TyCtxt;
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn check(tcx: TyCtxt<'_>) -> BTreeMap<String, Arc<RustSourceOrigin>> {
    let root = CRATE_DEF_ID.to_def_id();
    let mut cache = Cache::default();
    let exports = cache.exports(tcx).unwrap();
    assert_eq!(exports.root, identity(tcx, root));
    assert!(Arc::ptr_eq(&exports, &cache.exports(tcx).unwrap()));
    let mut origins = BTreeMap::new();
    let mut pending = vec![CRATE_DEF_ID];
    while let Some(module) = pending.pop() {
        for child in tcx.module_children_local(module) {
            let Res::Def(kind, id) = child.res else {
                continue;
            };
            // rustc also lists the implicit standard-library module here.
            // This fixture probe walks local declarations, not dependencies.
            if !id.is_local() {
                continue;
            }
            match kind {
                DefKind::Mod => pending.push(id.expect_local()),
                DefKind::Fn | DefKind::Struct => {
                    let origin = read(
                        tcx,
                        &mut cache,
                        id,
                        RustSourceNode::Declaration,
                        tcx.def_span(id),
                    )
                    .unwrap();
                    assert_eq!(origin.declaration, identity(tcx, id));
                    assert_eq!(origin.module, identity(tcx, module.to_def_id()));
                    assert!(origin.location.file.ends_with("documentation.rs"));
                    assert!(origin.location.line > 0);
                    assert!(Arc::ptr_eq(&exports, &origin.crate_exports));
                    assert!(origins.insert(tcx.def_path_str(id), origin).is_none());
                    if kind == DefKind::Struct {
                        for field in tcx.adt_def(id).all_fields() {
                            let origin = read(
                                tcx,
                                &mut cache,
                                field.did,
                                RustSourceNode::Declaration,
                                tcx.def_span(field.did),
                            )
                            .unwrap();
                            assert!(
                                origins
                                    .insert(tcx.def_path_str(field.did), origin)
                                    .is_none()
                            );
                        }
                    } else {
                        let local = read(
                            tcx,
                            &mut cache,
                            id,
                            RustSourceNode::Binding(17),
                            tcx.def_span(id),
                        )
                        .unwrap();
                        assert!(local.documentation.is_empty());
                        assert!(local.module_ancestors.is_empty());
                        assert_eq!(local.node, RustSourceNode::Binding(17));
                        assert!(Arc::ptr_eq(&exports, &local.crate_exports));
                    }
                }
                _ => panic!("unexpected documentation fixture declaration: {kind:?}"),
            }
        }
    }
    assert_eq!(origins.len(), 5);
    let checked = CheckedRustDocumentation::check(origins.values().map(Arc::as_ref)).unwrap();
    assert_eq!(checked.declarations().count(), 5);
    assert_eq!(checked.modules().count(), 3);
    let first = &origins["first::Ticket"];
    let field = &origins["first::Ticket::value"];
    let second = &origins["second::Ticket"];
    let score = &origins["score"];
    assert_eq!(
        first.documentation,
        [" FIRST_RECORD", "INCLUDED_DOCUMENTATION\n"]
    );
    assert_eq!(field.documentation, [" FIRST_FIELD"]);
    assert_eq!(second.documentation, [" SECOND_RECORD"]);
    assert_eq!(
        origins["second::Ticket::value"].documentation,
        [" SECOND_FIELD"]
    );
    assert_eq!(score.documentation[0], " SCORE_FIRST");
    assert_eq!(score.documentation[1], "SCORE_LINE_ONE\r\nSCORE_LINE_TWO");
    assert_eq!(score.documentation.len(), 3);
    assert_eq!(first.visibility, RustVisibility::Public);
    assert_eq!(second.visibility, RustVisibility::Public);
    assert!(first.externally_reachable);
    assert!(!second.externally_reachable);
    assert!(score.externally_reachable);
    assert!(Arc::ptr_eq(
        &first.module_ancestors,
        &field.module_ancestors
    ));
    let root_docs = &first.module_ancestors[0];
    assert_eq!(root_docs.documentation, ["CRATE_FIRST", "", "CRATE_LAST"]);
    assert!(Arc::ptr_eq(root_docs, &second.module_ancestors[0]));
    assert!(Arc::ptr_eq(root_docs, &score.module_ancestors[0]));
    assert_eq!(first.module_ancestors[1].documentation, [" MODULE_FIRST"]);
    assert_eq!(second.module_ancestors[1].documentation, ["MODULE_SECOND"]);
    origins
}
