//! Compare the opaque owner API with actual compiler source identities.
use portable_backend_java::dialect::{JavaDependencyApi, JavaDialect};
use portable_codegen::RenderReadyPackage;
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use std::collections::BTreeSet;

pub(super) fn check(tcx: TyCtxt<'_>, certificate: &RenderReadyPackage<JavaDialect>) {
    let api =
        JavaDependencyApi::from_certificate(certificate.clone()).expect("closed Java owner API");
    let mut expected = BTreeSet::new();
    for definition in tcx
        .hir_body_owners()
        .filter(|id| tcx.def_kind(*id) == DefKind::Fn)
    {
        let identity = crate::source_origin::identity(tcx, definition.to_def_id());
        if tcx.effective_visibilities(()).is_exported(definition) {
            expected.insert(identity);
            let function = api
                .function(identity)
                .expect("every compiler export has a certified handle");
            assert_eq!(function.package_identity(), api.package_identity());
            println!(
                "DEPENDENCY_SOURCE\t{}\t{}",
                tcx.def_path_str(definition.to_def_id()),
                function.path().member().as_str()
            );
        } else {
            assert!(
                api.function(identity).is_none(),
                "private helper acquired a public handle"
            );
        }
    }
    assert_eq!(
        api.functions()
            .map(|function| function.declaration())
            .collect::<BTreeSet<_>>(),
        expected
    );
    assert_eq!(
        portable_codegen::render_certified_package(
            &portable_backend_java::dialect::JavaStructuralRenderer,
            api.package()
        )
        .unwrap(),
        portable_codegen::render_certified_package(
            &portable_backend_java::dialect::JavaStructuralRenderer,
            certificate
        )
        .unwrap(),
    );
    println!("DEPENDENCY_API_CHECKED\t{}", api.functions().len());
}
