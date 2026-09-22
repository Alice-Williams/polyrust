use super::source_dependency_fixture::*;
use crate::{ast::*, dialect::JavaDependencyApi};

#[test]
fn mixed_width_comparisons_reject_before_java_dependency_certification() {
    for operator in [
        JavaBinaryOperator::Equal,
        JavaBinaryOperator::NotEqual,
        JavaBinaryOperator::Less,
        JavaBinaryOperator::LessEqual,
        JavaBinaryOperator::Greater,
        JavaBinaryOperator::GreaterEqual,
    ] {
        for mixed in [false, true] {
            for reverse in [false, true] {
                let wide = JavaExpr::literal(
                    JavaType::primitive(JavaPrimitive::Long),
                    JavaLiteral::I64(1),
                );
                let other = if mixed {
                    JavaExpr::literal(int(), JavaLiteral::I32(1))
                } else {
                    wide.clone()
                };
                let (left, right) = if reverse {
                    (other, wide)
                } else {
                    (wide, other)
                };
                let mut fixture = functions(42);
                fixture[0].result = boolean();
                fixture[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                    ty: boolean(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Binary {
                        operator,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                }))]);
                let draft = package(7, fixture);
                if mixed {
                    // Java syntax permits promotion, but our existing AST
                    // contract requires exact operand types before certification.
                    let errors = portable_codegen::verify_unresolved_package(
                        &crate::dialect::JavaDialect,
                        draft,
                    )
                    .unwrap_err();
                    assert!(
                        errors
                            .iter()
                            .any(|error| error.message.contains("binary operator type mismatch")),
                        "{errors:?}"
                    );
                } else {
                    assert!(JavaDependencyApi::from_certificate(certify(draft)).is_ok());
                }
            }
        }
    }
}

#[test]
fn long_signatures_remain_distinct_and_long_mutation_is_not_admitted() {
    let long = JavaType::primitive(JavaPrimitive::Long);
    let mut fixture = functions(42);
    fixture[1].parameters[0].ty = long.clone();
    fixture[1].result = long.clone();
    fixture[1].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::local(
        long.clone(),
        name("p0"),
    )))]);
    let api = JavaDependencyApi::from_certificate(certify(package(7, fixture))).unwrap();
    let signature = api.function(id(7, 11)).unwrap().signature();
    assert_eq!(
        signature.parameters,
        [long.clone(), boolean(), int(), boolean()]
    );
    assert_eq!(signature.result, long);
    let mut fixture = functions(42);
    fixture[0].body.statements.insert(
        0,
        JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty: long.clone(),
            name: name("wide"),
            value: Some(JavaExpr::literal(long, JavaLiteral::I64(i64::MIN))),
        },
    );
    assert!(
        JavaDependencyApi::from_certificate(certify(package(7, fixture)))
            .unwrap_err()
            .contains("unadmitted statement")
    );
}

#[test]
fn mutable_bool_local_branch_writes_are_closed_dependency_bodies() {
    let mut fixture = functions(42);
    let local = JavaExpr::local(boolean(), name("result"));
    fixture[2].body = JavaBlock::new(vec![
        JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty: boolean(),
            name: name("result"),
            value: Some(JavaExpr::literal(boolean(), JavaLiteral::Boolean(false))),
        },
        JavaStmt::If {
            condition: JavaExpr::local(boolean(), name("p0")),
            then_block: JavaBlock::new(vec![JavaStmt::Assign {
                target: local.clone(),
                value: JavaExpr::literal(boolean(), JavaLiteral::Boolean(true)),
            }]),
            else_block: Some(JavaBlock::new(vec![])),
        },
        JavaStmt::Return(Some(local)),
    ]);
    assert!(JavaDependencyApi::from_certificate(certify(package(7, fixture))).is_ok());
}

#[test]
fn mutable_bool_parameters_remain_outside_dependency_signature_admission() {
    let mut fixture = functions(42);
    fixture[0].body.statements.insert(
        0,
        JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty: boolean(),
            name: name("p0"),
            value: Some(JavaExpr::literal(boolean(), JavaLiteral::Boolean(false))),
        },
    );
    fixture[2].parameters[0].final_parameter = false;
    fixture[2].body.statements.insert(
        0,
        JavaStmt::Assign {
            target: JavaExpr::local(boolean(), name("p0")),
            value: JavaExpr::literal(boolean(), JavaLiteral::Boolean(true)),
        },
    );
    assert!(
        JavaDependencyApi::from_certificate(certify(package(7, fixture)))
            .unwrap_err()
            .contains("visibility/signature/declaration inventory disagrees")
    );
}

#[test]
fn mutable_integer_locals_remain_outside_dependency_body_admission() {
    let mut fixture = functions(42);
    fixture[0].body.statements.insert(
        0,
        JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty: int(),
            name: name("result"),
            value: Some(JavaExpr::literal(int(), JavaLiteral::I32(1))),
        },
    );
    assert!(
        JavaDependencyApi::from_certificate(certify(package(7, fixture)))
            .unwrap_err()
            .contains("unadmitted statement")
    );
}

#[test]
fn boolean_negation_is_a_closed_dependency_expression() {
    let mut fixture = functions(42);
    fixture[2].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
        ty: boolean(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator: JavaUnaryOperator::Not,
            operand: Box::new(JavaExpr::local(boolean(), name("p0"))),
        },
    }))]);
    let api = JavaDependencyApi::from_certificate(certify(package(7, fixture))).unwrap();
    assert_eq!(
        api.function(id(7, 12)).unwrap().signature().result,
        boolean()
    );
}

#[test]
fn integer_addition_is_admitted_but_subtraction_remains_outside() {
    for operator in [JavaBinaryOperator::Add, JavaBinaryOperator::Subtract] {
        let mut fixture = functions(42);
        fixture[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: int(),
            precedence: JavaPrecedence::Additive,
            kind: JavaExprKind::Binary {
                operator,
                left: Box::new(JavaExpr::literal(int(), JavaLiteral::I32(42))),
                right: Box::new(JavaExpr::literal(int(), JavaLiteral::I32(1))),
            },
        }))]);
        let result = JavaDependencyApi::from_certificate(certify(package(7, fixture)));
        if operator == JavaBinaryOperator::Add {
            assert_eq!(
                result
                    .unwrap()
                    .function(id(7, 10))
                    .unwrap()
                    .signature()
                    .result,
                int()
            );
        } else {
            assert!(result.unwrap_err().contains("unadmitted expression"));
        }
    }
}

#[test]
fn integer_complement_has_full_dependency_certificates_for_both_widths() {
    for (ty, literal) in [
        (int(), JavaLiteral::I32(i32::MIN)),
        (
            JavaType::primitive(JavaPrimitive::Long),
            JavaLiteral::I64(i64::MIN),
        ),
    ] {
        let mut fixture = functions(42);
        fixture[0].result = ty.clone();
        fixture[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: ty.clone(),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Unary {
                operator: JavaUnaryOperator::BitNot,
                operand: Box::new(JavaExpr::literal(ty.clone(), literal)),
            },
        }))]);
        let api = JavaDependencyApi::from_certificate(certify(package(7, fixture))).unwrap();
        assert_eq!(api.function(id(7, 10)).unwrap().signature().result, ty);
    }
}

#[test]
fn private_record_is_closed_without_exposing_its_constructor_or_fields() {
    let api = JavaDependencyApi::from_certificate(certify(record_package(|_| {}))).unwrap();
    assert_eq!(api.functions().len(), 1);
    assert!(api.function(id(7, 5)).is_some());
    for hash in [2, 3, 4] {
        assert!(api.function(id(7, hash)).is_none());
    }
}

#[test]
fn valid_java_custom_record_initialization_is_not_a_closed_source_mapping() {
    let draft = record_package(|fixture| {
        let JavaStmt::Assign { value, .. } = &mut fixture.constructor().body.statements[0] else {
            panic!("assignment")
        };
        *value = JavaExpr::literal(int(), JavaLiteral::I32(17));
    });
    assert!(
        JavaDependencyApi::from_certificate(certify(draft))
            .unwrap_err()
            .contains("exact field initialization")
    );
}

#[test]
fn dependency_api_exposes_exact_public_scalars_aliases_and_docs_only() {
    let api = JavaDependencyApi::from_certificate(certify(package(7, functions(42)))).unwrap();
    assert_eq!(api.root(), id(7, 1));
    assert_eq!(api.functions().len(), 3);
    assert!(api.function(id(7, 13)).is_none());
    assert!(api.function(id(8, 10)).is_none());
    for (hash, arity, result) in [(10, 0, int()), (11, 4, int()), (12, 1, boolean())] {
        let function = api.function(id(7, hash)).unwrap();
        assert_eq!(function.signature().parameters.len(), arity);
        assert_eq!(function.signature().result, result);
        assert_eq!(function.package_identity(), api.package_identity());
        assert_eq!(function.path().package(), JavaPackage::RustCrate(7));
        assert_eq!(function.path().owners(), [name("Generated")]);
        assert_eq!(function.path().member(), &name(&format!("fn{hash:016x}")));
        assert_eq!(
            function.source().documentation,
            [format!(" Function {hash} documentation.")]
        );
        assert_eq!(
            function.source().crate_exports.modules[&id(7, 1)]
                .values()
                .filter(|target| **target
                    == portable_codegen::RustExportTarget::Declaration(id(7, hash)))
                .count(),
            2
        );
    }
}

#[test]
fn dependency_proof_identity_distinguishes_certificates_not_just_source_ids() {
    let first = JavaDependencyApi::from_certificate(certify(package(7, functions(42)))).unwrap();
    let cloned = first.clone();
    let second = JavaDependencyApi::from_certificate(certify(package(7, functions(-7)))).unwrap();
    let repeated = JavaDependencyApi::from_certificate(first.package().clone()).unwrap();
    assert_eq!(first.package_identity(), cloned.package_identity());
    assert_eq!(first.function(id(7, 10)), cloned.function(id(7, 10)));
    assert_ne!(first.package_identity(), second.package_identity());
    assert_ne!(first.function(id(7, 10)), second.function(id(7, 10)));
    assert_ne!(first.function(id(7, 10)), repeated.function(id(7, 10)));
    assert_ne!(first.function(id(7, 10)), first.function(id(7, 11)));
}

#[test]
fn valid_java_with_mutation_is_not_a_closed_dependency_body() {
    let mut fixture = functions(42);
    fixture[0].body.statements.insert(
        0,
        JavaStmt::Local {
            finality: JavaLocalFinality::Mutable,
            ty: int(),
            name: name("effect"),
            value: Some(JavaExpr::literal(int(), JavaLiteral::I32(0))),
        },
    );
    fixture[0].body.statements.insert(
        1,
        JavaStmt::Assign {
            target: JavaExpr::local(int(), name("effect")),
            value: JavaExpr::literal(int(), JavaLiteral::I32(1)),
        },
    );
    let certificate = certify(package(7, fixture));
    assert!(
        JavaDependencyApi::from_certificate(certificate)
            .unwrap_err()
            .contains("unadmitted statement")
    );
}

#[test]
fn valid_java_arithmetic_is_not_silently_admitted_as_a_source_capability() {
    let mut fixture = functions(42);
    fixture[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
        ty: int(),
        precedence: JavaPrecedence::Multiplicative,
        kind: JavaExprKind::Binary {
            operator: JavaBinaryOperator::Divide,
            left: Box::new(JavaExpr::literal(int(), JavaLiteral::I32(42))),
            right: Box::new(JavaExpr::literal(int(), JavaLiteral::I32(0))),
        },
    }))]);
    let certificate = certify(package(7, fixture));
    assert!(
        JavaDependencyApi::from_certificate(certificate)
            .unwrap_err()
            .contains("unadmitted expression")
    );
}

#[test]
fn recursive_valid_java_cannot_borrow_a_pure_signature_as_closed_proof() {
    let draft = package_with(7, functions(42), |_, _, declaration| {
        let JavaMember::Method(method) = &mut declaration.members[1] else {
            panic!("method")
        };
        let JavaMethodDeclaration::Callable(symbol) = method.declared else {
            panic!("callable")
        };
        method.body = Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: int(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Call {
                callable: JavaCallableRef::Generated {
                    symbol,
                    signature: JavaMethodSignature {
                        receiver: None,
                        parameters: vec![],
                        result: int(),
                        checked_exceptions: vec![],
                        nullable_result: false,
                        pure: true,
                    },
                },
                receiver: None,
                arguments: vec![],
            },
        }))]));
    });
    assert!(
        JavaDependencyApi::from_certificate(certify(draft))
            .unwrap_err()
            .contains("recursive")
    );
}

#[test]
fn public_facade_constructor_is_valid_java_but_not_an_admitted_dependency() {
    let draft = package_with(7, functions(42), |_, _, declaration| {
        let JavaMember::Constructor(constructor) = &mut declaration.members[0] else {
            panic!("constructor")
        };
        constructor.modifiers = vec![JavaModifier::Public];
    });
    assert!(
        JavaDependencyApi::from_certificate(certify(draft))
            .unwrap_err()
            .contains("unadmitted member")
    );
}
