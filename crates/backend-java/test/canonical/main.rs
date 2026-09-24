//! Strict type-owner certification; source-only dependency publication stays closed.
mod fixture;
#[allow(dead_code)]
#[path = "../error_family/fixture.rs"]
mod local_fixture;
#[allow(dead_code)]
#[path = "../error_family/model.rs"]
mod model;
mod native;
mod rejection;

use fixture::*;
use model::certify;
use portable_backend_java::{ast::*, dialect::*};
use portable_codegen::*;

#[test]
fn original_descriptor_and_six_state_family_survive_certification() {
    for maximum in [false, true] {
        let owner = Owner::new(maximum);
        let descriptor = owner.metadata.clone();
        let package = certify(owner.finish()).unwrap();
        assert_eq!(java_canonical_type_package(&package), Some(&descriptor));
        let family = JavaErrorResultFamily::from_certificate(
            package.clone(),
            descriptor.types(),
            descriptor.kinds(),
        )
        .unwrap();
        assert_eq!(family.types(), descriptor.types());
        assert_eq!(
            descriptor.owner(),
            TargetPackageOwner::CanonicalInstance {
                instance: facts(maximum).instance().key(),
                profile: JavaCanonicalTypeProfile::ScalarResultV2,
            }
        );
        assert!(
            JavaDependencyApi::from_certificate(package)
                .unwrap_err()
                .contains("RustCrate namespace")
        );
    }
}

#[test]
fn independent_registries_preserve_bytes_and_disjoint_namespace() {
    let render = |maximum| {
        render_certified_package(
            &JavaStructuralRenderer,
            &certify(Owner::new(maximum).finish()).unwrap(),
        )
        .unwrap()
    };
    let first = render(false);
    let unrelated = render(true);
    assert_ne!(first, unrelated);
    assert_eq!(first, render(false));
    assert_eq!(unrelated, render(true));
    for maximum in [false, true] {
        let reversed = Owner::with_order(maximum, true);
        assert_ne!(reversed.fixture.types, Owner::new(maximum).fixture.types);
        assert_eq!(
            render_certified_package(
                &JavaStructuralRenderer,
                &certify(reversed.finish()).unwrap()
            )
            .unwrap(),
            render(maximum)
        );
    }
    let normal = Owner::new(false).namespace;
    assert_ne!(normal, JavaPackage::RustCrate(7));
    assert_ne!(normal, JavaPackage::Generated);
    assert_eq!(
        normal.name(),
        "org.polyrust.generated.t2.c0000000000000007.r0000000000000001.i32.e0000000000000002"
    );
    let maximum = Owner::new(true);
    assert_eq!(
        maximum.namespace.name(),
        "org.polyrust.generated.t2.cffffffffffffffff.rfffffffffffffffe.i32.efffffffffffffffd"
    );
    assert_eq!(normal.name().len(), maximum.namespace.name().len());
    assert!(maximum.path.split('/').all(|part| part.len() <= 18));
}

#[test]
fn canonical_owners_compile_as_standalone_ordinary_java_packages() {
    native::prove(|_, _| {});
}
