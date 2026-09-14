//! A call signature is not purity, storage validity or a rendering certificate.
use super::{ScalarCalls, fixture::Fixture};
use crate::ast::*;
use crate::ownership::{CSafetyError, context_facts::ContextFacts};

fn accepted(f: &Fixture, source: &CSourceFile) -> Vec<bool> {
    let facts = ContextFacts::check(&f.registry, std::slice::from_ref(source)).unwrap();
    f.functions
        .iter()
        .map(|function| facts.scalar_call(&f.values().direct(function.clone()).unwrap()))
        .collect()
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
