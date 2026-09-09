//! Actual cleanup and return paths must account for every raw allocation.
use super::{
    allocation_fixture::*,
    contextual_reconstruction::{fixture_return, key, package},
    numeric_fixture::Fixture,
    storage_fixture::check,
    *,
};
use crate::dialect::CKnownCall;

#[test]
fn cleanup_jump_must_not_bypass_release() {
    for bypass in [false, true] {
        let mut f = Fixture::new(&[]);
        let pointer = raw(&mut f, "pointer");
        let label = f
            .registry
            .register_cleanup_exit(&f.scope, key("done"))
            .unwrap();
        let cleanup = release(&f, f.read(&pointer));
        let mut body = vec![f.declare(&pointer, allocate(&f, f.size(8)))];
        if bypass {
            body.extend([f.ast().cleanup_jump(label.clone()).unwrap(), cleanup]);
        } else {
            body.extend([cleanup, f.ast().cleanup_jump(label.clone()).unwrap()]);
        }
        body.push(
            f.ast()
                .label(label, f.ast().return_statement(None).unwrap())
                .unwrap(),
        );
        check(
            &f,
            body,
            if bypass {
                Err(CSafetyError::UnreleasedAllocation)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn raw_allocation_return_cannot_claim_an_ownership_transfer_contract() {
    let ty = CObjectType::pointer(CPointerTarget::Void(CConstness::Unqualified));
    let (registry, file, function, scope) =
        fixture_return(CReturnType::Value(CReturnValue::new(ty).unwrap()));
    let e = CExpressions::new(&registry);
    let bytes = e
        .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(8)))
        .unwrap();
    let call = e
        .call_value(e.known(CKnownCall::Allocate), vec![bytes])
        .unwrap();
    let result = CStatements::new(&registry, function.clone())
        .unwrap()
        .return_statement(Some(call))
        .unwrap();
    let source = package(&registry, file, function, scope, vec![result]);
    assert_eq!(
        registry.check_storage_paths(&[source]),
        Err(CSafetyError::UnreleasedAllocation)
    );
}

#[test]
fn release_on_only_one_runtime_branch_leaves_an_outstanding_resource() {
    for both in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Bool]);
        let pointer = raw(&mut f, "pointer");
        let cleanup = release(&f, f.read(&pointer));
        let branch = f.branch(
            f.input(0),
            vec![cleanup.clone()],
            if both { vec![cleanup] } else { vec![] },
        );
        check(
            &f,
            vec![f.declare(&pointer, allocate(&f, f.size(8))), branch],
            if both {
                Ok(())
            } else {
                Err(CSafetyError::UnreleasedAllocation)
            },
        );
    }
}

#[test]
fn null_test_of_a_copy_refines_the_original_and_keeps_polarity() {
    for negate in [false, true] {
        let mut f = Fixture::new(&[]);
        let pointer = raw(&mut f, "pointer");
        let copy = raw(&mut f, "copy");
        let test = f
            .values()
            .pointer_test(CPointerTest::IsNull(Box::new(f.read(&copy))))
            .unwrap();
        let condition = if negate {
            f.values()
                .unary(CUnaryOperator::LogicalNot, f.boolean(test))
                .unwrap()
        } else {
            test
        };
        let cleanup = release(&f, f.read(&pointer));
        let returned = f.ast().return_statement(None).unwrap();
        let branch = f.branch(
            f.boolean(condition),
            if negate {
                vec![cleanup.clone()]
            } else {
                vec![returned.clone()]
            },
            if negate {
                vec![returned]
            } else {
                vec![cleanup]
            },
        );
        check(
            &f,
            vec![
                f.declare(&pointer, allocate(&f, f.size(8))),
                f.declare(&copy, f.read(&pointer)),
                branch,
            ],
            Ok(()),
        );
    }
}
