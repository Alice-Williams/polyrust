//! Publication and type/member-only uses of original certified closed families.
mod adversarial;
pub(crate) mod fixture;
mod methods;
mod native;
mod signatures;
use crate::{ast::*, dialect::*, tests::source_dependency_fixture as source};
use fixture::*;

#[test]
fn result_imports_family_only_owner_retains_metadata_and_source_reservation() {
    let (draft, types) = owner_package();
    let certificate = source::certify(draft);
    assert!(JavaDependencyApi::from_certificate(certificate.clone()).is_err());
    assert!(
        JavaDependencyApi::from_certificate_with_results(certificate.clone(), &types[..1]).is_err()
    );
    let api = JavaDependencyApi::from_certificate_with_results(certificate, &types).unwrap();
    assert_eq!(api.functions().len(), 0);
    assert_eq!(api.constants().len(), 0);
    assert!(api.source_descriptions().unwrap().is_empty());
    assert!(api.source_byte_bound().unwrap() >= text(api.package()).len() as u64);
    let families = api.result_families().collect::<Vec<_>>();
    assert_eq!(families.len(), 2);
    assert_eq!(families[0].clone(), families[0]);
    assert_ne!(families[0], families[1]);
    assert_ne!(families[0], owner().result_families().next().unwrap());
    let interface = families[0].ty(JavaResultTypeRole::Interface);
    assert!(interface.constructor().is_none());
    assert!(interface.payload_accessor().is_none());
    assert!(
        families[0]
            .ty(JavaResultTypeRole::Error)
            .payload_accessor()
            .is_none()
    );
}

#[test]
fn result_imports_type_only_membership_original_identity_and_atomic_conflicts() {
    let api = owner();
    let family = api.result_families().next().unwrap();
    let original = family.ty(JavaResultTypeRole::Interface);
    let (scope, imported) = JavaDependencyScope::new()
        .import_result_type(original.clone())
        .unwrap();
    let (scope, duplicate) = scope.import_result_type(original.clone()).unwrap();
    assert_eq!(imported, duplicate);
    let bindings = scope.finish();
    assert_eq!(bindings.result_types().count(), 1);
    assert_eq!(bindings.functions().count(), 0);
    let ty = imported.ty();
    let value = JavaExpr::local(ty.clone(), source::name("p0"));
    let package = source::certify(consumer(
        bindings.clone(),
        ty.clone(),
        vec![ty.clone()],
        value.clone(),
    ));
    let output = text(&package);
    assert!(output.contains("org.polyrust.generated.r0000000000000007.Generated.Outcome"));
    assert!(!output.contains("sealed interface"));
    reject(consumer(
        Default::default(),
        ty.clone(),
        vec![ty.clone()],
        value.clone(),
    ));
    let (other, _) = JavaDependencyScope::new()
        .import_result_type(original.clone())
        .unwrap();
    reject(consumer(other.finish(), ty.clone(), vec![ty], value));
    let (scope, _) = JavaDependencyScope::new()
        .import_result_type(original)
        .unwrap();
    let replacement = owner()
        .result_families()
        .next()
        .unwrap()
        .ty(JavaResultTypeRole::Interface);
    assert!(scope.import_result_type(replacement).is_err());
}

#[test]
fn result_imports_constructor_and_accessor_are_original_scope_bound_symbols() {
    let api = owner();
    let family = api.result_families().next().unwrap();
    let success = family.ty(JavaResultTypeRole::Success);
    let (scope, constructor) = JavaDependencyScope::new()
        .import_result_constructor(success.constructor().unwrap())
        .unwrap();
    let (scope, accessor) = scope
        .import_result_accessor(success.payload_accessor().unwrap())
        .unwrap();
    let value = JavaExpr {
        ty: constructor.owner().ty(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::New {
            constructor: JavaConstructorRef::Dependency(constructor.clone()),
            arguments: vec![JavaExpr::literal(source::int(), JavaLiteral::I32(17))],
        },
    };
    let read = JavaExpr {
        ty: source::int(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: accessor.owner().ty(),
                name: accessor.name().clone(),
                signature: Box::new(accessor.signature()),
                origin: JavaMemberOrigin::Dependency(accessor.clone()),
            },
            receiver: Some(Box::new(value)),
            arguments: vec![],
        },
    };
    let bindings = scope.finish();
    assert_eq!(bindings.result_types().count(), 1);
    assert_eq!(bindings.result_constructors().count(), 1);
    assert_eq!(bindings.result_accessors().count(), 1);
    let package = source::certify(consumer(
        bindings.clone(),
        source::int(),
        vec![],
        read.clone(),
    ));
    assert!(
        text(&package)
            .contains("new org.polyrust.generated.r0000000000000007.Generated.Success(17)")
    );
    let (only_type, _) = JavaDependencyScope::new()
        .import_result_type(success)
        .unwrap();
    reject(consumer(
        only_type.finish(),
        source::int(),
        vec![],
        read.clone(),
    ));
    let mut forged = read;
    let JavaExprKind::Call {
        callable: JavaCallableRef::Member { name, .. },
        ..
    } = &mut forged.kind
    else {
        unreachable!()
    };
    *name = source::name("hashCode");
    reject(consumer(bindings, source::int(), vec![], forged));
}
