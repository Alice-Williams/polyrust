//! Primitive doubles pass through immutable record storage and a private call.
use super::super::package_fixture::{self, RecordLayout};
use super::*;

pub(super) fn record() -> RenderReadyPackage<CDialect> {
    let origin = CGeneratedOrigin::Synthesized(CSynthesisReason::OwnershipAdapter);
    let fixture = package_fixture::with_source_shape(
        origin.clone(),
        origin,
        RecordLayout::Implementation,
        CScalarType::F64,
    );
    let draft = super::super::project_c_package(fixture.registry, fixture.files).unwrap();
    let verified = verify_unresolved_package(&CDialect, draft).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&verified).unwrap();
    certify_resolved_package(&CDialect, linked).unwrap()
}

#[test]
fn record_and_local_double_transport_certifies() {
    let certificate = record();
    let output = render_certified_package(&CStructuralRenderer, &certificate).unwrap();
    let text: String = output
        .files()
        .iter()
        .map(|file| {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            text.as_str()
        })
        .collect();
    assert!(text.contains("double poly_secret"));
    assert!(text.contains("double poly_result"));
    let size: usize = output
        .files()
        .iter()
        .map(|file| {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            text.len()
        })
        .sum();
    assert!(size as u64 <= crate::dialect::c_output_byte_bound(&certificate).unwrap());
}
