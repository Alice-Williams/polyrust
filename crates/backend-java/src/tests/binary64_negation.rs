//! Primitive negate retains exact Double types and original imported authority.
use super::*;

pub(super) fn negate(operand: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: double(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator: JavaUnaryOperator::Negate,
            operand: Box::new(operand),
        },
    }
}
pub(super) fn owner(crate_id: u64, dependencies: Option<&JavaDependencyApi>) -> JavaDependencyApi {
    let mut scope = JavaDependencyScope::new();
    let imports: Vec<_> = dependencies
        .into_iter()
        .flat_map(|owner| owner.functions())
        .collect();
    let mut declarations = Vec::new();
    for index in 0..2 {
        let input = JavaExpr::local(double(), f::name("input"));
        let operand = if let Some(function) = imports.get(index) {
            let (next, callable) = scope.import((*function).clone()).unwrap();
            scope = next;
            JavaExpr {
                ty: double(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Call {
                    callable: JavaCallableRef::Dependency(callable),
                    receiver: None,
                    arguments: vec![input],
                },
            }
        } else {
            input
        };
        let mut value = negate(operand);
        if imports.is_empty() && index == 1 {
            value = negate(value);
        }
        declarations.push(f::Function {
            hash: 10 + index as u64,
            public: true,
            name: f::name(&format!("negate{index}")),
            parameters: vec![JavaParameter {
                ty: double(),
                name: f::name("input"),
                final_parameter: true,
            }],
            result: double(),
            body: JavaBlock::new(vec![JavaStmt::Return(Some(value))]),
        });
    }
    JavaDependencyApi::from_certificate(f::certify(f::package_with_dependencies(
        crate_id,
        declarations,
        scope.finish(),
    )))
    .unwrap()
}

#[test]
fn primitive_and_imported_double_negation_have_exact_certificates() {
    let first = owner(96, None);
    let second = owner(97, Some(&first));
    for api in [&first, &second] {
        assert_eq!(api.functions().count(), 2);
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(text.len() as u64 <= api.source_byte_bound().unwrap());
            assert!(
                !text.contains("Runtime")
                    && !text.contains("longBitsToDouble")
                    && !text.contains("0.0 -")
            );
        }
    }
}
#[test]
fn counterfeit_double_unary_types_operators_and_precedence_reject() {
    for fault in 0..5 {
        let mut declarations = functions(&[]);
        let mut value = negate(JavaExpr::local(double(), f::name("input")));
        if fault == 0 {
            value.ty = JavaType::primitive(JavaPrimitive::Long);
        }
        if fault == 1 {
            value.precedence = JavaPrecedence::Primary;
        }
        let JavaExprKind::Unary { operator, operand } = &mut value.kind else {
            unreachable!()
        };
        if fault == 2 {
            *operator = JavaUnaryOperator::BitNot;
        }
        if fault == 3 {
            *operator = JavaUnaryOperator::Not;
        }
        if fault == 4 {
            operand.ty = JavaType::primitive(JavaPrimitive::Long);
        }
        declarations[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(value))]);
        let unresolved = f::package(96, declarations);
        match verify_unresolved_package(&JavaDialect, unresolved) {
            Err(_) => {}
            Ok(package) => {
                if let Ok(linked) = TargetLinker::new(JavaDialect).link_ast(&package)
                    && let Ok(certificate) = certify_resolved_package(&JavaDialect, linked)
                {
                    assert!(
                        JavaDependencyApi::from_certificate(certificate).is_err(),
                        "invalid unary metadata must not acquire callable authority"
                    );
                }
            }
        }
    }
}

#[path = "binary64_negation_native.rs"]
mod native;
