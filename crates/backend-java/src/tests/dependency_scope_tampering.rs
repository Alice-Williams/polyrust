use super::*;
use crate::dialect::JavaQualifiedName;
use portable_codegen::{DependencySpelling, LinkerDialect, TargetTypeRef};

#[test]
fn java_dependency_catalogue_changes_cannot_replace_the_opaque_witness() {
    let owner = owner(7, 42);
    let replacement = super::owner(8, 99);
    let (scope, callable) =
        JavaDependencyScope::new().import(owner.function(f::id(7, 10)).unwrap().clone());
    let package = consumer(9, scope.finish(), &callable);
    let catalogue = JavaDialect.package_symbol_catalogue(&package).unwrap();
    catalogue.verify(&JavaDialect).unwrap();
    for fault in 0..4 {
        let mut changed = catalogue.clone();
        let spec = &mut changed.dependency_callables[0];
        match fault {
            0 => spec.name = f::name("invented"),
            1 => spec.owner = replacement.package_identity().clone(),
            2 => {
                spec.spelling = DependencySpelling::Qualified(JavaQualifiedName::Dependency(
                    replacement.function(f::id(8, 10)).unwrap().path().clone(),
                ))
            }
            3 => spec.signature.return_type = TargetTypeRef::Primitive(JavaPrimitive::Boolean),
            _ => unreachable!(),
        }
        assert!(changed.verify(&JavaDialect).is_err(), "fault {fault}");
    }
}

#[test]
fn resolved_dependency_paths_and_local_aliases_fail_independent_java_verification() {
    let owner = owner(7, 42);
    let replacement = super::owner(8, 99);
    let (scope, callable) =
        JavaDependencyScope::new().import(owner.function(f::id(7, 10)).unwrap().clone());
    let ready = f::certify(consumer(9, scope.finish(), &callable));
    let item = &ready.ast().files()[0].items()[0];
    assert!(JavaDialect.verify_resolved_file_item(item).is_empty());
    for substituted in [
        JavaResolvedName::Qualified(JavaQualifiedName::Dependency(
            replacement.function(f::id(8, 10)).unwrap().path().clone(),
        )),
        JavaResolvedName::Local(callable.function().path().member().clone()),
        JavaResolvedName::DeclaredPath(callable.function().path().clone()),
    ] {
        let mut changed = item.clone();
        changed.names.insert(
            TargetSymbolRef::DependencyCallable(callable.clone()),
            substituted,
        );
        assert!(!JavaDialect.verify_resolved_file_item(&changed).is_empty());
    }
}
