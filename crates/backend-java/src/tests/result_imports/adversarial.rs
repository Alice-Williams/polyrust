//! Nominal substitution, signature and resolver mutations must fail closed.
use super::{fixture::*, methods};
use crate::{ast::*, dialect::*, tests::source_dependency_fixture as source};
use portable_codegen::{LinkerDialect, TargetSymbolRef};

#[test]
fn result_imports_reject_same_shaped_family_and_member_substitutions() {
    let api = owner();
    let mut families = api.result_families();
    let first = families.next().unwrap();
    let other = families.next().unwrap();
    let (scope, interface) = JavaDependencyScope::new()
        .import_result_type(first.ty(JavaResultTypeRole::Interface))
        .unwrap();
    let (scope, constructor) = scope
        .import_result_constructor(first.ty(JavaResultTypeRole::Success).constructor().unwrap())
        .unwrap();
    let (scope, accessor) = scope
        .import_result_accessor(
            first
                .ty(JavaResultTypeRole::Success)
                .payload_accessor()
                .unwrap(),
        )
        .unwrap();
    let (scope, other_constructor) = scope
        .import_result_constructor(other.ty(JavaResultTypeRole::Success).constructor().unwrap())
        .unwrap();
    let bindings = scope.finish();
    let number = || JavaExpr::literal(source::int(), JavaLiteral::I32(17));
    let valid = methods::new(&constructor, vec![number()]);
    let other = methods::new(&other_constructor, vec![number()]);
    source::certify(consumer(
        bindings.clone(),
        interface.ty(),
        vec![],
        methods::upcast(interface.ty(), valid.clone()),
    ));
    // Same payload shape, original package and scope do not make two families equal.
    reject(consumer(
        bindings.clone(),
        interface.ty(),
        vec![],
        methods::upcast(interface.ty(), other.clone()),
    ));
    reject(consumer(
        bindings.clone(),
        source::int(),
        vec![],
        methods::read(&accessor, other),
    ));
    for args in [
        vec![],
        vec![number(), number()],
        vec![JavaExpr::literal(
            source::boolean(),
            JavaLiteral::Boolean(true),
        )],
    ] {
        reject(consumer(
            bindings.clone(),
            constructor.owner().ty(),
            vec![],
            methods::new(&constructor, args),
        ));
    }
    let mut wrong_result = valid.clone();
    wrong_result.ty = interface.ty();
    reject(consumer(
        bindings.clone(),
        interface.ty(),
        vec![],
        wrong_result,
    ));
    for fault in 0..4 {
        let mut read = methods::read(&accessor, valid.clone());
        let JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                signature, owner, ..
            },
            arguments,
            ..
        } = &mut read.kind
        else {
            unreachable!()
        };
        match fault {
            0 => signature.result = source::boolean(),
            1 => signature.receiver = Some(interface.ty()),
            2 => *owner = interface.ty(),
            3 => arguments.push(number()),
            _ => unreachable!(),
        }
        reject(consumer(bindings.clone(), source::int(), vec![], read));
    }
}

#[test]
fn result_imports_resolved_nominal_and_member_paths_cannot_be_renamed() {
    let api = owner();
    let family = api.result_families().next().unwrap();
    let success = family.ty(JavaResultTypeRole::Success);
    let (scope, constructor) = JavaDependencyScope::new()
        .import_result_constructor(success.constructor().unwrap())
        .unwrap();
    let (scope, accessor) = scope
        .import_result_accessor(success.payload_accessor().unwrap())
        .unwrap();
    let certificate = source::certify(consumer(
        scope.finish(),
        source::int(),
        vec![],
        methods::read(
            &accessor,
            methods::new(
                &constructor,
                vec![JavaExpr::literal(source::int(), JavaLiteral::I32(17))],
            ),
        ),
    ));
    let original = &certificate.ast().files()[0].items()[0];
    for symbol in [
        TargetSymbolRef::KnownType(constructor.owner().clone().into()),
        TargetSymbolRef::KnownConstructor(constructor.into()),
        TargetSymbolRef::KnownMethod(accessor.into()),
    ] {
        assert!(original.names.contains_key(&symbol));
        let mut changed = original.clone();
        changed
            .names
            .insert(symbol, JavaResolvedName::Local(source::name("invented")));
        assert!(!JavaDialect.verify_resolved_file_item(&changed).is_empty());
    }
}

#[test]
fn result_imports_type_only_closure_rejects_owned_namespace_and_foreign_extensions() {
    let api = owner();
    let (scope, interface) = JavaDependencyScope::new()
        .import_result_type(
            api.result_families()
                .next()
                .unwrap()
                .ty(JavaResultTypeRole::Interface),
        )
        .unwrap();
    let bindings = scope.finish();
    // An unused type-only binding still reserves its original owner's namespace.
    reject(source::package_with_dependencies(
        7,
        vec![],
        bindings.clone(),
    ));
    // A consumer-owned facade cannot implement or add a permit for a foreign family.
    for heritage in [true, false] {
        reject(source::package_configured(
            9,
            vec![],
            |_| {},
            |_, _, declaration| {
                if heritage {
                    declaration.heritage = JavaHeritage::Interfaces(vec![interface.ty()]);
                } else {
                    declaration.permits.push(interface.ty());
                }
            },
            bindings.clone(),
        ));
    }
}
