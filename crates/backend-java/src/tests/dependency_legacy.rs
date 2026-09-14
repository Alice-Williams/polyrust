//! General Java certificates are not automatically Rust-source dependency APIs.
use super::source_dependency_fixture::{certify, name};
use crate::{
    ast::*,
    dialect::{JavaDependencyApi, JavaDialect},
};
use portable_codegen::*;
use portable_diagnostics::SourceRef;

fn certificate(module: JavaPackage, spelling: &str) -> RenderReadyPackage<JavaDialect> {
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let source = SourceRef::logical(["legacy-dependency-test"]);
    let id = builder.generated_type(GeneratedType {
        name: spelling.into(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(if spelling == "Generated" {
            SynthesisReason::PackageEntryPoint
        } else {
            SynthesisReason::TestHarness
        }),
        source: source.clone(),
    });
    let file = builder.file(TargetFile::new(
        RelativeOutputPath::new(format!(
            "{}{spelling}.java",
            module.source_directory(JavaFilePlacement::Main)
        ))
        .unwrap(),
        SourceRole::PublicApi,
        module,
        JavaFilePlacement::Main,
        vec![JavaFileItem::Type {
            declared: vec![GeneratedSymbolId::Type(id)],
            conformances: JavaConformanceInventory::structural().into(),
            dependencies: Default::default(),
            declaration: Box::new(JavaTypeDeclaration {
                declared: Some(id),
                kind: JavaDeclarationKind::FinalClass,
                visibility: JavaVisibility::Public,
                modifiers: vec![],
                name: name(spelling),
                type_parameters: vec![],
                record_components: vec![],
                heritage: JavaHeritage::None,
                permits: vec![],
                members: vec![],
            }),
        }],
        JavaSourceFileKind::CompilationUnit,
        source.clone(),
    ));
    builder.group(TargetFileGroup::new(
        FileGroupRole::PublicApi,
        vec![TargetFileMember::Source(file)],
        source,
    ));
    certify(builder.build())
}

#[test]
fn valid_general_java_certificates_cannot_impersonate_the_source_facade() {
    for (module, spelling, diagnostic) in [
        (JavaPackage::Generated, "Generated", "RustCrate namespace"),
        (
            JavaPackage::RustCrate(7),
            "Other",
            "canonical public Generated.java",
        ),
        (
            JavaPackage::RustCrate(7),
            "Generated",
            "no source declarations",
        ),
    ] {
        let ready = certificate(module, spelling);
        let error = JavaDependencyApi::from_certificate(ready).unwrap_err();
        assert!(error.contains(diagnostic), "{module:?}/{spelling}: {error}");
    }
}
