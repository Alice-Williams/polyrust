//! Compiler facts for every retained description, including private records/fields.
use portable_backend_java::{
    ast::{JavaPrimitive, JavaType},
    dialect::{JavaDependencyApi, JavaSourceDescriptionKind as Kind},
};
use portable_codegen::{RustSourceNode, RustVisibility};
use rustc_hir::def::DefKind;
use rustc_middle::ty::{self, TyCtxt, Visibility};
use std::collections::BTreeMap;

pub(super) fn check(tcx: TyCtxt<'_>, api: &JavaDependencyApi) {
    let definitions: BTreeMap<_, _> = tcx
        .iter_local_def_id()
        .filter(|id| {
            matches!(
                tcx.def_kind(*id),
                DefKind::Fn | DefKind::Struct | DefKind::Field
            )
        })
        .map(|id| (crate::source_origin::identity(tcx, id.to_def_id()), id))
        .collect();
    let mut cache = crate::source_origin::Cache::default();
    // This probe runs the four-crate fixture: every Fn/Struct/Field is retained.
    // Detect a collector omission, not just incorrect facts on returned entries.
    assert_eq!(
        api.source_descriptions()
            .unwrap()
            .iter()
            .map(|d| d.source().declaration)
            .collect::<std::collections::BTreeSet<_>>(),
        definitions.keys().copied().collect()
    );
    for description in api.source_descriptions().unwrap() {
        let source = description.source();
        let id = definitions[&source.declaration];
        let expected = crate::source_origin::read(
            tcx,
            &mut cache,
            id.to_def_id(),
            RustSourceNode::Declaration,
            tcx.def_span(id),
        )
        .unwrap();
        assert_eq!(source, expected.as_ref(), "compiler-origin description");
        let docs: Vec<_> = tcx
            .hir_attrs(tcx.local_def_id_to_hir_id(id))
            .iter()
            .filter_map(|attribute| attribute.doc_str().map(|value| value.as_str().to_owned()))
            .collect();
        assert_eq!(source.documentation, docs, "direct compiler doc attributes");
        assert_eq!(
            source.visibility,
            match tcx.visibility(id) {
                Visibility::Public => RustVisibility::Public,
                Visibility::Restricted(module) =>
                    RustVisibility::RestrictedTo(crate::source_origin::identity(tcx, module)),
            }
        );
        assert_eq!(
            source.externally_reachable,
            tcx.effective_visibilities(()).is_exported(id)
        );
        match description.kind() {
            Kind::Function { parameters, result } => {
                assert_eq!(tcx.def_kind(id), DefKind::Fn);
                let signature = tcx.fn_sig(id).instantiate_identity().skip_binder();
                assert_eq!(*result, scalar(signature.output()));
                assert_eq!(
                    parameters.iter().map(|p| p.ty.clone()).collect::<Vec<_>>(),
                    signature
                        .inputs()
                        .iter()
                        .map(|ty| scalar(*ty))
                        .collect::<Vec<_>>()
                );
            }
            Kind::Record => assert_eq!(tcx.def_kind(id), DefKind::Struct),
            Kind::Field { owner, ty } => {
                assert_eq!(tcx.def_kind(id), DefKind::Field);
                assert_eq!(
                    owner,
                    crate::source_origin::identity(tcx, tcx.parent(id.to_def_id()))
                );
                let field_type = tcx
                    .try_normalize_erasing_regions(
                        ty::TypingEnv::fully_monomorphized(),
                        tcx.type_of(id).instantiate_identity(),
                    )
                    .unwrap();
                assert_eq!(*ty, scalar(field_type));
            }
        }
    }
}

fn scalar(ty: ty::Ty<'_>) -> JavaType {
    JavaType::primitive(match ty.kind() {
        ty::Int(ty::IntTy::I32) => JavaPrimitive::Int,
        ty::Bool => JavaPrimitive::Boolean,
        _ => panic!("compiler scalar description"),
    })
}
