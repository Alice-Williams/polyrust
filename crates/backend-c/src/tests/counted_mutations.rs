//! Local/context prerequisites and all-syntax mutations cannot be bypassed.
use super::contextual_reconstruction::key;
use super::counted_fixture::Fixture;
use super::*;
use crate::dialect::CKnownCall;

#[test]
fn unreachable_noncanonical_writes_still_fail_the_inventory() {
    let f = Fixture::new();
    let values = f.expressions();
    let ast = f.statements();
    let wrong = ast
        .assign(values.local(f.counter.clone()).unwrap(), f.size(99))
        .unwrap();
    assert_eq!(
        f.run(vec![ast.return_statement(None).unwrap(), wrong]),
        Err(CSafetyError::InvalidLoopMutation)
    );
}

#[test]
fn aliases_in_call_operands_and_indirect_assignments_are_found() {
    for indirect_write in [false, true] {
        let f = Fixture::new();
        let values = f.expressions();
        let ast = f.statements();
        let address = values
            .address_of(values.local(f.counter.clone()).unwrap())
            .unwrap();
        let escape = if indirect_write {
            ast.assign(values.dereference(address).unwrap(), f.size(0))
                .unwrap()
        } else {
            let pointer = values
                .object_to_void(
                    CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified)),
                    address,
                )
                .unwrap();
            ast.evaluate(
                values
                    .call_effect(values.known(CKnownCall::Release), vec![pointer])
                    .unwrap(),
            )
            .unwrap()
        };
        assert_eq!(
            f.run(vec![escape, f.step()]),
            Err(CSafetyError::LoopAddressEscape)
        );
    }
}

#[test]
fn bound_is_const_and_cannot_be_forged_mutable_behind_a_valid_progress_record() {
    let f = Fixture::new();
    let values = f.expressions();
    let ast = f.statements();
    assert!(
        ast.assign(values.local(f.bound.clone()).unwrap(), f.size(0))
            .is_err()
    );
    let mut forged = f.step();
    let CStatementKind::Assign { place, .. } = &mut forged.kind else {
        unreachable!()
    };
    *place = values.local(f.bound.clone()).unwrap();
    assert!(matches!(
        f.run(vec![forged, f.step()]),
        Err(CSafetyError::Context(_))
    ));
}

#[test]
fn missing_or_uninitialized_declarations_are_not_repaired_by_progress_metadata() {
    let f = Fixture::new();
    let ast = f.statements();
    assert!(matches!(
        f.check(vec![
            ast.declare(f.counter.clone(), None).unwrap(),
            f.declare(&f.bound, f.size(3)),
            f.iteration(vec![f.step()])
        ]),
        Err(CSafetyError::Context(_))
    ));
    assert!(matches!(
        f.check(vec![
            f.declare(&f.counter, f.size(0)),
            f.iteration(vec![f.step()])
        ]),
        Err(CSafetyError::Context(_))
    ));
}

#[test]
fn unrelated_runtime_calls_do_not_mutate_unexposed_automatic_counters() {
    let f = Fixture::new();
    let values = f.expressions();
    let ast = f.statements();
    let call = values
        .call_value(
            values.known(CKnownCall::IsNan),
            vec![
                values
                    .numeric_conversion(CScalarType::F64, f.size(0))
                    .unwrap(),
            ],
        )
        .unwrap();
    f.run(vec![ast.discard(call).unwrap(), f.step()]).unwrap();
}

#[test]
fn bound_snapshot_can_depend_on_a_materialized_runtime_call() {
    let mut f = Fixture::new();
    let local = f
        .registry
        .register_local(
            &f.scope,
            key("predicate_result"),
            CObjectType::scalar(CScalarType::Int),
        )
        .unwrap();
    let values = f.expressions();
    let call = values
        .call_value(
            values.known(CKnownCall::IsNan),
            vec![
                values
                    .numeric_conversion(CScalarType::F64, f.size(0))
                    .unwrap(),
            ],
        )
        .unwrap();
    // The call is materialized before conversion, preserving root sequencing.
    f.check(vec![
        f.declare(&local, call),
        f.declare(&f.counter, f.size(0)),
        f.declare(
            &f.bound,
            values
                .numeric_conversion(CScalarType::Size, f.read(&local))
                .unwrap(),
        ),
        f.iteration(vec![f.step()]),
    ])
    .unwrap();
}

#[test]
fn bound_initializer_can_be_a_direct_size_returning_call() {
    let mut f = Fixture::new();
    let helper = f
        .registry
        .register_function(
            &f.file,
            key("bound_source"),
            CFunctionType::new(
                CReturnType::Value(
                    CReturnValue::new(CObjectType::scalar(CScalarType::Size)).unwrap(),
                ),
                vec![],
            ),
        )
        .unwrap();
    let scope = f
        .registry
        .register_scope(&helper, None, key("helper_scope"))
        .unwrap();
    let values = f.expressions();
    let call = values
        .call_value(values.direct(helper.clone()).unwrap(), vec![])
        .unwrap();
    let source = f.source(vec![
        f.declare(&f.counter, f.size(0)),
        f.declare(&f.bound, call),
        f.iteration(vec![f.step()]),
    ]);
    let statements = CStatements::new(&f.registry, helper.clone()).unwrap();
    let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
    let definition = declarations
        .function_definition(
            helper,
            CLinkage::External,
            vec![],
            statements
                .block(
                    scope,
                    vec![statements.return_statement(Some(f.size(3))).unwrap()],
                )
                .unwrap(),
        )
        .unwrap();
    let mut items = source.items().to_vec();
    items.push(CFileItem::Definition(definition));
    f.registry
        .check_counted_loops(&[declarations.source_file(items).unwrap()])
        .unwrap();
}

#[test]
fn constant_predicate_evaluation_respects_numeric_narrowing() {
    let mut f = Fixture::new();
    let then_scope = f.child(&f.body.clone(), "then_branch");
    let else_scope = f.child(&f.body.clone(), "else_branch");
    let values = f.expressions();
    let condition = values
        .numeric_conversion(
            CScalarType::Bool,
            values
                .numeric_conversion(CScalarType::U8, f.size(256))
                .unwrap(),
        )
        .unwrap();
    let ast = f.statements();
    let branch = ast
        .if_statement(
            condition,
            ast.block(then_scope, vec![]).unwrap(),
            ast.block(else_scope, vec![f.step()]).unwrap(),
        )
        .unwrap();
    f.run(vec![branch]).unwrap();
}
