//! Primitive Java wrapping negation preserves width and original callable evidence.
use crate::tests::source_dependency_fixture as f;
use crate::{ast::*, dialect::*};
use portable_codegen::*;

pub(super) fn negate(value: JavaExpr) -> JavaExpr {
    JavaExpr {
        ty: value.ty.clone(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator: JavaUnaryOperator::Negate,
            operand: Box::new(value),
        },
    }
}
pub(super) fn functions(dependencies: Option<&[JavaImportedCallable]>) -> Vec<f::Function> {
    [JavaPrimitive::Int, JavaPrimitive::Long]
        .into_iter()
        .enumerate()
        .map(|(index, primitive)| {
            let ty = JavaType::primitive(primitive);
            let input = JavaExpr::local(ty.clone(), f::name("input"));
            let operand = dependencies.map_or_else(
                || input.clone(),
                |calls| JavaExpr {
                    ty: ty.clone(),
                    precedence: JavaPrecedence::Primary,
                    kind: JavaExprKind::Call {
                        callable: JavaCallableRef::Dependency(calls[index].clone()),
                        receiver: None,
                        arguments: vec![input.clone()],
                    },
                },
            );
            f::Function {
                hash: 10 + index as u64,
                public: true,
                name: f::name(if index == 0 { "negate32" } else { "negate64" }),
                parameters: vec![JavaParameter {
                    ty: ty.clone(),
                    name: f::name("input"),
                    final_parameter: true,
                }],
                result: ty,
                body: JavaBlock::new(vec![JavaStmt::Return(Some(negate(operand)))]),
            }
        })
        .collect()
}
pub(super) fn chain() -> Vec<JavaDependencyApi> {
    chain_with_operator(JavaUnaryOperator::Negate)
}
pub(super) fn chain_with_operator(operator: JavaUnaryOperator) -> Vec<JavaDependencyApi> {
    let mut declarations = functions(None);
    for function in &mut declarations {
        let JavaStmt::Return(Some(value)) = &mut function.body.statements[0] else {
            unreachable!()
        };
        let JavaExprKind::Unary {
            operator: actual, ..
        } = &mut value.kind
        else {
            unreachable!()
        };
        *actual = operator;
    }
    let first =
        JavaDependencyApi::from_certificate(f::certify(f::package(81, declarations))).unwrap();
    let mut scope = JavaDependencyScope::new();
    let mut calls = vec![];
    for function in first.functions() {
        let (next, callable) = scope.import(function.clone());
        scope = next;
        calls.push(callable);
    }
    let second = JavaDependencyApi::from_certificate(f::certify(f::package_with_dependencies(
        82,
        functions(Some(&calls)),
        scope.finish(),
    )))
    .unwrap();
    vec![first, second]
}

#[test]
fn wrapping_negation_certifies_nested_calls_with_source_and_height_accounting() {
    for (index, api) in chain().iter().enumerate() {
        assert!(
            api.functions()
                .all(|function| function.call_height() == index + 1)
        );
        let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
        assert_eq!(output.files().len(), 1);
        let OutputContents::Text(text) = output.files()[0].contents() else {
            panic!("source")
        };
        assert!(text.len() as u64 <= api.source_byte_bound().unwrap());
        assert!(!text.contains("Runtime") && !text.contains("Math.negateExact"));
        assert!(text.contains("int negate32") && text.contains("long negate64"));
    }
}

#[test]
fn wrapping_negation_wrong_result_width_is_not_render_ready() {
    let mut functions = functions(None);
    functions[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
        ty: JavaType::primitive(JavaPrimitive::Long),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator: JavaUnaryOperator::Negate,
            operand: Box::new(JavaExpr::local(f::int(), f::name("input"))),
        },
    }))]);
    let package = f::package(81, functions);
    if let Ok(checked) = verify_unresolved_package(&JavaDialect, package)
        && let Ok(linked) = TargetLinker::new(JavaDialect).link_ast(&checked)
    {
        assert!(certify_resolved_package(&JavaDialect, linked).is_err());
    }
}

#[test]
fn wrapping_negation_wrong_precedence_cannot_acquire_dependency_authority() {
    let mut functions = functions(None);
    let JavaStmt::Return(Some(value)) = &mut functions[0].body.statements[0] else {
        panic!("negate return")
    };
    value.precedence = JavaPrecedence::Primary;
    let package = f::package(81, functions);
    let checked = verify_unresolved_package(&JavaDialect, package).unwrap();
    let linked = TargetLinker::new(JavaDialect).link_ast(&checked).unwrap();
    match certify_resolved_package(&JavaDialect, linked) {
        Err(_) => {}
        Ok(certificate) => assert!(JavaDependencyApi::from_certificate(certificate).is_err()),
    }
}

#[path = "wrapping_negation_native.rs"]
mod native;
