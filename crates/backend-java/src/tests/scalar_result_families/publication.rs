//! Local representation certification never opens the source dependency API.
use super::{expressions::*, fixture::*};
use crate::ast::*;
use crate::dialect::{JavaDependencyApi, JavaDialect, JavaScalarResultFamily};
use crate::tests::source_record_fixture::{add_file, id, name, origin, source};
use portable_codegen::*;
use std::sync::Arc;

#[test]
fn scalar_result_canonical_source_facade_cannot_publish_public_nominal_signature() {
    // Positive control: the ordinary canonical scalar-source facade is admitted.
    let scalar = crate::tests::source_dependency_fixture::package(
        7,
        crate::tests::source_dependency_fixture::functions(42),
    );
    JavaDependencyApi::from_certificate(certify(scalar)).unwrap();
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let facade = builder.generated_type(GeneratedType {
        name: "Generated".into(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
        source: source(),
    });
    let family = Family::new(&mut builder, "");
    let types = family.types;
    let result = reference(types.interface);
    // Descriptive source fixture metadata only, not a claim of rustc analysis.
    // The three generated family types deliberately receive no Rust identities.
    let mut metadata = origin();
    metadata.declaration = id(10);
    metadata.visibility = RustVisibility::Public;
    metadata.externally_reachable = true;
    Arc::make_mut(&mut metadata.crate_exports)
        .modules
        .get_mut(&id(1))
        .unwrap()
        .insert(
            RustExportName {
                namespace: RustExportNamespace::Value,
                name: "create".into(),
            },
            RustExportTarget::Declaration(id(10)),
        );
    let callable = builder.callable(GeneratedCallable {
        name: "create".into(),
        signature: JavaDialect.coarse_signature(&signature(vec![], result.clone())),
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::RustSource(Arc::new(metadata)),
        source: source(),
    });
    let mut declaration = declaration(facade, "Generated", JavaDeclarationKind::FinalClass);
    declaration
        .members
        .push(JavaMember::Constructor(JavaConstructor {
            modifiers: vec![JavaModifier::Private],
            name: name("Generated"),
            parameters: vec![],
            body: JavaBlock::new(vec![]),
        }));
    // Visit this source method before the family declarations: rejection must
    // specifically be the public nominal signature, not an unrelated type gate.
    declaration.members.push(JavaMember::Method(JavaMethod {
        declared: JavaMethodDeclaration::Callable(callable),
        annotations: vec![],
        modifiers: vec![JavaModifier::Public, JavaModifier::Static],
        type_parameters: vec![],
        return_type: result,
        name: name("create"),
        parameters: vec![],
        body: Some(JavaBlock::new(vec![returned(upcast(
            new(types.error, vec![]),
            types.error,
            types.interface,
        ))])),
    }));
    declaration.members.extend(
        family
            .declarations()
            .into_iter()
            .map(JavaMember::NestedType),
    );
    let mut declared = vec![GeneratedSymbolId::Callable(callable)];
    declared
        .extend([facade, types.interface, types.success, types.error].map(GeneratedSymbolId::Type));
    add_file(&mut builder, "Generated", declared, declaration);
    let certificate = certify(builder.build());
    JavaScalarResultFamily::from_certificate(certificate.clone(), types).unwrap();
    let error = JavaDependencyApi::from_certificate(certificate).unwrap_err();
    assert_eq!(
        error,
        "Java dependency callable visibility/signature/declaration inventory disagrees"
    );
}
