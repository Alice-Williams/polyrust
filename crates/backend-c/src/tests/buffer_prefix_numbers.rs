//! Representation coverage never invents clean across-iteration numeric history.
use super::{allocation_fixture::*, buffer_prefix_fixture::Counted, buffer_symbolic_fixture::*, *};

#[test]
fn prefix_values_keep_unproved_numeric_history_even_after_later_guards() {
    for wrapped in [false, true] {
        let (mut f, buffer, mut body) = setup();
        let construction = Counted::new(&mut f, "construct");
        let extra = raw(&mut f, "extra");
        body.push(construction.declaration(&f));
        let value = if wrapped {
            f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(8))
        } else {
            f.size(7)
        };
        body.push(construction.finish(
            &f,
            buffer.count.local(),
            vec![
                write(&f, &buffer, f.read(&construction.counter), value),
                construction.step(&f),
            ],
        ));
        let value = buffer.read(&f, 0);
        let condition = positive_small(&f, value.clone());
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        body.push(f.declare(&extra, allocate(&f, value)));
        body.push(release(&f, f.read(&extra)));
        body.push(buffer.release(&f));
        assert_eq!(
            f.registry.check_storage_paths(&[f.source(body)]),
            Err(CSafetyError::UnprovedSizeArithmetic)
        );
    }
}
