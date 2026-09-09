//! Early exit retains a frontier, never an unearned full-count claim.
use super::{
    buffer_prefix_fixture::{Counted, immutable_size},
    buffer_symbolic_fixture::*,
    *,
};

#[test]
fn only_a_direct_counter_snapshot_can_authorize_partial_cleanup() {
    for copied_snapshot in [false, true] {
        let (mut f, buffer, mut body) = setup();
        let construction = Counted::new(&mut f, "construct");
        let cleanup = Counted::new(&mut f, "cleanup");
        let first = immutable_size(&mut f, "first_frontier");
        let limit = immutable_size(&mut f, "cleanup_limit");
        body.push(construction.declaration(&f));
        let stop = f.compare(
            CBinaryOperator::Greater,
            f.read(&construction.counter),
            f.input(2),
        );
        let break_now = construction.break_now(&f);
        let exit = construction.branch(&mut f, stop, vec![break_now], vec![]);
        body.push(construction.finish(
            &f,
            buffer.count.local(),
            vec![
                write(&f, &buffer, f.read(&construction.counter), f.size(7)),
                construction.step(&f),
                exit,
            ],
        ));
        body.push(f.declare(&first, f.read(&construction.counter)));
        body.push(f.declare(
            &limit,
            f.read(if copied_snapshot {
                &first
            } else {
                &construction.counter
            }),
        ));
        body.push(cleanup.declaration(&f));
        body.push(cleanup.finish(
            &f,
            &limit,
            vec![
                f.discard(selected(&f, &buffer, f.read(&cleanup.counter))),
                cleanup.step(&f),
            ],
        ));
        body.push(buffer.release(&f));
        let result = f.registry.check_storage_paths(&[f.source(body)]);
        if copied_snapshot {
            assert!(
                result.is_err(),
                "transitive snapshot is outside the grammar: {result:?}"
            );
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}

#[test]
fn an_early_exit_allows_only_reads_strictly_before_its_current_frontier() {
    for at_frontier in [false, true] {
        let (mut f, buffer, mut body) = setup();
        let construction = Counted::new(&mut f, "construct");
        body.push(construction.declaration(&f));
        let actions = vec![
            write(&f, &buffer, f.read(&construction.counter), f.size(7)),
            construction.step(&f),
            construction.break_now(&f),
        ];
        body.push(construction.finish(&f, buffer.count.local(), actions));
        let condition = f.compare(
            CBinaryOperator::Less,
            f.input(1),
            f.read(&construction.counter),
        );
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        let index = if at_frontier {
            f.read(&construction.counter)
        } else {
            f.input(1)
        };
        body.push(f.discard(selected(&f, &buffer, index)));
        body.push(buffer.release(&f));
        assert_eq!(
            f.registry.check_storage_paths(&[f.source(body)]),
            if at_frontier {
                Err(CSafetyError::UninitializedStorage)
            } else {
                Ok(())
            }
        );
    }
}

#[test]
fn a_captured_frontier_supports_a_second_partial_cleanup_loop() {
    for original_count in [false, true] {
        let (mut f, buffer, mut body) = setup();
        let construction = Counted::new(&mut f, "construct");
        let cleanup = Counted::new(&mut f, "cleanup");
        let limit = immutable_size(&mut f, "initialized_count");
        body.push(construction.declaration(&f));
        body.push(construction.finish(
            &f,
            buffer.count.local(),
            vec![
                write(&f, &buffer, f.read(&construction.counter), f.size(7)),
                construction.step(&f),
                construction.break_now(&f),
            ],
        ));
        let captured = if original_count {
            f.read(buffer.count.local())
        } else {
            f.read(&construction.counter)
        };
        body.push(f.declare(&limit, captured));
        body.push(cleanup.declaration(&f));
        body.push(cleanup.finish(
            &f,
            &limit,
            vec![
                f.discard(selected(&f, &buffer, f.read(&cleanup.counter))),
                cleanup.step(&f),
            ],
        ));
        body.push(buffer.release(&f));
        let result = f.registry.check_storage_paths(&[f.source(body)]);
        if original_count {
            assert!(result.is_err(), "{result:?}");
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}
