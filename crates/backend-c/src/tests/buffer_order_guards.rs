//! Runtime indices require a real current relation to the original captured count.
use super::contextual_reconstruction::key;
use super::{buffer_fixture::Buffer, numeric_fixture::Fixture, storage_fixture::*, *};

#[derive(Clone, Copy, Debug)]
enum Guard {
    Matching,
    Reversed,
    FalseArm,
    OtherCount,
    StaleIndex,
    Unguarded,
    Disjunction,
}

#[test]
fn only_actual_strict_current_count_relations_admit_runtime_element_addresses() {
    for form in [
        Guard::Matching,
        Guard::Reversed,
        Guard::FalseArm,
        Guard::OtherCount,
        Guard::StaleIndex,
        Guard::Unguarded,
        Guard::Disjunction,
    ] {
        let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size, CScalarType::Bool]);
        let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::U8));
        let positive = f.compare(CBinaryOperator::Greater, f.input(0), f.size(0));
        let positive = f.branch(
            positive,
            vec![],
            vec![f.ast().return_statement(None).unwrap()],
        );
        let mut body = vec![positive];
        let length = f.input(0);
        body.extend(buffer.prefix(&mut f, length));
        let count = if matches!(form, Guard::OtherCount) {
            let other = f
                .registry
                .register_buffer_count(&f.scope, key("other_count"))
                .unwrap();
            body.push(f.declare(other.local(), f.read(buffer.count.local())));
            f.read(other.local())
        } else {
            f.read(buffer.count.local())
        };
        let guard = match form {
            Guard::Reversed => f.compare(CBinaryOperator::Greater, count, f.input(1)),
            Guard::FalseArm => f.compare(CBinaryOperator::GreaterEqual, f.input(1), count),
            _ => f.compare(CBinaryOperator::Less, f.input(1), count),
        };
        let guard = if matches!(form, Guard::Disjunction) {
            f.boolean(f.binary(CBinaryOperator::LogicalOr, guard, f.input(2)))
        } else {
            guard
        };
        let mut selected = vec![];
        if matches!(form, Guard::StaleIndex) {
            selected.push(
                f.ast()
                    .assign(
                        f.values().parameter(f.parameters[1].clone()).unwrap(),
                        f.size(u64::MAX),
                    )
                    .unwrap(),
            );
        }
        selected.push(f.discard(address(&f, buffer.place(&f, f.input(1)))));
        match form {
            Guard::Unguarded => body.extend(selected),
            Guard::FalseArm => body.push(f.branch(guard, vec![], selected)),
            _ => body.push(f.branch(guard, selected, vec![])),
        }
        body.push(buffer.release(&f));
        let expected = if matches!(form, Guard::Matching | Guard::Reversed | Guard::FalseArm) {
            Ok(())
        } else {
            Err(CSafetyError::IndexOutOfBounds)
        };
        assert_eq!(
            f.registry.check_storage_paths(&[f.source(body)]),
            expected,
            "{form:?}"
        );
    }
}
