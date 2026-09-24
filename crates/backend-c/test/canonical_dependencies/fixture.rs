//! 04B-02 test preparation: synthetic target metadata, never compiler evidence.
use portable_backend_c::{ast::*, dialect::*};
use portable_codegen::*;

pub const PROFILE: CCanonicalTypeProfile = CCanonicalTypeProfile::ScalarResultV2;

pub fn facts(offset: u64) -> RustCanonicalErrorKindFacts {
    let ids: [RustDeclarationId; 15] = std::array::from_fn(|index| RustDeclarationId {
        crate_id: 7,
        // Keep a single actual-core provenance anchor across distinct fixture keys.
        definition_path_hash: if index == 0 { 0 } else { offset + index as u64 },
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
    RustCanonicalErrorKindFacts::new(
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
    .unwrap()
}

pub fn certificate(facts: RustCanonicalErrorKindFacts) -> RenderReadyPackage<CDialect> {
    let base = PROFILE.basename(facts.instance().key());
    let key = |role| {
        PROFILE
            .declaration_key(facts.instance().key(), role)
            .unwrap()
    };
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
        .register_canonical_type_package(&header, &implementation, PROFILE, facts, &record)
        .unwrap();
    let d = CDeclarations::new(&registry, header).unwrap();
    let header = d
        .source_file(vec![CFileItem::Declaration(d.aggregate(owner).unwrap())])
        .unwrap();
    let implementation = CDeclarations::new(&registry, implementation)
        .unwrap()
        .source_file(vec![])
        .unwrap();
    let ast = project_c_package(registry.freeze(), vec![header, implementation]).unwrap();
    let checked = verify_unresolved_package(&CDialect, ast).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    certify_resolved_package(&CDialect, linked).unwrap()
}

pub fn api(offset: u64) -> CDependencyApi {
    CDependencyApi::from_certificate(certificate(facts(offset))).unwrap()
}
