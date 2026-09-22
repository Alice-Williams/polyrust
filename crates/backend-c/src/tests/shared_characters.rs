//! U32 target transport certification, not unchecked Rust character admission.
use super::{dependency_fixture as f, project_c_package};
use crate::{
    ast::*,
    dialect::{CDependencyApi, CDialect, CStructuralRenderer},
};
use portable_codegen::*;

#[path = "shared_characters_fixture.rs"]
mod fixture;
#[path = "shared_characters_native.rs"]
mod native;

fn api(source: &f::Fixture) -> CDependencyApi {
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, f::linked(source)).unwrap(),
    )
    .unwrap()
}
fn chain() -> Vec<CDependencyApi> {
    let first = api(&fixture::build(941, None));
    let identity = first
        .functions()
        .find(|f| f.symbol().as_str().ends_with("_identity"))
        .unwrap()
        .clone();
    let second = api(&fixture::build(942, Some(identity)));
    let third = api(&fixture::build(
        943,
        Some(second.functions().next().unwrap().clone()),
    ));
    vec![first, second, third]
}

fn record() -> RenderReadyPackage<CDialect> {
    let origin = CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter);
    let source = super::package_fixture::with_source_shape(
        origin.clone(),
        origin,
        super::package_fixture::RecordLayout::Implementation,
        CScalarType::U32,
    );
    let ast = project_c_package(source.registry, source.files).unwrap();
    let verified = verify_unresolved_package(&CDialect, ast).unwrap();
    certify_resolved_package(
        &CDialect,
        TargetLinker::new(CDialect).link_ast(&verified).unwrap(),
    )
    .unwrap()
}

#[test]
fn character_private_record_transport_has_typed_fields_and_bounded_output() {
    let certificate = record();
    let output = render_certified_package(&CStructuralRenderer, &certificate).unwrap();
    let text = output
        .files()
        .iter()
        .map(|file| {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            text.as_str()
        })
        .collect::<String>();
    assert!(text.contains("uint32_t poly_secret"));
    assert!((text.len() as u64) <= crate::dialect::c_output_byte_bound(&certificate).unwrap());
}

#[test]
fn characters_certify_compact_transport_imports_and_resource_bounds() {
    let owners = chain();
    assert_eq!(owners[0].functions().count(), 28);
    for (index, owner) in owners.iter().enumerate() {
        assert!(owner.system_libraries().is_empty());
        let output = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        assert_eq!(output.files().len(), 2);
        let mut bytes = 0;
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(text.contains("#include <stdint.h>"));
            assert!(
                !text.contains("runtime") && !text.contains("wchar") && !text.contains("goto ")
            );
            assert!(!text.contains("<math.h>") && !text.contains("<uchar.h>"));
            bytes += text.len() as u64;
        }
        assert!(bytes <= crate::dialect::c_output_byte_bound(owner.package()).unwrap());
        let bound = owner.functions().next().unwrap().stack_bound_bytes();
        assert!(bound > 0);
        if index > 0 {
            assert!(
                bound
                    > owners[index - 1]
                        .functions()
                        .next()
                        .unwrap()
                        .stack_bound_bytes()
            );
        }
    }
    let foreign = fixture::build(
        942,
        Some(
            owners[0]
                .functions()
                .find(|f| f.symbol().as_str().ends_with("_identity"))
                .unwrap()
                .clone(),
        ),
    );
    let local = fixture::build(941, None);
    assert!(
        CExpressions::new(local.registry.registrations())
            .direct(foreign.imported[0].clone())
            .is_err()
    );
}

/// Insert a value before an otherwise valid return; all children must still pass.
fn with_discard(source: &f::Fixture, value: CValue) -> Vec<CSourceFile> {
    let mut files = source.files.clone();
    let d =
        CDeclarations::new(source.registry.registrations(), files[1].identity().clone()).unwrap();
    let mut items = files[1].items().to_vec();
    let CFileItem::Definition(definition) = &items[0] else {
        panic!("definition")
    };
    let CDefinitionKind::Function {
        function,
        linkage,
        parameters,
        body,
    } = definition.kind()
    else {
        panic!("function")
    };
    let s = CStatements::new(source.registry.registrations(), function.clone()).unwrap();
    let mut statements = vec![s.discard(value).unwrap()];
    statements.extend(body.statements().iter().cloned());
    items[0] = CFileItem::Definition(
        d.function_definition(
            function.clone(),
            *linkage,
            parameters.clone(),
            s.block(body.scope().clone(), statements).unwrap(),
        )
        .unwrap(),
    );
    files[1] = d.source_file(items).unwrap();
    files
}

#[test]
fn characters_do_not_bypass_numeric_grammar_types_or_depth_checks() {
    let source = fixture::build(941, None);
    let e = CExpressions::new(source.registry.registrations());
    let value = e
        .literal(CLiteral::Unsigned(CUnsignedLiteral::U32(0x10ffff)))
        .unwrap();
    let boolean = e.literal(CLiteral::Bool(true)).unwrap();
    let signed = e.literal(CLiteral::Signed(CSignedLiteral::I32(0))).unwrap();
    assert!(
        e.conditional(value.clone(), value.clone(), value.clone())
            .is_err()
    );
    assert!(
        e.conditional(boolean.clone(), value.clone(), signed.clone())
            .is_err()
    );
    let mut deep = value.clone();
    for _ in 0..128 {
        deep = e.conditional(boolean.clone(), value.clone(), deep).unwrap();
    }
    for invalid in [
        e.numeric_conversion(CScalarType::U16, value.clone())
            .unwrap(),
        e.binary(CBinaryOperator::Divide, value.clone(), value.clone())
            .unwrap(),
        e.binary(CBinaryOperator::Equal, value.clone(), signed)
            .unwrap(),
        deep,
    ] {
        assert!(
            project_c_package(source.registry.clone(), with_discard(&source, invalid)).is_err()
        );
    }
    // The existing allowed U32 -> I32 conversion still needs a representability proof.
    let too_large = e
        .literal(CLiteral::Unsigned(CUnsignedLiteral::U32(u32::MAX)))
        .unwrap();
    let narrow = e.numeric_conversion(CScalarType::I32, too_large).unwrap();
    let invalid = f::Fixture {
        registry: source.registry.clone(),
        files: with_discard(&source, narrow),
        functions: source.functions.clone(),
        imported: vec![],
    };
    assert!(project_c_package(invalid.registry, invalid.files).is_err());
    // U64 stays an internal intermediate, not a newly admitted public signature.
    let wide = f::fixture(944, &[CScalarType::U64], &[], &[None]);
    assert!(project_c_package(wide.registry, wide.files).is_err());
}

#[test]
fn character_target_certificates_do_not_claim_to_validate_unicode() {
    let source = fixture::build(941, None);
    let e = CExpressions::new(source.registry.registrations());
    for raw in [0xd800, 0xdfff, 0x110000, u32::MAX] {
        let value = e
            .literal(CLiteral::Unsigned(CUnsignedLiteral::U32(raw)))
            .unwrap();
        let widened = f::Fixture {
            registry: source.registry.clone(),
            files: with_discard(&source, value),
            functions: source.functions.clone(),
            imported: vec![],
        };
        api(&widened);
    }
}
