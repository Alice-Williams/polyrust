//! Observe actual package measurement, not a caller-supplied zero-cost claim.
use crate::ast::*;
use crate::dialect::{CDialect, project_c_package};
use portable_codegen::*;

fn fixture() -> (CRegistry, Vec<CSourceFile>) {
    let ids: [RustDeclarationId; 15] = std::array::from_fn(|index| RustDeclarationId {
        crate_id: 19,
        definition_path_hash: index as u64,
    });
    let instance = RustCanonicalInstanceFacts::new(
        RustCanonicalInstanceKey::i32_try_from_int_error_result(ids[1], ids[2]).unwrap(),
        ids[0],
        RustResultVariantFacts {
            variant: ids[3],
            payload: ids[5],
        },
        RustResultVariantFacts {
            variant: ids[4],
            payload: ids[6],
        },
    )
    .unwrap();
    let facts = RustCanonicalErrorKindFacts::new(
        instance,
        ids[7],
        ids[8],
        RustIntegerErrorVariants {
            empty: ids[9],
            invalid_digit: ids[10],
            positive_overflow: ids[11],
            negative_overflow: ids[12],
            zero: ids[13],
            not_a_power_of_two: ids[14],
        },
    )
    .unwrap();
    let profile = CCanonicalTypeProfile::ScalarResultV2;
    let base = profile.basename(instance.key());
    let mut registry = CRegistry::new();
    let header = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("{base}.h")).unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    let implementation = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(format!("{base}.c")).unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap();
    let key = |role| profile.declaration_key(instance.key(), role).unwrap();
    let record = registry
        .declare_struct(&header, key(CCanonicalTypeRole::Result))
        .unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let tag = registry
        .register_member(
            &owner,
            key(CCanonicalTypeRole::SuccessTag),
            CObjectType::scalar(CScalarType::Bool),
        )
        .unwrap();
    let value = registry
        .register_member(
            &owner,
            key(CCanonicalTypeRole::Value),
            CObjectType::scalar(CScalarType::I32),
        )
        .unwrap();
    registry.define_aggregate(&owner, vec![tag, value]).unwrap();
    registry
        .register_canonical_type_package(&header, &implementation, profile, facts, &record)
        .unwrap();
    let d = CDeclarations::new(&registry, header).unwrap();
    let header = d
        .source_file(vec![CFileItem::Declaration(d.aggregate(owner).unwrap())])
        .unwrap();
    let implementation = CDeclarations::new(&registry, implementation)
        .unwrap()
        .source_file(vec![])
        .unwrap();
    (registry, vec![header, implementation])
}

#[test]
fn canonical_type_owner_has_zero_measured_executable_contribution() {
    let (registry, files) = fixture();
    let ast = project_c_package(registry.freeze(), files).unwrap();
    let checked = verify_unresolved_package(&CDialect, ast).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let measured = super::measure_package(&linked).unwrap();
    assert_eq!(measured.total.frame_bound, 0);
    assert_eq!(measured.total.automatic_bytes, 0);
    assert_eq!(measured.total.automatic_objects, 0);
    assert!(measured.total.function_frames.is_empty());
    assert!(measured.total.source_bound > 0);
    assert_eq!(measured.files.len(), 2);
    for file in measured.files.values() {
        assert_eq!(file.frame_bound, 0);
        assert_eq!(file.automatic_bytes, 0);
        assert!(file.function_frames.is_empty());
    }
    certify_resolved_package(&CDialect, linked).unwrap();
}

#[test]
fn canonical_type_owner_rejects_even_unused_certified_foreign_inventory() {
    use super::super::result_fixture::{Mutation, PublicApi, public_fixture};
    let (registry, files) = public_fixture(Mutation::None, PublicApi::ResultSignatures);
    let ast = project_c_package(registry, files).unwrap();
    let checked = verify_unresolved_package(&CDialect, ast).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let certificate = certify_resolved_package(&CDialect, linked).unwrap();
    let api = crate::dialect::CDependencyApi::from_certificate(certificate).unwrap();
    let proof = api.structs().next().unwrap().clone();
    let (mut registry, files) = fixture();
    registry.import_struct(proof).unwrap();
    let error = project_c_package(registry.freeze(), files).unwrap_err();
    assert!(format!("{error:?}").contains("no foreign inventory"));
}
