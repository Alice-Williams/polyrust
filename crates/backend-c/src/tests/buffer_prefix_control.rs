//! Full coverage requires a complete write on every continuing path.
use super::{
    buffer_fixture::Buffer,
    buffer_prefix_fixture::{Counted, immutable_size},
    buffer_symbolic_fixture::*,
    numeric_fixture::Fixture,
    *,
};

#[derive(Clone, Copy, Debug)]
enum Complete {
    Direct,
    BothBranches,
    Continue,
}

#[test]
fn complete_runtime_construction_survives_branches_and_continues() {
    for mode in [Complete::Direct, Complete::BothBranches, Complete::Continue] {
        let (mut f, buffer, mut body) = setup();
        let construction = Counted::new(&mut f, "construct");
        body.push(construction.declaration(&f));
        let write_current = write(&f, &buffer, f.read(&construction.counter), f.size(7));
        let mut actions = match mode {
            Complete::BothBranches => {
                let condition = f.compare(CBinaryOperator::Greater, f.input(2), f.size(0));
                vec![construction.branch(
                    &mut f,
                    condition,
                    vec![write_current.clone()],
                    vec![write_current],
                )]
            }
            _ => vec![write_current],
        };
        actions.push(construction.step(&f));
        if matches!(mode, Complete::Continue) {
            actions.push(construction.continue_now(&f));
        }
        body.push(construction.finish(&f, buffer.count.local(), actions));
        let condition = f.compare(
            CBinaryOperator::Less,
            f.input(1),
            f.read(buffer.count.local()),
        );
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        body.push(f.discard(selected(&f, &buffer, f.input(1))));
        body.push(buffer.release(&f));
        assert_eq!(
            f.registry.check_storage_paths(&[f.source(body)]),
            Ok(()),
            "{mode:?}"
        );
    }
}

#[derive(Clone, Copy, Debug)]
enum Missing {
    Branch,
    Continue,
    Break,
}

#[test]
fn skipped_writes_or_early_exits_do_not_certify_full_coverage() {
    for mode in [Missing::Branch, Missing::Continue, Missing::Break] {
        let (mut f, buffer, mut body) = setup();
        let construction = Counted::new(&mut f, "construct");
        body.push(construction.declaration(&f));
        let write_current = write(&f, &buffer, f.read(&construction.counter), f.size(7));
        let condition = f.compare(CBinaryOperator::Greater, f.input(2), f.size(0));
        let yes = match mode {
            Missing::Branch => vec![],
            Missing::Continue => vec![construction.step(&f), construction.continue_now(&f)],
            Missing::Break => vec![construction.break_now(&f)],
        };
        let no = if matches!(mode, Missing::Branch) {
            vec![write_current.clone()]
        } else {
            vec![]
        };
        let mut actions = vec![construction.branch(&mut f, condition, yes, no)];
        if !matches!(mode, Missing::Branch) {
            actions.push(write_current);
        }
        actions.push(construction.step(&f));
        body.push(construction.finish(&f, buffer.count.local(), actions));
        body.push(f.discard(buffer.read(&f, 0)));
        body.push(buffer.release(&f));
        assert_eq!(
            f.registry.check_storage_paths(&[f.source(body)]),
            Err(CSafetyError::UninitializedStorage),
            "{mode:?}"
        );
    }
}

#[test]
fn writing_after_the_step_cannot_initialize_the_previous_frontier() {
    let (mut f, buffer, mut body) = setup();
    let construction = Counted::new(&mut f, "construct");
    body.push(construction.declaration(&f));
    let condition = f.compare(
        CBinaryOperator::Less,
        f.read(&construction.counter),
        f.read(buffer.count.local()),
    );
    let yes = vec![write(&f, &buffer, f.read(&construction.counter), f.size(7))];
    let no = vec![construction.break_now(&f)];
    let guarded = construction.branch(&mut f, condition, yes, no);
    body.push(construction.finish(
        &f,
        buffer.count.local(),
        vec![construction.step(&f), guarded],
    ));
    body.push(f.discard(buffer.read(&f, 0)));
    body.push(buffer.release(&f));
    assert_eq!(
        f.registry.check_storage_paths(&[f.source(body)]),
        Err(CSafetyError::UninitializedStorage)
    );
}

#[test]
fn unrelated_equal_bounds_or_written_indices_do_not_claim_a_complete_prefix() {
    for wrong_bound in [false, true] {
        for wrong_index in [false, true] {
            let mut f = Fixture::new(&[]);
            let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Size));
            let construction = Counted::new(&mut f, "construct");
            let copied = immutable_size(&mut f, "copied_count");
            let other = f.local(CScalarType::Size, "other_index");
            let count = f.size(2);
            let mut body = buffer.prefix(&mut f, count);
            body.push(f.declare(&copied, f.read(buffer.count.local())));
            body.push(f.declare(&other, f.size(0)));
            body.push(construction.declaration(&f));
            let index = f.read(if wrong_index {
                &other
            } else {
                &construction.counter
            });
            body.push(construction.finish(
                &f,
                if wrong_bound {
                    &copied
                } else {
                    buffer.count.local()
                },
                vec![write(&f, &buffer, index, f.size(7)), construction.step(&f)],
            ));
            body.push(f.discard(buffer.read(&f, 1)));
            body.push(buffer.release(&f));
            assert_eq!(
                f.registry.check_storage_paths(&[f.source(body)]),
                if wrong_bound || wrong_index {
                    Err(CSafetyError::UninitializedStorage)
                } else {
                    Ok(())
                }
            );
        }
    }
}
