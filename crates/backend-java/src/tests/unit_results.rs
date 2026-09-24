//! Ordinary void dependency results preserve value and authority boundaries.
use crate::tests::source_dependency_fixture as f;
use crate::{ast::*, dialect::*};
use portable_codegen::*;

fn void() -> JavaType {
    JavaType::primitive(JavaPrimitive::Void)
}

fn functions(scalar: bool, call: Option<JavaExpr>) -> Vec<f::Function> {
    let mut body = vec![];
    if let Some(call) = call {
        body.push(JavaStmt::Expression(call));
    }
    body.push(JavaStmt::Return(
        scalar.then(|| JavaExpr::local(f::int(), f::name("input"))),
    ));
    vec![f::Function {
        hash: 10,
        public: true,
        name: f::name("operation"),
        parameters: vec![JavaParameter {
            ty: f::int(),
            name: f::name("input"),
            final_parameter: true,
        }],
        result: if scalar { f::int() } else { void() },
        body: JavaBlock::new(body),
    }]
}
fn call(callable: &JavaImportedCallable) -> JavaExpr {
    JavaExpr {
        ty: void(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Dependency(callable.clone()),
            receiver: None,
            arguments: vec![JavaExpr::local(f::int(), f::name("input"))],
        },
    }
}
fn chain() -> Vec<JavaDependencyApi> {
    let mut owners = vec![];
    for (id, scalar) in [(71, false), (72, false), (73, true)] {
        let mut bindings = JavaDependencyBindings::default();
        let effect = owners.last().map(|owner: &JavaDependencyApi| {
            let (scope, callable) = JavaDependencyScope::new()
                .import(owner.functions().next().unwrap().clone())
                .unwrap();
            bindings = scope.finish();
            call(&callable)
        });
        let package = f::certify(f::package_with_dependencies(
            id,
            functions(scalar, effect),
            bindings,
        ));
        owners.push(JavaDependencyApi::from_certificate(package).unwrap());
    }
    owners
}
fn rejected(package: TargetAstPackage<JavaDialect>) {
    if let Ok(checked) = verify_unresolved_package(&JavaDialect, package)
        && let Ok(linked) = TargetLinker::new(JavaDialect).link_ast(&checked)
    {
        assert!(certify_resolved_package(&JavaDialect, linked).is_err());
    }
}

#[test]
fn unit_results_preserve_call_heights_and_real_void_types() {
    let owners = chain();
    for (index, owner) in owners.iter().enumerate() {
        let function = owner.functions().next().unwrap();
        assert_eq!(function.declaration_signature().result == void(), index < 2);
        assert_eq!(function.call_height(), index + 1);
        let output = render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
        assert_eq!(output.files().len(), 1);
        let byte_bound = owner.source_byte_bound().unwrap();
        let OutputContents::Text(text) = output.files()[0].contents() else {
            panic!("Java source");
        };
        assert!(
            !text.contains("Runtime") && !text.contains("java.lang.Void") && !text.contains("null")
        );
        assert!(text.len() as u64 <= byte_bound);
        if index > 0 {
            let previous = 70 + index;
            let expected =
                format!("org.polyrust.generated.r{previous:016x}.Generated.operation(input);");
            assert_eq!(
                text.matches(&expected).count(),
                1,
                "void call must render exactly once: {text}"
            );
        }
        if index < 2 {
            assert!(text.contains("void operation"));
        }
    }
}

#[test]
fn unit_results_conditionals_without_else_retain_calls_and_source_bound() {
    let owner = chain().remove(0);
    let (scope, callable) = JavaDependencyScope::new()
        .import(owner.functions().next().unwrap().clone())
        .unwrap();
    let mut methods = functions(false, None);
    methods[0].body.statements.insert(
        0,
        JavaStmt::If {
            condition: JavaExpr::literal(f::boolean(), JavaLiteral::Boolean(true)),
            then_block: JavaBlock::new(vec![JavaStmt::Expression(call(&callable))]),
            else_block: None,
        },
    );
    let mut unbound = functions(false, None);
    unbound[0].body = methods[0].body.clone();
    rejected(f::package(76, unbound));
    let certificate = f::certify(f::package_with_dependencies(76, methods, scope.finish()));
    let api = JavaDependencyApi::from_certificate(certificate).unwrap();
    assert_eq!(api.functions().next().unwrap().call_height(), 2);
    let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("Java source")
    };
    assert!(text.len() as u64 <= api.source_byte_bound().unwrap());
    assert_eq!(text.matches("Generated.operation(input);").count(), 1);
}

#[test]
fn unit_results_reject_value_returns_void_values_and_invalid_storage() {
    for scalar in [false, true] {
        let mut methods = functions(scalar, None);
        methods[0].body = JavaBlock::new(vec![JavaStmt::Return(
            (!scalar).then(|| JavaExpr::literal(f::int(), JavaLiteral::I32(1))),
        )]);
        rejected(f::package(71, methods));
    }
    for position in 0..3 {
        let mut methods = functions(false, None);
        match position {
            0 => methods[0].parameters[0].ty = void(),
            1 => methods[0].body.statements.insert(
                0,
                JavaStmt::Local {
                    finality: JavaLocalFinality::Final,
                    ty: void(),
                    name: f::name("illegal"),
                    value: None,
                },
            ),
            _ => {
                methods[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::local(
                    void(),
                    f::name("input"),
                )))])
            }
        }
        rejected(f::package(71, methods));
    }
}

#[test]
fn unit_results_require_original_scope_and_exact_call_arguments() {
    let owner = chain().remove(0);
    let function = owner.functions().next().unwrap().clone();
    let (scope, callable) = JavaDependencyScope::new().import(function.clone()).unwrap();
    let bindings = scope.finish();
    rejected(f::package(72, functions(false, Some(call(&callable)))));
    let (other, _) = JavaDependencyScope::new().import(function).unwrap();
    rejected(f::package_with_dependencies(
        72,
        functions(false, Some(call(&callable))),
        other.finish(),
    ));
    for mutation in 0..4 {
        let mut value = call(&callable);
        let JavaExprKind::Call { arguments, .. } = &mut value.kind else {
            unreachable!()
        };
        match mutation {
            0 => arguments.clear(),
            1 => arguments[0] = JavaExpr::literal(f::boolean(), JavaLiteral::Boolean(true)),
            2 => value.ty = f::int(),
            _ => {}
        }
        let mut methods = functions(false, Some(value.clone()));
        if mutation == 3 {
            methods[0].result = f::int();
            methods[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(value))]);
        }
        rejected(f::package_with_dependencies(72, methods, bindings.clone()));
    }
}

#[path = "unit_results_native.rs"]
mod native;

#[test]
fn unit_results_catalogue_rejects_changed_results_and_recertified_owners() {
    let original = chain().remove(0);
    let replacement =
        JavaDependencyApi::from_certificate(f::certify(f::package(71, functions(false, None))))
            .unwrap();
    assert_ne!(original.package_identity(), replacement.package_identity());
    let (scope, callable) = JavaDependencyScope::new()
        .import(original.functions().next().unwrap().clone())
        .unwrap();
    let package =
        f::package_with_dependencies(74, functions(false, Some(call(&callable))), scope.finish());
    let catalogue = JavaDialect.package_symbol_catalogue(&package).unwrap();
    for mutation in 0..3 {
        let mut changed = catalogue.clone();
        let spec = &mut changed.dependency_callables[0];
        match mutation {
            0 => spec.signature.return_type = TargetTypeRef::Primitive(JavaPrimitive::Int),
            1 => spec.owner = replacement.package_identity().clone(),
            _ => spec.signature.parameters.clear(),
        }
        assert!(changed.verify(&JavaDialect).is_err(), "mutation {mutation}");
    }
}

#[test]
fn unit_results_local_calls_have_height_and_recursive_bodies_reject() {
    for recursive in [false, true] {
        let mut methods = functions(false, None);
        let mut second = functions(false, None).remove(0);
        second.hash = 11;
        second.name = f::name("second");
        methods.push(second);
        let package = f::package_with(75, methods, |_, _, facade| {
            let ids = facade
                .members
                .iter()
                .filter_map(|member| {
                    if let JavaMember::Method(method) = member {
                        let JavaMethodDeclaration::Callable(id) = method.declared else {
                            unreachable!()
                        };
                        Some(id)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let mut index = 0;
            for member in &mut facade.members {
                let JavaMember::Method(method) = member else {
                    continue;
                };
                if index == 0 || recursive {
                    method.body.as_mut().unwrap().statements.insert(
                        0,
                        JavaStmt::Expression(JavaExpr {
                            ty: void(),
                            precedence: JavaPrecedence::Primary,
                            kind: JavaExprKind::Call {
                                callable: JavaCallableRef::Generated {
                                    symbol: ids[1 - index],
                                    signature: JavaMethodSignature {
                                        receiver: None,
                                        parameters: vec![f::int()],
                                        result: void(),
                                        checked_exceptions: vec![],
                                        nullable_result: false,
                                        pure: true,
                                    },
                                },
                                receiver: None,
                                arguments: vec![JavaExpr::local(f::int(), f::name("input"))],
                            },
                        }),
                    );
                }
                index += 1;
            }
        });
        let certificate = f::certify(package);
        let api = JavaDependencyApi::from_certificate(certificate);
        if recursive {
            assert!(api.unwrap_err().contains("recursive"));
        } else {
            assert_eq!(
                api.unwrap().function(f::id(75, 10)).unwrap().call_height(),
                2
            );
        }
    }
}
