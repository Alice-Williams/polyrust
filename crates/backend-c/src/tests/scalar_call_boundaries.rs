//! Effect summaries cannot replace registry, global-state or lifetime checks.
use super::{
    ScalarCalls,
    fixture::{Fixture, key},
};
use crate::ast::*;
use crate::ownership::context_facts::ContextFacts;

#[test]
fn global_reads_and_writes_are_not_closed_local_effects() {
    for write in [false, true] {
        let mut f = Fixture::new(&[0, 0]);
        let object = f
            .registry
            .register_object(
                &f.file,
                key("shared"),
                CObjectType::scalar(CScalarType::I32),
            )
            .unwrap();
        let global = f.values().global(object.clone()).unwrap();
        let mut helper = vec![];
        if write {
            helper.push(
                f.statements(1)
                    .assign(global.clone(), f.literal(2))
                    .unwrap(),
            );
        }
        let result = if write {
            f.literal(1)
        } else {
            f.values().read(global).unwrap()
        };
        helper.extend(f.returning(1, result));
        let source = f.source(vec![f.returning(0, f.call(1, vec![])), helper]);
        let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
        let mut items = vec![CFileItem::Definition(
            declarations
                .object_definition(
                    object,
                    CLinkage::External,
                    f.values().expression_initializer(f.literal(1)).unwrap(),
                )
                .unwrap(),
        )];
        items.extend(source.items().iter().cloned());
        let source = declarations.source_file(items).unwrap();
        let facts = ContextFacts::check(&f.registry, std::slice::from_ref(&source)).unwrap();
        assert!(
            f.functions
                .iter()
                .all(|function| !facts.scalar_call(&f.values().direct(function.clone()).unwrap()))
        );
        assert!(f.registry.check_storage_paths(&[source]).is_err());
    }
}

#[test]
fn same_spelling_and_signature_from_a_foreign_registry_is_not_the_proved_target() {
    let f = Fixture::new(&[0]);
    let source = f.source(vec![f.returning(0, f.literal(1))]);
    let files = [source];
    let facts = ContextFacts::check(&f.registry, &files).unwrap();
    assert!(facts.scalar_call(&f.values().direct(f.functions[0].clone()).unwrap()));
    let foreign = Fixture::new(&[0]);
    assert!(
        !facts.scalar_call(
            &foreign
                .values()
                .direct(foreign.functions[0].clone())
                .unwrap()
        )
    );
}

#[test]
fn boolean_arguments_require_a_sequenced_call_result_temporary() {
    let mut f = Fixture::typed(&[
        vec![CObjectType::scalar(CScalarType::Bool)],
        vec![CObjectType::scalar(CScalarType::Bool)],
    ]);
    let call = f.call(1, vec![f.input(0, 0)]);
    let compared = f
        .values()
        .binary(CBinaryOperator::Equal, call.clone(), f.literal(1))
        .unwrap();
    let result = f
        .values()
        .numeric_conversion(CScalarType::I32, compared)
        .unwrap();
    let helper = f
        .values()
        .numeric_conversion(CScalarType::I32, f.input(1, 0))
        .unwrap();
    let source = f.source(vec![f.returning(0, result), f.returning(1, helper.clone())]);
    assert_eq!(
        f.registry.check_storage_paths(&[source]),
        Err(crate::ownership::CSafetyError::UnsequencedCall)
    );

    let temporary = f
        .registry
        .register_local(
            &f.scopes[0],
            key("result"),
            CObjectType::scalar(CScalarType::I32),
        )
        .unwrap();
    let declaration = f
        .statements(0)
        .declare(
            temporary.clone(),
            Some(f.values().expression_initializer(call).unwrap()),
        )
        .unwrap();
    let read = f
        .values()
        .read(f.values().local(temporary).unwrap())
        .unwrap();
    let compared = f
        .values()
        .binary(CBinaryOperator::Equal, read, f.literal(1))
        .unwrap();
    let result = f
        .values()
        .numeric_conversion(CScalarType::I32, compared)
        .unwrap();
    let mut caller = vec![declaration];
    caller.extend(f.returning(0, result));
    let source = f.source(vec![caller, f.returning(1, helper)]);
    f.registry.check_storage_paths(&[source]).unwrap();
}

#[test]
fn missing_bodies_and_duplicate_definitions_fail_context_before_summary_use() {
    let f = Fixture::new(&[0]);
    let source = f.source(vec![f.returning(0, f.literal(1))]);
    let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
    for duplicate in [false, true] {
        let mut items = source.items().to_vec();
        if duplicate {
            items.push(items.last().unwrap().clone());
        } else {
            items.pop();
        }
        let changed = declarations.source_file(items).unwrap();
        assert!(ContextFacts::check(&f.registry, &[changed]).is_err());
    }
    let unchanged = ScalarCalls::derive(&[source]);
    assert!(unchanged.accepts(&f.values().direct(f.functions[0].clone()).unwrap()));
}

#[test]
fn bool_results_are_closed_but_pointer_results_are_not() {
    let boolean = CObjectType::scalar(CScalarType::Bool);
    let f = Fixture::with_result(&[vec![boolean.clone()], vec![boolean.clone()]], boolean);
    let source = f.source(vec![
        f.returning(0, f.call(1, vec![f.input(0, 0)])),
        f.returning(1, f.input(1, 0)),
    ]);
    f.registry
        .check_storage_paths(std::slice::from_ref(&source))
        .unwrap();
    let summary = ScalarCalls::derive(&[source]);
    assert!(
        f.functions
            .iter()
            .all(|function| summary.accepts(&f.values().direct(function.clone()).unwrap()))
    );

    let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::scalar(
        CScalarType::I32,
    ))));
    let f = Fixture::with_result(&[vec![]], pointer.clone());
    let value = f
        .values()
        .literal(CLiteral::NullPointer(CNullPointer::new(pointer).unwrap()))
        .unwrap();
    let source = f.source(vec![f.returning(0, value)]);
    let facts = ContextFacts::check(&f.registry, std::slice::from_ref(&source)).unwrap();
    assert!(!facts.scalar_call(&f.values().direct(f.functions[0].clone()).unwrap()));
}
