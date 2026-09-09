//! Expression-local guards and counted-loop phases flow into extent checking.
use super::{
    contextual_reconstruction::key, counted_fixture, index_extent_fixture::*,
    numeric_fixture::Fixture, *,
};

#[test]
fn short_circuit_and_conditional_children_keep_their_local_guard() {
    for conditional in [false, true] {
        for guarded in [false, true] {
            let mut f = Fixture::new(&[CScalarType::Size]);
            let local = array(&mut f, 2, "array");
            let bound = if guarded { 2 } else { 3 };
            let guard = f.compare(CBinaryOperator::Less, f.input(0), f.size(bound));
            let selected = read(&f, &local, f.input(0));
            let expression = if conditional {
                f.values().conditional(guard, selected, f.int(0)).unwrap()
            } else {
                f.binary(CBinaryOperator::LogicalAnd, guard, f.boolean(selected))
            };
            check(
                &f,
                vec![declare(&f, &local), f.discard(expression)],
                if guarded {
                    Ok(())
                } else {
                    Err(CSafetyError::IndexOutOfBounds)
                },
            );
        }
    }
}

#[test]
fn a_proved_unreachable_index_does_not_gain_a_runtime_obligation() {
    for selected in [false, true] {
        let mut f = Fixture::new(&[]);
        let local = array(&mut f, 2, "array");
        let invalid = f.discard(read(&f, &local, f.size(2)));
        let condition = f.values().literal(CLiteral::Bool(selected)).unwrap();
        let branch = f.branch(condition, vec![invalid], vec![]);
        check(
            &f,
            vec![declare(&f, &local), branch],
            if selected {
                Err(CSafetyError::IndexOutOfBounds)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn actual_loop_step_phase_controls_the_array_bound() {
    for after in [false, true] {
        for bound in [0, 1, 3] {
            let mut f = counted_fixture::Fixture::new();
            let ty = CObjectType::array(
                CObjectType::scalar(CScalarType::Int),
                CArrayLength::new(3).unwrap(),
            )
            .unwrap();
            let local = f
                .registry
                .register_local(&f.scope, key("array"), ty)
                .unwrap();
            let selected = f
                .expressions()
                .index(
                    CIndexBase::Array(Box::new(f.expressions().local(local.clone()).unwrap())),
                    f.read(&f.counter),
                )
                .unwrap();
            let access = f
                .statements()
                .discard(f.expressions().address_of(selected).unwrap())
                .unwrap();
            let body = if after {
                vec![f.step(), access]
            } else {
                vec![access, f.step()]
            };
            let source = f.source(vec![
                f.statements().declare(local, None).unwrap(),
                f.declare(&f.counter, f.size(0)),
                f.declare(&f.bound, f.size(bound)),
                f.iteration(body),
            ]);
            assert_eq!(
                f.registry.check_index_extents(&[source]),
                if after && bound == 3 {
                    Err(CSafetyError::IndexOutOfBounds)
                } else {
                    Ok(())
                }
            );
        }
    }
}
