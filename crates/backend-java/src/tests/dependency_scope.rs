use super::*;
use crate::tests::source_dependency_fixture as f;
use crate::{
    ast::*,
    dialect::{JavaDependencyApi, JavaDialect, JavaStructuralRenderer},
};
use portable_codegen::{
    OutputContents, TargetAstPackage, TargetLinker, TargetSymbolRef, certify_resolved_package,
    render_certified_package, verify_unresolved_package,
};

mod graph {
    include!("dependency_scope_graph.rs");
}
mod native {
    include!("dependency_scope_native.rs");
}
mod tampering {
    include!("dependency_scope_tampering.rs");
}

fn owner(crate_id: u64, value: i32) -> JavaDependencyApi {
    JavaDependencyApi::from_certificate(f::certify(f::package(crate_id, f::functions(value))))
        .unwrap()
}
fn call(callable: &JavaImportedCallable, arguments: Vec<JavaExpr>) -> JavaExpr {
    JavaExpr {
        ty: callable.signature().result.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Dependency(callable.clone()),
            receiver: None,
            arguments,
        },
    }
}
fn consumer(
    crate_id: u64,
    bindings: JavaDependencyBindings,
    callable: &JavaImportedCallable,
) -> TargetAstPackage<JavaDialect> {
    let mut functions = f::functions(0);
    functions.truncate(1);
    functions[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(call(callable, vec![])))]);
    f::package_with_dependencies(crate_id, functions, bindings)
}
fn rejected(package: TargetAstPackage<JavaDialect>) {
    if let Ok(verified) = verify_unresolved_package(&JavaDialect, package)
        && let Ok(linked) = TargetLinker::new(JavaDialect).link_ast(&verified)
    {
        assert!(certify_resolved_package(&JavaDialect, linked).is_err());
    }
}

#[test]
fn exact_owner_scope_and_signature_produce_qualified_calls_without_imports() {
    let first = owner(7, 42);
    let unused = owner(8, 99);
    let (scope, callable) = JavaDependencyScope::new()
        .import(first.function(f::id(7, 10)).unwrap().clone())
        .unwrap();
    let (scope, _) = scope
        .import(unused.function(f::id(8, 10)).unwrap().clone())
        .unwrap();
    let bindings = scope.finish();
    assert!(bindings.contains(&callable));
    let package = f::certify(consumer(9, bindings, &callable));
    assert!(
        package
            .ast()
            .files()
            .iter()
            .all(|file| file.imports().is_empty())
    );
    let api = JavaDependencyApi::from_certificate(package.clone()).unwrap();
    assert_eq!(
        api.dependencies().cloned().collect::<Vec<_>>(),
        vec![
            first.package_identity().clone(),
            unused.package_identity().clone()
        ]
    );
    assert_eq!(api.function(f::id(9, 10)).unwrap().call_height(), 2);
    let renderer = JavaStructuralRenderer;
    let rendered = render_certified_package(&renderer, &package).unwrap();
    for _ in 0..3 {
        assert_eq!(
            rendered,
            render_certified_package(&renderer, &package).unwrap()
        );
    }
    let OutputContents::Text(text) = rendered.files()[0].contents() else {
        panic!("Java text");
    };
    assert!(
        text.contains("org.polyrust.generated.r0000000000000007.Generated.fn000000000000000a()")
    );
    assert!(!text.contains("import "));
    assert!(!text.contains("r0000000000000008"));
    assert!(!text.contains("return 42"));
}

#[test]
fn empty_and_wrong_scopes_cannot_authorize_a_matching_owner_function() {
    let api = owner(7, 42);
    let function = api.function(f::id(7, 10)).unwrap().clone();
    let (scope, callable) = JavaDependencyScope::new().import(function.clone()).unwrap();
    let expected = scope.finish();
    let (other, other_callable) = JavaDependencyScope::new().import(function).unwrap();
    let other = other.finish();
    assert_ne!(expected, other);
    assert_ne!(callable, other_callable);
    rejected(consumer(9, Default::default(), &callable));
    rejected(consumer(9, other, &callable));
    f::certify(consumer(9, expected.clone(), &callable));
    f::certify(consumer(9, expected, &callable));
}

#[test]
fn conflicting_owner_certificates_and_consumer_overlap_reject_even_unused() {
    let first = owner(7, 42);
    let replacement = owner(7, 99);
    let (scope, _) = JavaDependencyScope::new()
        .import(first.function(f::id(7, 10)).unwrap().clone())
        .unwrap();
    assert!(
        scope
            .import(replacement.function(f::id(7, 11)).unwrap().clone())
            .is_err()
    );
    let (scope, callable) = JavaDependencyScope::new()
        .import(first.function(f::id(7, 10)).unwrap().clone())
        .unwrap();
    rejected(consumer(7, scope.finish(), &callable));
}

#[test]
fn identical_member_names_in_distinct_owners_keep_distinct_typed_references() {
    let first = owner(7, 42);
    let second = owner(8, 99);
    let (scope, left) = JavaDependencyScope::new()
        .import(first.function(f::id(7, 10)).unwrap().clone())
        .unwrap();
    let (scope, right) = scope
        .import(second.function(f::id(8, 10)).unwrap().clone())
        .unwrap();
    let mut functions = f::functions(0);
    functions.truncate(1);
    functions[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
        ty: f::int(),
        precedence: JavaPrecedence::Conditional,
        kind: JavaExprKind::Conditional {
            condition: Box::new(JavaExpr::literal(f::boolean(), JavaLiteral::Boolean(true))),
            when_true: Box::new(call(&left, vec![])),
            when_false: Box::new(call(&right, vec![])),
        },
    }))]);
    let package = f::certify(f::package_with_dependencies(9, functions, scope.finish()));
    let item = &package.ast().files()[0].items()[0];
    assert_ne!(
        item.names[&TargetSymbolRef::DependencyCallable(left)],
        item.names[&TargetSymbolRef::DependencyCallable(right)]
    );
    let api = JavaDependencyApi::from_certificate(package).unwrap();
    assert_eq!(
        api.dependencies()
            .map(|owner| owner.root().crate_id)
            .collect::<Vec<_>>(),
        [7, 8]
    );
}

#[test]
fn owner_signature_controls_argument_types_and_result_even_for_valid_handles() {
    let api = owner(7, 42);
    let (scope, callable) = JavaDependencyScope::new()
        .import(api.function(f::id(7, 10)).unwrap().clone())
        .unwrap();
    for bad_result in [false, true] {
        let mut functions = f::functions(0);
        functions.truncate(1);
        let mut value = call(
            &callable,
            if bad_result {
                vec![]
            } else {
                vec![JavaExpr::literal(f::int(), JavaLiteral::I32(1))]
            },
        );
        if bad_result {
            value.ty = f::boolean();
            functions[0].result = f::boolean();
        }
        functions[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(value))]);
        let bindings = JavaDependencyBindings(Some(Arc::new(Frozen {
            values: BTreeSet::new(),
            result_types: BTreeSet::new(),
            result_constructors: BTreeSet::new(),
            result_accessors: BTreeSet::new(),
            identity: scope.identity.clone(),
            functions: scope.functions.clone(),
        })));
        rejected(f::package_with_dependencies(9, functions, bindings));
    }
}

#[test]
fn owner_substitution_with_equal_source_ids_is_not_an_imported_handle() {
    let original = owner(7, 42);
    let replacement = owner(7, 99);
    let (scope, callable) = JavaDependencyScope::new()
        .import(original.function(f::id(7, 10)).unwrap().clone())
        .unwrap();
    let substituted = JavaImportedCallable {
        scope: callable.scope.clone(),
        signature: callable.signature.clone(),
        function: Arc::new(replacement.function(f::id(7, 10)).unwrap().clone()),
    };
    rejected(consumer(9, scope.finish(), &substituted));
}
