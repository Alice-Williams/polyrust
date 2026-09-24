//! Real imported Result signatures and call chains exercise target limits.
use super::{fixture, methods};
use crate::{ast::*, dialect::*, tests::source_dependency_fixture as source};
use portable_codegen::*;

fn call(callable: JavaCallableRef, result: JavaType) -> JavaExpr {
    JavaExpr {
        ty: result,
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable,
            receiver: None,
            arguments: vec![],
        },
    }
}

#[test]
fn result_reference_parameters_charge_exact_jvm_slots() {
    let api = fixture::owner();
    let family = api.result_families().next().unwrap();
    for count in [255, 256] {
        let (scope, outcome) = JavaDependencyScope::new()
            .import_result_type(family.ty(JavaResultTypeRole::Interface))
            .unwrap();
        let parameters = std::iter::once(outcome.ty())
            .chain(std::iter::repeat_n(source::int(), count - 1))
            .collect();
        let draft = fixture::consumer(
            scope.finish(),
            source::int(),
            parameters,
            JavaExpr::literal(source::int(), JavaLiteral::I32(0)),
        );
        let checked = verify_unresolved_package(&JavaDialect, draft).unwrap();
        let linked = TargetLinker::new(JavaDialect).link_ast(&checked).unwrap();
        let result = certify_resolved_package(&JavaDialect, linked);
        if count == 255 {
            let api = JavaDependencyApi::from_certificate(result.unwrap()).unwrap();
            assert!(fixture::text(api.package()).len() as u64 <= api.source_byte_bound().unwrap());
        } else {
            let Err(errors) = result else {
                panic!("256 slots certified")
            };
            assert!(
                errors.iter().any(|error| error.code
                    == portable_diagnostics::DiagnosticCode::TargetResourceLimit
                    && error.message.contains("parameter slots")),
                "{errors:?}"
            );
        }
    }
}

#[test]
fn imported_result_call_chain_accepts_height_128_and_rejects_129() {
    let family = fixture::owner();
    let error = family
        .result_families()
        .next()
        .unwrap()
        .ty(JavaResultTypeRole::Error);
    let (scope, constructor) = JavaDependencyScope::new()
        .import_result_constructor(error.constructor().unwrap())
        .unwrap();
    let producer = JavaDependencyApi::from_certificate(source::certify(fixture::consumer_at(
        8,
        scope.finish(),
        constructor.owner().ty(),
        vec![],
        methods::new(&constructor, vec![]),
    )))
    .unwrap();
    let original = producer.functions().next().unwrap();
    assert_eq!(original.call_height(), 1);
    for count in [127, 128] {
        let (scope, imported) = JavaDependencyScope::new().import(original.clone()).unwrap();
        let signature = imported.signature().clone();
        let functions = (0..count)
            .map(|index| source::Function {
                hash: 10 + index as u64,
                public: true,
                name: source::name(&format!("chain{index}")),
                parameters: vec![],
                result: signature.result.clone(),
                body: JavaBlock::new(vec![JavaStmt::Return(Some(call(
                    JavaCallableRef::Dependency(imported.clone()),
                    signature.result.clone(),
                )))]),
            })
            .collect();
        let draft = source::package_configured(
            9,
            functions,
            |_| {},
            |_, declared, facade| {
                let ids = declared
                    .iter()
                    .filter_map(|symbol| match symbol {
                        GeneratedSymbolId::Callable(id) => Some(*id),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                for (index, method) in facade
                    .members
                    .iter_mut()
                    .filter_map(|member| match member {
                        JavaMember::Method(method) => Some(method),
                        _ => None,
                    })
                    .enumerate()
                {
                    if index > 0 {
                        method.body = Some(JavaBlock::new(vec![JavaStmt::Return(Some(call(
                            JavaCallableRef::Generated {
                                symbol: ids[index - 1],
                                signature: signature.clone(),
                            },
                            signature.result.clone(),
                        )))]));
                    }
                }
            },
            scope.finish(),
        );
        let result = JavaDependencyApi::from_certificate(source::certify(draft));
        if count == 127 {
            let api = result.unwrap();
            assert_eq!(api.function(source::id(9, 136)).unwrap().call_height(), 128);
            assert!(api.source_byte_bound().unwrap() >= fixture::text(api.package()).len() as u64);
        } else {
            assert!(result.unwrap_err().contains("call height"));
        }
    }
}
