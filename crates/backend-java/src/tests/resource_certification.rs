//! Real linked Java packages cannot bypass capacity through direct certification.

use super::tests::{declaration, item, method};
use crate::ast::{JavaFilePlacement, JavaPackage, JavaPrimitive, JavaSourceFileKind, JavaType};
use crate::dialect::JavaDialect;
use portable_codegen::{
    FileGroupRole, LinkedTargetPackage, RelativeOutputPath, SourceRole, TargetAstBuilder,
    TargetDialect, TargetFile, TargetFileGroup, TargetFileMember, TargetLinker,
    certify_resolved_package, render_certified_package, verify_unresolved_package,
};
use portable_diagnostics::{DiagnosticCode, SourceRef};

fn linked_parameter_fixture(count: usize) -> LinkedTargetPackage<JavaDialect> {
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let file = builder.file(TargetFile::new(
        RelativeOutputPath::new("src/main/java/org/polyrust/generated/Fixture.java").unwrap(),
        SourceRole::PublicApi,
        JavaPackage::Generated,
        JavaFilePlacement::Main,
        vec![item(declaration(vec![method(
            JavaType::Primitive(JavaPrimitive::Int),
            count,
            false,
        )]))],
        JavaSourceFileKind::CompilationUnit,
        SourceRef::logical(["resource-certificate", "file"]),
    ));
    builder.group(TargetFileGroup::new(
        FileGroupRole::PublicApi,
        vec![TargetFileMember::Source(file)],
        SourceRef::logical(["resource-certificate", "group"]),
    ));
    let verified = verify_unresolved_package(&JavaDialect, builder.build()).unwrap();
    TargetLinker::new(JavaDialect).link_ast(&verified).unwrap()
}

#[test]
fn direct_certification_checks_java_parameter_capacity_before_render_readiness() {
    for count in [255, 256] {
        let linked = linked_parameter_fixture(count);
        JavaDialect.verify_resolved(&linked).unwrap();
        let result = certify_resolved_package(&JavaDialect, linked);
        if count == 255 {
            let ready = result.expect("exact parameter-slot boundary is render-ready");
            let rendered = render_certified_package(&crate::render::JavaRenderer, &ready).unwrap();
            assert_eq!(rendered.files().len(), 1);
        } else {
            let errors = match result {
                Err(errors) => errors,
                Ok(_) => panic!("one-over slots cannot acquire a render capability"),
            };
            assert!(
                errors
                    .iter()
                    .all(|d| d.code == DiagnosticCode::TargetResourceLimit)
            );
            assert!(errors.iter().any(|d| d.message.contains("parameter slots")));
        }
    }
}
