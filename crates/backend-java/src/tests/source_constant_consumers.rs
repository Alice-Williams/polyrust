use super::{
    source_constant_consumer_fixture as f, source_constant_fixture as c,
    source_dependency_fixture as source,
};
use crate::{ast::*, dialect::*};
use portable_codegen::*;

#[test]
fn authenticated_values_and_mixed_calls_keep_exact_catalogues_and_paths() {
    for mixed in [false, true] {
        let fixture = f::Fixture::new(mixed);
        for used in [false, true] {
            let draft = fixture.draft(used);
            let catalogue = JavaDialect.package_symbol_catalogue(&draft).unwrap();
            assert_eq!(catalogue.dependency_values.len(), 8);
            assert_eq!(catalogue.dependency_callables.len(), usize::from(mixed));
            catalogue.verify(&JavaDialect).unwrap();
            for value in &fixture.values {
                let spec = JavaDialect.dependency_value_spec(value);
                assert_eq!(&spec.owner, fixture.owner.package_identity());
                assert_eq!(spec.ty, JavaDialect.registered_type(value.ty()));
                assert!(fixture.bindings.contains_value(value));
            }
            let api = c::admit(draft).unwrap();
            assert_eq!(api.dependencies().len(), 1);
            assert_eq!(
                api.dependencies().next().unwrap(),
                fixture.owner.package_identity()
            );
            assert_eq!(api.constants().len(), 0); // No foreign field becomes an owned field.
            let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
            let OutputContents::Text(text) = output.files()[0].contents() else {
                panic!()
            };
            assert_eq!(text.contains(".constant0"), used);
            assert!(!text.contains("import ") && !text.contains("Runtime"));
            assert!(api.source_byte_bound().unwrap() >= text.len() as u64);
        }
    }
}
#[test]
fn wrong_scope_missing_registration_mistyped_values_and_writes_reject() {
    let fixture = f::Fixture::new(false);
    let foreign = f::Fixture::new(false);
    for bindings in [JavaDependencyBindings::default(), foreign.bindings] {
        assert!(c::admit(f::consumer(0x500, bindings, &fixture.values, None)).is_err());
    }
    for write in [false, true] {
        let value = &fixture.values[0];
        let mut read = f::read(value);
        let body = if write {
            JavaBlock::new(vec![
                JavaStmt::Assign {
                    target: read.clone(),
                    value: JavaExpr::literal(value.ty().clone(), JavaLiteral::Boolean(true)),
                },
                JavaStmt::Return(Some(read)),
            ])
        } else {
            read.ty = source::int();
            JavaBlock::new(vec![JavaStmt::Return(Some(read))])
        };
        let functions = vec![source::Function {
            hash: 20,
            public: true,
            name: source::name("bad"),
            parameters: vec![],
            result: if write {
                value.ty().clone()
            } else {
                source::int()
            },
            body,
        }];
        assert!(
            c::admit(source::package_with_dependencies(
                0x500,
                functions,
                fixture.bindings.clone()
            ))
            .is_err()
        );
    }
}
#[test]
fn mutable_catalogue_rows_and_resolved_paths_cannot_replace_witnesses() {
    let fixture = f::Fixture::new(false);
    let draft = fixture.draft(true);
    let catalogue = JavaDialect.package_symbol_catalogue(&draft).unwrap();
    let other = c::api(false);
    for fault in 0..5 {
        let mut changed = catalogue.clone();
        let spec = &mut changed.dependency_values[0];
        match fault {
            0 => spec.name = source::name("forged"),
            1 => spec.owner = other.package_identity().clone(),
            2 => spec.ty = JavaDialect.registered_type(&source::int()),
            3 => {
                spec.spelling = DependencySpelling::Qualified(JavaQualifiedName::Dependency(
                    fixture.values[1].constant().path().clone(),
                ))
            }
            4 => {
                spec.name = source::name("forged");
                let mut path = fixture.values[0].constant().path().clone();
                path.member = source::name("forged");
                spec.spelling = DependencySpelling::Qualified(JavaQualifiedName::Dependency(path));
            }
            _ => unreachable!(),
        }
        assert!(changed.verify(&JavaDialect).is_err(), "catalogue {fault}");
    }
    let api = c::admit(draft).unwrap();
    let item = &api.package().ast().files()[0].items()[0];
    for path in [
        JavaResolvedName::DeclaredPath(fixture.values[0].constant().path().clone()),
        JavaResolvedName::Local(source::name("constant0")),
        JavaResolvedName::Qualified(JavaQualifiedName::Dependency(
            fixture.values[1].constant().path().clone(),
        )),
    ] {
        let mut changed = item.clone();
        changed.names.insert(
            TargetSymbolRef::DependencyValue(fixture.values[0].clone()),
            path,
        );
        assert!(!JavaDialect.verify_resolved_file_item(&changed).is_empty());
    }
}
#[test]
fn unused_values_preserve_transitive_owner_conflicts_and_crate_boundaries() {
    let fixture = f::Fixture::new(false);
    let bridge = c::admit(fixture.draft(false)).unwrap();
    assert_eq!(bridge.dependencies().len(), 1);
    let (scope, callable) =
        JavaDependencyScope::new().import(bridge.functions().next().unwrap().clone());
    let other = c::api(false);
    let (scope, _) = scope.import_constant(other.constants().next().unwrap().clone());
    assert!(c::admit(f::consumer(0x501, scope.finish(), &[], Some(&callable))).is_err());
    // A consumer must not impersonate the source namespace of an unused owner.
    assert!(c::admit(f::consumer(0x35c, fixture.bindings, &[], None)).is_err());
}
#[test]
fn same_authority_diamonds_preserve_constant_only_owners_and_real_calls() {
    let fixture = f::Fixture::new(false);
    let bridge = c::admit(fixture.draft(true)).unwrap();
    let (scope, callable) =
        JavaDependencyScope::new().import(bridge.functions().next().unwrap().clone());
    let (scope, value) = scope.import_constant(fixture.owner.constants().next().unwrap().clone());
    let api = c::admit(f::consumer(
        0x501,
        scope.finish(),
        &[value],
        Some(&callable),
    ))
    .unwrap();
    assert_eq!(api.dependencies().len(), 2);
    let bridge_call = api
        .functions()
        .find(|f| f.path().member().as_str() == "callBridge")
        .unwrap();
    assert_eq!(bridge_call.call_height(), 2);
    assert!(fixture.owner.functions().next().is_none());
}
#[test]
fn duplicate_value_imports_are_coherent_in_either_registration_order() {
    let owner = c::api(true);
    for reverse in [false, true] {
        let mut scope = JavaDependencyScope::new();
        let mut values = vec![];
        let function = owner.functions().next().unwrap().clone();
        if reverse {
            scope = scope.import(function.clone()).0;
        }
        for _ in 0..3 {
            let (next, value) = scope.import_constant(owner.constants().next().unwrap().clone());
            scope = next;
            values.push(value);
        }
        if !reverse {
            scope = scope.import(function).0;
        }
        assert_eq!(values[0], values[1]);
        let bindings = scope.finish();
        assert_eq!(bindings.values().count(), 1);
        assert_eq!(bindings.functions().count(), 1);
        c::admit(f::consumer(0x500, bindings, &values, None)).unwrap();
    }
}
