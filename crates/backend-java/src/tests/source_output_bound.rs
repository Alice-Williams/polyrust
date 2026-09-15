use super::*;
use crate::{
    ast::*,
    dialect::{JavaDependencyScope, JavaStructuralRenderer},
    tests::source_dependency_fixture as f,
};
use portable_codegen::{OutputContents, render_certified_package};

fn within(api: &JavaDependencyApi) -> u64 {
    let reserved = api.source_byte_bound().unwrap();
    let rendered = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
    let actual: u64 = rendered
        .files()
        .iter()
        .map(|file| {
            let OutputContents::Text(text) = file.contents() else {
                panic!("Java source text")
            };
            text.len() as u64
        })
        .sum();
    assert!(reserved >= actual, "reserved {reserved}, actual {actual}");
    reserved
}

#[test]
fn wide_literals_have_a_real_source_reservation() {
    for value in [
        i64::MIN,
        -9_007_199_254_740_993,
        0,
        9_007_199_254_740_993,
        i64::MAX,
    ] {
        let long = JavaType::primitive(JavaPrimitive::Long);
        let mut functions = f::functions(0);
        functions[0].result = long.clone();
        functions[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
            long.clone(),
            JavaLiteral::I64(value),
        )))]);
        let api =
            JavaDependencyApi::from_certificate(f::certify(f::package(7, functions))).unwrap();
        assert_eq!(api.function(f::id(7, 10)).unwrap().signature().result, long);
        within(&api);
    }
}

#[test]
fn small_records_and_expanded_documentation_fit_before_rendering() {
    for value in [i32::MIN, -1, 0, i32::MAX] {
        within(
            &JavaDependencyApi::from_certificate(f::certify(f::package(7, f::functions(value))))
                .unwrap(),
        );
    }
    let ordinary =
        JavaDependencyApi::from_certificate(f::certify(f::record_package(|_| {}))).unwrap();
    let large = JavaDependencyApi::from_certificate(f::certify(f::record_package(|fixture| {
        for component in &mut fixture.record.record_components {
            let JavaRecordComponentOrigin::RustSource(field) = &mut component.origin else {
                panic!("source field")
            };
            std::sync::Arc::make_mut(&mut field.origin).documentation = vec![
                crate::tests::source_documentation_fixture::HOSTILE.repeat(200),
                "\n".repeat(300),
            ];
        }
    })))
    .unwrap();
    assert!(within(&large) > within(&ordinary));
}

#[test]
fn long_name_and_large_local_inventory() {
    // Declaration, parameter, local use and every repeated occurrence are
    // charged independently; the renderer's symbol table is not a one-off cost.
    for width in [1, 128, 1024] {
        let mut functions = f::functions(0);
        functions[1].name = f::name(&format!("function{}", "x".repeat(width)));
        let parameter = f::name(&format!("parameter{}", "x".repeat(width)));
        functions[1].parameters[0].name = parameter.clone();
        let mut statements = Vec::new();
        for index in 0..256 {
            statements.push(JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: f::int(),
                name: f::name(&format!("local{index}")),
                value: Some(JavaExpr::local(f::int(), parameter.clone())),
            });
        }
        statements.push(JavaStmt::Return(Some(JavaExpr::local(f::int(), parameter))));
        functions[1].body = JavaBlock::new(statements);
        within(&JavaDependencyApi::from_certificate(f::certify(f::package(7, functions))).unwrap());
    }
}

#[test]
fn deeply_nested_conditionals_and_indentation_are_reserved() {
    for depth in [1, 16, 64] {
        let mut body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
            f::int(),
            JavaLiteral::I32(-1),
        )))]);
        for _ in 0..depth {
            body = JavaBlock::new(vec![JavaStmt::If {
                condition: JavaExpr::literal(f::boolean(), JavaLiteral::Boolean(true)),
                then_block: body,
                else_block: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
                    JavaExpr::literal(f::int(), JavaLiteral::I32(1)),
                ))])),
            }]);
        }
        let mut functions = f::functions(0);
        functions[0].body = body;
        within(&JavaDependencyApi::from_certificate(f::certify(f::package(7, functions))).unwrap());
    }
}

#[test]
fn each_repeated_foreign_call_reserves_its_qualified_path() {
    let mut functions = f::functions(0);
    functions[0].name = f::name(&format!("foreign{}", "x".repeat(2048)));
    let owner = JavaDependencyApi::from_certificate(f::certify(f::package(7, functions))).unwrap();
    within(&owner);
    let mut previous = 0;
    for count in [1, 32, 256] {
        let (scope, callable) =
            JavaDependencyScope::new().import(owner.function(f::id(7, 10)).unwrap().clone());
        let mut functions = f::functions(0);
        let mut statements = Vec::new();
        for index in 0..count {
            statements.push(JavaStmt::Local {
                finality: JavaLocalFinality::Final,
                ty: f::int(),
                name: f::name(&format!("local{index}")),
                value: Some(JavaExpr {
                    ty: f::int(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Call {
                        callable: JavaCallableRef::Dependency(callable.clone()),
                        receiver: None,
                        arguments: vec![],
                    },
                }),
            });
        }
        statements.push(JavaStmt::Return(Some(JavaExpr::literal(
            f::int(),
            JavaLiteral::I32(0),
        ))));
        functions[0].body = JavaBlock::new(statements);
        let consumer = JavaDependencyApi::from_certificate(f::certify(
            f::package_with_dependencies(8, functions, scope.finish()),
        ))
        .unwrap();
        let reserved = within(&consumer);
        assert!(reserved > previous);
        previous = reserved;
    }
}

#[test]
fn unmeasured_shapes_do_not_receive_a_small_fallback() {
    let names = BTreeMap::new();
    let mut reader = Reader {
        budget: Budget::new(),
        names: &names,
    };
    assert!(
        reader
            .ty(&JavaType::primitive(JavaPrimitive::Double))
            .is_err()
    );
    assert!(
        reader
            .block(&JavaBlock::new(vec![JavaStmt::Break]), 0)
            .is_err()
    );
}
