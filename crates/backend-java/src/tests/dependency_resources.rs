use super::*;
use crate::{
    ast::{JavaBlock, JavaCallableRef, JavaExpr, JavaExprKind, JavaPrecedence, JavaStmt},
    dialect::{JavaDependencyApi, JavaDependencyScope},
    tests::source_dependency_fixture as f,
};
use portable_codegen::{TargetLinker, verify_unresolved_package};

#[test]
fn used_and_unused_registrations_consume_exact_shared_limits() {
    let first =
        JavaDependencyApi::from_certificate(f::certify(f::package(7, f::functions(42)))).unwrap();
    let second =
        JavaDependencyApi::from_certificate(f::certify(f::package(8, f::functions(99)))).unwrap();
    let mut scope = JavaDependencyScope::new();
    let functions = [
        first.function(f::id(7, 10)).unwrap(),
        first.function(f::id(7, 11)).unwrap(),
        second.function(f::id(8, 10)).unwrap(),
    ];
    let names = functions
        .iter()
        .map(|function| function.path().encoded_len())
        .sum();
    let name = functions
        .iter()
        .map(|function| function.path().encoded_len())
        .max()
        .unwrap();
    let mut imported = None;
    for function in functions {
        let (next, callable) = scope.import(function.clone());
        scope = next;
        imported.get_or_insert(callable);
    }
    let bindings = scope.finish();
    for used in [false, true] {
        let mut functions = f::functions(0);
        if used {
            functions[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                ty: f::int(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Call {
                    callable: JavaCallableRef::Dependency(imported.clone().unwrap()),
                    receiver: None,
                    arguments: vec![],
                },
            }))]);
        }
        let draft = f::package_with_dependencies(9, functions, bindings.clone());
        let verified = verify_unresolved_package(&JavaDialect, draft).unwrap();
        let linked = TargetLinker::new(JavaDialect).link_ast(&verified).unwrap();
        assert!(linked.files().iter().all(|file| file.imports().is_empty()));
        let mut limits = Limits {
            bindings: 3,
            owners: 2,
            names,
            name,
        };
        assert!(check(&linked, &limits).is_empty());
        for fault in 0..4 {
            let (counter, needle) = match fault {
                0 => (&mut limits.bindings, "registered dependency bindings"),
                1 => (&mut limits.owners, "registered dependency owners"),
                2 => (&mut limits.names, "dependency name bytes"),
                3 => (&mut limits.name, "dependency qualified-name bytes"),
                _ => unreachable!(),
            };
            *counter -= 1;
            let errors = check(&linked, &limits);
            if used && fault == 3 {
                assert!(errors.iter().any(|error| {
                    error
                        .message
                        .contains("resolved dependency qualified-name bytes")
                }));
            }
            assert!(
                errors.iter().any(|error| error.message.contains(needle)),
                "{errors:?}"
            );
            limits = Limits {
                bindings: 3,
                owners: 2,
                names,
                name,
            };
        }
    }
    assert_eq!(
        (LIMITS.bindings, LIMITS.owners, LIMITS.names, LIMITS.name),
        (100_000, 1024, 64 * 1024 * 1024, 65_535)
    );
}
