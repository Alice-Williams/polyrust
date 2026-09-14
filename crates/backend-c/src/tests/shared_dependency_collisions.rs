//! Source identities and actual include spellings cannot name conflicting owners.
use super::*;
use fixture::{PackageIdentity, configured};

fn api(crate_id: u64, stem: &str) -> CDependencyApi {
    let package = configured(
        PackageIdentity {
            crate_id,
            file_stem: stem.into(),
            definition_base: 10,
        },
        &[CScalarType::I32],
        &[],
        &[None],
    );
    let certificate = certify_resolved_package(&CDialect, fixture::linked(&package)).unwrap();
    CDependencyApi::from_certificate(certificate).unwrap()
}

#[test]
fn equal_include_spellings_reject_different_owners_in_both_registration_orders() {
    let first = api(70, "first/polyrust_shared");
    let second = api(80, "second/polyrust_shared");
    assert_ne!(
        first.public_header().file().key().path,
        second.public_header().file().key().path
    );
    assert_eq!(
        first.public_header().include_path(),
        second.public_header().include_path()
    );
    for (first, second) in [(&first, &second), (&second, &first)] {
        let proof = first.functions().next().unwrap().clone();
        let mut registry = CRegistry::new();
        registry.import_function(proof.clone()).unwrap();
        assert!(
            registry
                .import_function(second.functions().next().unwrap().clone())
                .is_err()
        );
        assert_eq!(registry.imported_functions().count(), 1);
        let file = CFileKey {
            path: RelativeOutputPath::new("consumer/polyrust_shared.h").unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        };
        assert!(registry.register_file(file.clone()).is_err());
        assert_eq!(registry.files().count(), 0);
        let mut reverse = CRegistry::new();
        reverse.register_file(file).unwrap();
        assert!(reverse.import_function(proof).is_err());
        assert_eq!(reverse.imported_functions().count(), 0);
    }
}

#[test]
fn a_consumer_cannot_claim_the_crate_identity_of_an_imported_package() {
    let api = api(70, "dependency/polyrust_original");
    let dependencies: Vec<_> = api.functions().cloned().collect();
    for call in [None, Some(0)] {
        let consumer = configured(
            PackageIdentity {
                crate_id: 70,
                file_stem: "consumer/polyrust_other".into(),
                definition_base: 100,
            },
            &[CScalarType::I32],
            &dependencies,
            &[call],
        );
        assert_ne!(
            consumer.functions[0].key(),
            dependencies[0].function().key()
        );
        let ast = project_c_package(consumer.registry, consumer.files).unwrap();
        let checked = verify_unresolved_package(&CDialect, ast).unwrap();
        let error = match TargetLinker::new(CDialect).link_ast(&checked) {
            Err(error) => error,
            Ok(_) => panic!("consumer/dependency crate collision was accepted"),
        };
        assert!(format!("{error:?}").contains("consumer crate identity"));
    }
}
