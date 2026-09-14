use super::*;
use portable_codegen::{GeneratedSymbolId, RelativeOutputPath, SourceRole, TargetFile};
use portable_diagnostics::SourceRef;

#[test]
fn imported_call_height_includes_the_complete_owner_path() {
    let functions = (0..127)
        .map(|index| {
            let mut function = f::functions(0).remove(0);
            function.hash += index;
            function.name = f::name(&format!("chain{index}"));
            function
        })
        .collect();
    let package = f::package_with(7, functions, |_, declared, facade| {
        let ids = declared
            .iter()
            .filter_map(|symbol| match symbol {
                GeneratedSymbolId::Callable(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut index = 0;
        for member in &mut facade.members {
            if let JavaMember::Method(method) = member {
                if index > 0 {
                    method.body = Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                        ty: f::int(),
                        precedence: JavaPrecedence::Primary,
                        kind: JavaExprKind::Call {
                            callable: JavaCallableRef::Generated {
                                symbol: ids[index - 1],
                                signature: JavaMethodSignature {
                                    receiver: None,
                                    parameters: vec![],
                                    result: f::int(),
                                    checked_exceptions: vec![],
                                    nullable_result: false,
                                    pure: true,
                                },
                            },
                            receiver: None,
                            arguments: vec![],
                        },
                    }))]));
                }
                index += 1;
            }
        }
    });
    let owner = JavaDependencyApi::from_certificate(f::certify(package)).unwrap();
    let last = owner.function(f::id(7, 136)).unwrap();
    assert_eq!(last.call_height(), 127);
    let (scope, callable) = JavaDependencyScope::new().import(last.clone());
    let exact =
        JavaDependencyApi::from_certificate(f::certify(consumer(8, scope.finish(), &callable)))
            .unwrap();
    let check = |limit| {
        super::super::verification::verify_owners(
            vec![exact.package_identity(), owner.package_identity()],
            &BTreeSet::new(),
            limit,
        )
    };
    assert!(check(2).is_ok());
    let errors = check(1).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == portable_diagnostics::DiagnosticCode::TargetResourceLimit)
    );
    assert_eq!(super::super::verification::MAX_OWNERS, 1024);
    let last = exact.function(f::id(8, 10)).unwrap();
    assert_eq!(last.call_height(), 128);
    let (scope, callable) = JavaDependencyScope::new().import(last.clone());
    let one_over = f::certify(consumer(9, scope.finish(), &callable));
    assert!(
        JavaDependencyApi::from_certificate(one_over)
            .unwrap_err()
            .contains("call height")
    );
}

#[test]
fn transitive_owner_conflicts_and_source_cycles_cannot_hide_behind_an_import() {
    let leaf = owner(7, 42);
    let replacement = owner(7, 99);
    let (scope, callable) =
        JavaDependencyScope::new().import(leaf.function(f::id(7, 10)).unwrap().clone());
    let middle =
        JavaDependencyApi::from_certificate(f::certify(consumer(8, scope.finish(), &callable)))
            .unwrap();
    let (scope, callable) =
        JavaDependencyScope::new().import(middle.function(f::id(8, 10)).unwrap().clone());
    let bindings = scope.finish();
    // The immediate owner is 8, but it requires the original certificate for 7.
    rejected(consumer(7, bindings.clone(), &callable));
    f::certify(consumer(9, bindings, &callable));
    let (scope, callable) =
        JavaDependencyScope::new().import(middle.function(f::id(8, 10)).unwrap().clone());
    let (scope, _) = scope.import(replacement.function(f::id(7, 11)).unwrap().clone());
    rejected(consumer(9, scope.finish(), &callable));
    // An exact shared leaf in a diamond is allowed; no repeated proof needed.
    let (scope, callable) =
        JavaDependencyScope::new().import(middle.function(f::id(8, 10)).unwrap().clone());
    let (scope, _) = scope.import(leaf.function(f::id(7, 11)).unwrap().clone());
    f::certify(consumer(9, scope.finish(), &callable));
}

#[test]
fn independently_frozen_scopes_in_one_original_package_are_rejected() {
    let api = owner(7, 42);
    let scopes = (0..2)
        .map(|_| {
            JavaDependencyScope::new()
                .import(api.function(f::id(7, 10)).unwrap().clone())
                .0
                .finish()
        })
        .collect::<Vec<_>>();
    let package = f::package_with(9, f::functions(0), |builder, _, facade| {
        for (index, dependencies) in scopes.into_iter().enumerate() {
            // Catalogue-only probe: no copied facade is admitted as a certificate.
            builder.file(TargetFile::new(
                RelativeOutputPath::new(format!("extra{index}.java")).unwrap(),
                SourceRole::PublicApi,
                JavaPackage::RustCrate(9),
                JavaFilePlacement::Main,
                vec![JavaFileItem::Type {
                    declared: vec![],
                    conformances: JavaConformanceInventory::structural().into(),
                    dependencies,
                    declaration: Box::new(facade.clone()),
                }],
                JavaSourceFileKind::CompilationUnit,
                SourceRef::logical(["mixed-scope-probe"]),
            ));
        }
    });
    let errors = super::super::verification::catalogue(&package).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("mixes independently frozen"))
    );
}

#[test]
fn dependency_qualification_cannot_be_shadowed_by_a_local() {
    let api = owner(7, 42);
    let (scope, callable) =
        JavaDependencyScope::new().import(api.function(f::id(7, 10)).unwrap().clone());
    let mut functions = f::functions(0);
    functions.truncate(1);
    functions[0].body = JavaBlock::new(vec![
        JavaStmt::Local {
            finality: JavaLocalFinality::Final,
            ty: f::int(),
            name: f::name("org"),
            value: Some(JavaExpr::literal(f::int(), JavaLiteral::I32(0))),
        },
        JavaStmt::Return(Some(call(&callable, vec![]))),
    ]);
    rejected(f::package_with_dependencies(9, functions, scope.finish()));
}

#[test]
fn entirely_unused_owner_closures_still_reject_conflicts_and_overlap() {
    let leaf = owner(7, 42);
    let replacement = owner(7, 99);
    let safe = owner(6, 1);
    let (scope, callable) =
        JavaDependencyScope::new().import(leaf.function(f::id(7, 10)).unwrap().clone());
    let middle =
        JavaDependencyApi::from_certificate(f::certify(consumer(8, scope.finish(), &callable)))
            .unwrap();
    let (scope, safe_call) =
        JavaDependencyScope::new().import(safe.function(f::id(6, 10)).unwrap().clone());
    let (scope, _) = scope.import(middle.function(f::id(8, 10)).unwrap().clone());
    let bindings = scope.finish();
    rejected(consumer(7, bindings.clone(), &safe_call));
    let valid =
        JavaDependencyApi::from_certificate(f::certify(consumer(9, bindings, &safe_call))).unwrap();
    assert_eq!(
        valid
            .dependencies()
            .map(|owner| owner.root().crate_id)
            .collect::<Vec<_>>(),
        [6]
    );
    let (scope, safe_call) =
        JavaDependencyScope::new().import(safe.function(f::id(6, 10)).unwrap().clone());
    let (scope, _) = scope.import(middle.function(f::id(8, 10)).unwrap().clone());
    let (scope, _) = scope.import(replacement.function(f::id(7, 11)).unwrap().clone());
    rejected(consumer(9, scope.finish(), &safe_call));
    // Debug must not recursively expand retained certificates/graphs.
    assert!(format!("{:?}", middle.package_identity()).len() < 256);
    assert!(format!("{:?}", middle.function(f::id(8, 10)).unwrap()).len() < 1024);
}
