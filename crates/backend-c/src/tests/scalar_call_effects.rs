//! A call signature is not purity, storage validity or a rendering certificate.
use super::{ScalarCalls, fixture::Fixture};
use crate::ast::*;
use crate::ownership::{CSafetyError, context_facts::ContextFacts};

#[test]
fn builtin_bool_not_retains_body_derived_scalar_call_evidence() {
    let f = Fixture::new(&[0, 0]);
    let operand = f.values().literal(CLiteral::Bool(true)).unwrap();
    let value = f
        .values()
        .unary(CUnaryOperator::LogicalNot, operand)
        .unwrap();
    let value = f
        .values()
        .numeric_conversion(CScalarType::I32, value)
        .unwrap();
    let source = f.source(vec![
        f.returning(0, f.call(1, vec![])),
        f.returning(1, value),
    ]);
    assert_eq!(accepted(&f, &source), vec![true, true]);
    f.registry.check_storage_paths(&[source]).unwrap();
}

#[test]
fn logical_not_requires_bool_but_safe_negation_retains_purity_evidence() {
    let f = Fixture::new(&[0]);
    assert!(
        f.values()
            .unary(CUnaryOperator::LogicalNot, f.literal(1))
            .is_err()
    );
    let operator = CUnaryOperator::Negate;
    let f = Fixture::new(&[0, 0]);
    let value = f.values().unary(operator, f.literal(1)).unwrap();
    let value = f
        .values()
        .numeric_conversion(CScalarType::I32, value)
        .unwrap();
    let source = f.source(vec![
        f.returning(0, f.call(1, vec![])),
        f.returning(1, value),
    ]);
    assert_eq!(accepted(&f, &source), vec![true, true]);
    f.registry.check_numeric_flow(&[source]).unwrap();
}

#[test]
fn signed_bitwise_complement_retains_nested_transitive_call_evidence() {
    for scalar in [CScalarType::I32, CScalarType::I64] {
        let f = Fixture::with_result(&[vec![], vec![], vec![]], CObjectType::scalar(scalar));
        let literal = if scalar == CScalarType::I32 {
            CLiteral::Signed(CSignedLiteral::I32(1))
        } else {
            CLiteral::Signed(CSignedLiteral::I64(1))
        };
        let mut value = f.values().literal(literal).unwrap();
        for _ in 0..2 {
            value = f.values().unary(CUnaryOperator::BitNot, value).unwrap();
            if scalar == CScalarType::I32 {
                value = f
                    .values()
                    .numeric_conversion(CScalarType::I32, value)
                    .unwrap();
            }
        }
        let source = f.source(vec![
            f.returning(0, f.call(1, vec![])),
            f.returning(1, f.call(2, vec![])),
            f.returning(2, value),
        ]);
        assert_eq!(accepted(&f, &source), vec![true; 3], "{scalar:?}");
        f.registry.check_storage_paths(&[source]).unwrap();
    }
}

fn accepted(f: &Fixture, source: &CSourceFile) -> Vec<bool> {
    let facts = ContextFacts::check(&f.registry, std::slice::from_ref(source)).unwrap();
    f.functions
        .iter()
        .map(|function| facts.scalar_call(&f.values().direct(function.clone()).unwrap()))
        .collect()
}

#[test]
fn infinity_leaves_retain_transitive_body_evidence_but_not_missing_callees() {
    for negative in [false, true] {
        for defined in [false, true] {
            let f = Fixture::with_result(
                &[vec![], vec![], vec![]],
                CObjectType::scalar(CScalarType::F64),
            );
            let mut value = f.values().known_constant(CKnownConstant::DoubleInfinity);
            if negative {
                value = f.values().unary(CUnaryOperator::Negate, value).unwrap();
            }
            let mut bodies = vec![
                f.returning(0, f.call(1, vec![])),
                f.returning(1, f.call(2, vec![])),
            ];
            if defined {
                bodies.push(f.returning(2, value));
            }
            let source = f.source(bodies);
            let summary = ScalarCalls::derive(std::slice::from_ref(&source));
            for function in &f.functions {
                assert_eq!(
                    summary.accepts(&f.values().direct(function.clone()).unwrap()),
                    defined,
                );
            }
            assert_eq!(f.registry.check_storage_paths(&[source]).is_ok(), defined);
        }
    }
}

#[test]
fn standard_stream_pointer_cannot_gain_scalar_call_evidence() {
    for constant in [
        CKnownConstant::StandardInput,
        CKnownConstant::StandardOutput,
        CKnownConstant::StandardError,
    ] {
        let f = Fixture::new(&[0, 0]);
        let mut body = vec![
            f.statements(1)
                .discard(f.values().known_constant(constant))
                .unwrap(),
        ];
        body.extend(f.returning(1, f.literal(1)));
        let source = f.source(vec![f.returning(0, f.call(1, vec![])), body]);
        assert_eq!(accepted(&f, &source), vec![false; 2]);
        assert!(f.registry.check_storage_paths(&[source]).is_err());
    }
}

#[test]
fn initialized_bool_local_assignment_retains_transitive_call_evidence() {
    use super::fixture::key;
    let boolean = CObjectType::scalar(CScalarType::Bool);
    let mut f = Fixture::with_result(&[vec![], vec![]], boolean.clone());
    let local = f
        .registry
        .register_local(&f.scopes[1], key("result"), boolean)
        .unwrap();
    let place = f.values().local(local.clone()).unwrap();
    let no = f.values().literal(CLiteral::Bool(false)).unwrap();
    let yes = f.values().literal(CLiteral::Bool(true)).unwrap();
    let initializer = f.values().expression_initializer(no).unwrap();
    let helper = vec![
        f.statements(1).declare(local, Some(initializer)).unwrap(),
        f.statements(1).assign(place.clone(), yes).unwrap(),
        f.statements(1)
            .return_statement(Some(f.values().read(place).unwrap()))
            .unwrap(),
    ];
    let source = f.source(vec![f.returning(0, f.call(1, vec![])), helper]);
    assert_eq!(accepted(&f, &source), vec![true, true]);
    f.registry.check_storage_paths(&[source]).unwrap();
}

#[test]
fn parameter_and_non_bool_local_writes_remain_outside_scalar_call_evidence() {
    use super::fixture::key;
    let boolean = CObjectType::scalar(CScalarType::Bool);
    let f = Fixture::with_result(&[vec![boolean.clone()]], boolean);
    let place = f.values().parameter(f.parameters[0][0].clone()).unwrap();
    let value = f.values().literal(CLiteral::Bool(false)).unwrap();
    let source = f.source(vec![vec![
        f.statements(0).assign(place, value).unwrap(),
        f.statements(0)
            .return_statement(Some(f.input(0, 0)))
            .unwrap(),
    ]]);
    assert_eq!(accepted(&f, &source), vec![false]);

    let mut f = Fixture::new(&[0]);
    let local = f
        .registry
        .register_local(
            &f.scopes[0],
            key("result"),
            CObjectType::scalar(CScalarType::I32),
        )
        .unwrap();
    let place = f.values().local(local.clone()).unwrap();
    let initializer = f.values().expression_initializer(f.literal(0)).unwrap();
    let source = f.source(vec![vec![
        f.statements(0).declare(local, Some(initializer)).unwrap(),
        f.statements(0).assign(place.clone(), f.literal(1)).unwrap(),
        f.statements(0)
            .return_statement(Some(f.values().read(place).unwrap()))
            .unwrap(),
    ]]);
    assert_eq!(accepted(&f, &source), vec![false]);
}

#[test]
fn scalar_bodies_and_their_transitive_callers_are_checked() {
    let f = Fixture::new(&[1, 3, 0, 1]);
    let source = f.source(vec![
        f.returning(
            0,
            f.call(1, vec![f.input(0, 0), f.literal(2), f.literal(3)]),
        ),
        f.returning(1, f.call(3, vec![f.input(1, 0)])),
        f.returning(2, f.literal(17)),
        f.returning(3, f.input(3, 0)),
    ]);
    assert_eq!(accepted(&f, &source), vec![true; 4]);
    f.registry
        .check_storage_paths(std::slice::from_ref(&source))
        .unwrap();
}

#[test]
fn self_cycles_mutual_cycles_and_their_callers_are_not_effect_evidence() {
    for graph in [vec![1, 1, 3, 2], vec![1, 2, 1, 3]] {
        let f = Fixture::new(&[0; 4]);
        let source = f.source(
            graph
                .into_iter()
                .enumerate()
                .map(|(index, target)| f.returning(index, f.call(target, vec![])))
                .collect(),
        );
        assert_eq!(accepted(&f, &source), vec![false; 4]);
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            Err(CSafetyError::UnprovedStorageCall)
        );
    }
}

#[test]
fn missing_definition_cannot_become_a_leaf_proof() {
    let f = Fixture::new(&[0, 0]);
    let source = f.source(vec![f.returning(0, f.call(1, vec![]))]);
    let summary = ScalarCalls::derive(std::slice::from_ref(&source));
    for function in &f.functions {
        assert!(!summary.accepts(&f.values().direct(function.clone()).unwrap()));
    }
    assert!(f.registry.check_storage_paths(&[source]).is_err());
}

#[test]
fn indirect_calls_do_not_inherit_the_known_functions_summary() {
    let f = Fixture::new(&[0, 0]);
    let callable = f
        .values()
        .indirect(
            f.values().function_address(f.functions[1].clone()).unwrap(),
            f.functions[1].clone(),
        )
        .unwrap();
    let call = f.values().call_value(callable, vec![]).unwrap();
    let source = f.source(vec![f.returning(0, call), f.returning(1, f.literal(1))]);
    assert_eq!(accepted(&f, &source), vec![false, true]);
    assert_eq!(
        f.registry.check_storage_paths(&[source]),
        Err(CSafetyError::UnprovedStorageCall)
    );
}

#[test]
fn a_scalar_result_does_not_hide_a_foreign_call_in_the_body() {
    let f = Fixture::new(&[0, 0]);
    let foreign = f
        .values()
        .call_value(
            f.values().known(crate::dialect::CKnownCall::Allocate),
            vec![
                f.values()
                    .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(8)))
                    .unwrap(),
            ],
        )
        .unwrap();
    let mut body = vec![f.statements(1).discard(foreign).unwrap()];
    body.extend(f.returning(1, f.literal(1)));
    let source = f.source(vec![f.returning(0, f.call(1, vec![])), body]);
    assert_eq!(accepted(&f, &source), vec![false, false]);
    assert!(f.registry.check_storage_paths(&[source]).is_err());
}

#[test]
fn pointer_parameters_are_not_closed_local_storage() {
    let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::scalar(
        CScalarType::I32,
    ))));
    let f = Fixture::typed(&[vec![pointer]]);
    let source = f.source(vec![f.returning(0, f.literal(1))]);
    assert_eq!(accepted(&f, &source), vec![false]);
}

#[test]
fn callee_numeric_errors_and_argument_initialization_still_reject() {
    let f = Fixture::new(&[0, 0]);
    let overflow = f
        .values()
        .binary(CBinaryOperator::Add, f.literal(i32::MAX), f.literal(1))
        .unwrap();
    let source = f.source(vec![
        f.returning(0, f.call(1, vec![])),
        f.returning(1, overflow),
    ]);
    // Purity is deliberately weaker than arithmetic validity.
    assert_eq!(accepted(&f, &source), vec![true, true]);
    assert!(f.registry.check_storage_paths(&[source]).is_err());

    let mut f = Fixture::new(&[0, 1]);
    let local = f
        .registry
        .register_local(
            &f.scopes[0],
            super::fixture::key("missing"),
            CObjectType::scalar(CScalarType::I32),
        )
        .unwrap();
    let read = f
        .values()
        .read(f.values().local(local.clone()).unwrap())
        .unwrap();
    let mut caller = vec![f.statements(0).declare(local, None).unwrap()];
    caller.extend(f.returning(0, f.call(1, vec![read])));
    let source = f.source(vec![caller, f.returning(1, f.input(1, 0))]);
    // This error is already caught by contextual initialization, before effect
    // evidence can authorize evaluation of the call's arguments.
    assert!(ContextFacts::check(&f.registry, std::slice::from_ref(&source)).is_err());
    assert!(f.registry.check_storage_paths(&[source]).is_err());
}
