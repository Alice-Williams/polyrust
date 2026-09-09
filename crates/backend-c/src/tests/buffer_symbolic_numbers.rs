//! Current writes carry numbers; potential aliases cannot erase their loss history.
use super::{allocation_fixture::*, heap_fixture::assign};
use super::{buffer_symbolic_fixture::*, storage_fixture::check, *};

#[test]
fn exact_runtime_selection_keeps_clean_or_wrapped_numeric_history() {
    for wrapped in [false, true] {
        let (mut f, buffer, mut body) = setup();
        let extra = raw(&mut f, "extra");
        body.push(f.declare(&extra, null(&f, &extra)));
        let value = if wrapped {
            f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(8))
        } else {
            f.size(7)
        };
        let actions = vec![
            write(&f, &buffer, f.input(1), value),
            assign(&f, &extra, allocate(&f, selected(&f, &buffer, f.input(1)))),
            release(&f, f.read(&extra)),
        ];
        let index = f.input(1);
        body.push(within(&mut f, &buffer, index, actions));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if wrapped {
                Err(CSafetyError::UnprovedSizeArithmetic)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn a_symbolic_write_poisoning_a_constant_observation_cannot_be_repaired_by_a_later_guard() {
    let (mut f, buffer, mut body) = setup();
    let extra = raw(&mut f, "extra");
    body.push(f.declare(&extra, null(&f, &extra)));
    body.push(buffer.write(&f, 0, f.size(8)));
    body.push(buffer.write(&f, 1, f.size(8)));
    let wrapped = f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(9));
    let action = write(&f, &buffer, f.input(1), wrapped);
    let index = f.input(1);
    body.push(within(&mut f, &buffer, index, vec![action]));
    let guard = positive_small(&f, buffer.read(&f, 0));
    let actions = vec![
        assign(&f, &extra, allocate(&f, buffer.read(&f, 0))),
        release(&f, f.read(&extra)),
    ];
    body.push(f.branch(guard, actions, vec![]));
    body.push(buffer.release(&f));
    check(&f, body, Err(CSafetyError::UnprovedSizeArithmetic));
}

#[test]
fn unseen_symbolic_reads_cannot_skip_losses_in_initialized_constant_elements() {
    let (mut f, buffer, mut body) = setup();
    let extra = raw(&mut f, "extra");
    body.push(f.declare(&extra, null(&f, &extra)));
    body.push(buffer.write(
        &f,
        0,
        f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(9)),
    ));
    body.push(buffer.write(&f, 1, f.size(8)));
    let index_bound = f.compare(CBinaryOperator::Less, f.input(1), f.size(2));
    let value_bound = positive_small(&f, selected(&f, &buffer, f.input(1)));
    let guard = f.boolean(f.binary(CBinaryOperator::LogicalAnd, index_bound, value_bound));
    let actions = vec![
        assign(&f, &extra, allocate(&f, selected(&f, &buffer, f.input(1)))),
        release(&f, f.read(&extra)),
    ];
    body.push(f.branch(guard, actions, vec![]));
    body.push(buffer.release(&f));
    check(&f, body, Err(CSafetyError::UnprovedSizeArithmetic));
}
