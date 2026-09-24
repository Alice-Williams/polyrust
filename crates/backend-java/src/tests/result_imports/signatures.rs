//! Producer arena IDs disappear at export; a relay preserves the original owner.
use super::{fixture, methods};
use crate::{
    ast::*,
    dialect::*,
    tests::{scalar_result_families::publication, source_dependency_fixture as source},
};

#[test]
fn result_imports_signature_phases_retain_nominal_identity_across_relays() {
    let (certificate, types) = publication::canonical_source_fixture();
    let api = JavaDependencyApi::from_certificate_with_results(certificate, &[types]).unwrap();
    let function = api.functions().next().unwrap();
    assert_eq!(
        function.declaration_signature().result,
        JavaType::Reference(JavaTypeName::Generated(types.interface))
    );
    let JavaDependencyType::Result(original) = function.exported_signature().result() else {
        panic!("exported nominal")
    };
    assert_eq!(original.family().package_identity(), api.package_identity());
    assert_eq!(original.role(), JavaResultTypeRole::Interface);
    let (scope, imported) = JavaDependencyScope::new().import(function.clone()).unwrap();
    let JavaType::Reference(JavaTypeName::Imported(bound)) = &imported.signature().result else {
        panic!("bound nominal")
    };
    assert_eq!(bound.original(), original);
    assert!(scope.finish().contains_result_type(bound));
    let middle = methods::relay(&api, 9);
    let leaf = methods::relay(&middle, 10);
    for relay in [&middle, &leaf] {
        let result = relay
            .functions()
            .next()
            .unwrap()
            .exported_signature()
            .result();
        assert_eq!(result, function.exported_signature().result());
        assert_eq!(relay.result_families().len(), 0);
        assert!(!fixture::text(relay.package()).contains("sealed interface"));
    }
}

#[test]
fn result_imports_publication_rejects_unchecked_nullable_passthrough() {
    let api = fixture::owner();
    let family = api.result_families().next().unwrap();
    let (scope, ty) = JavaDependencyScope::new()
        .import_result_type(family.ty(JavaResultTypeRole::Interface))
        .unwrap();
    let ty = ty.ty();
    let certificate = source::certify(fixture::consumer(
        scope.finish(),
        ty.clone(),
        vec![ty.clone()],
        JavaExpr::local(ty, source::name("p0")),
    ));
    // Valid Java syntax is not enough to promise a non-null published result.
    let error = JavaDependencyApi::from_certificate(certificate).unwrap_err();
    assert!(error.contains("non-null boundary"), "{error}");
}

#[test]
fn result_imports_publication_keeps_general_downcasts_closed() {
    let api = fixture::owner();
    let family = api.result_families().next().unwrap();
    let (scope, interface) = JavaDependencyScope::new()
        .import_result_type(family.ty(JavaResultTypeRole::Interface))
        .unwrap();
    let (scope, success) = scope
        .import_result_type(family.ty(JavaResultTypeRole::Success))
        .unwrap();
    // Java can express this checked downcast, but the narrower source profile
    // requires a guarded variant binding rather than a possible ClassCastException.
    let certificate = source::certify(fixture::consumer(
        scope.finish(),
        success.ty(),
        vec![interface.ty()],
        methods::upcast(
            success.ty(),
            JavaExpr::local(interface.ty(), source::name("p0")),
        ),
    ));
    let error = JavaDependencyApi::from_certificate(certificate).unwrap_err();
    assert!(error.contains("unadmitted expression"), "{error}");
}

#[test]
fn result_imports_function_arguments_and_transitive_owners_keep_original_identity() {
    let (owner, imported) = methods::certificates();
    let middle = JavaDependencyApi::from_certificate(imported).unwrap();
    let observe = middle
        .functions()
        .find(|function| function.path().member().as_str() == "observe")
        .unwrap();
    let (scope, function) = JavaDependencyScope::new().import(observe.clone()).unwrap();
    let other = owner
        .result_families()
        .nth(1)
        .unwrap()
        .ty(JavaResultTypeRole::Error);
    let (scope, constructor) = scope
        .import_result_constructor(other.constructor().unwrap())
        .unwrap();
    let value = JavaExpr {
        ty: source::int(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Dependency(function),
            receiver: None,
            arguments: vec![
                methods::new(&constructor, vec![]),
                JavaExpr::literal(source::int(), JavaLiteral::I32(17)),
            ],
        },
    };
    fixture::reject(fixture::consumer_at(
        10,
        scope.finish(),
        source::int(),
        vec![],
        value,
    ));
    let (scope, _) = JavaDependencyScope::new().import(observe.clone()).unwrap();
    let replacement = fixture::owner()
        .result_families()
        .next()
        .unwrap()
        .ty(JavaResultTypeRole::Interface);
    assert!(scope.import_result_type(replacement).is_err());
}
