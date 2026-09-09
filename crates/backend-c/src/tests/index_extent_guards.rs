//! Guard dominance and invalidation must reach the exact array access.
use super::{index_extent_fixture::*, numeric_fixture::Fixture, *};

#[test]
fn signed_runtime_indices_need_both_lower_and_upper_bounds() {
    for lower in [false, true] {
        for reversed in [false, true] {
            for bound in [2, 3] {
                let mut f = Fixture::new(&[CScalarType::Int]);
                let local = array(&mut f, 2, "array");
                let upper = if reversed {
                    f.compare(CBinaryOperator::Greater, f.int(bound), f.input(0))
                } else {
                    f.compare(CBinaryOperator::Less, f.input(0), f.int(bound))
                };
                let condition = if lower {
                    f.boolean(f.binary(
                        CBinaryOperator::LogicalAnd,
                        f.compare(CBinaryOperator::GreaterEqual, f.input(0), f.int(0)),
                        upper,
                    ))
                } else {
                    upper
                };
                let access = f.discard(read(&f, &local, f.input(0)));
                let branch = f.branch(condition, vec![access], vec![]);
                check(
                    &f,
                    vec![declare(&f, &local), branch],
                    if lower && bound == 2 {
                        Ok(())
                    } else {
                        Err(CSafetyError::IndexOutOfBounds)
                    },
                );
            }
        }
    }
}

#[test]
fn a_guard_on_another_value_or_before_a_write_cannot_authorize_an_index() {
    for stale in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size]);
        let local = array(&mut f, 2, "array");
        let index = f.local(CScalarType::Size, "index");
        let condition = f.compare(
            CBinaryOperator::Less,
            if stale { f.read(&index) } else { f.input(1) },
            f.size(2),
        );
        let mut yes = vec![];
        if stale {
            yes.push(
                f.ast()
                    .assign(f.values().local(index.clone()).unwrap(), f.size(2))
                    .unwrap(),
            );
        }
        yes.push(f.discard(read(&f, &local, f.read(&index))));
        let branch = f.branch(condition, yes, vec![]);
        check(
            &f,
            vec![declare(&f, &local), f.declare(&index, f.input(0)), branch],
            Err(CSafetyError::IndexOutOfBounds),
        );
    }
}

#[test]
fn joins_require_a_bounded_value_from_every_predecessor() {
    for mask in 0..4 {
        let mut f = Fixture::new(&[CScalarType::Bool, CScalarType::Size]);
        let local = array(&mut f, 2, "array");
        let index = f.local(CScalarType::Size, "index");
        let assignment = || {
            f.ast()
                .assign(f.values().local(index.clone()).unwrap(), f.size(1))
                .unwrap()
        };
        let yes = if mask & 1 != 0 {
            vec![assignment()]
        } else {
            vec![]
        };
        let no = if mask & 2 != 0 {
            vec![assignment()]
        } else {
            vec![]
        };
        let branch = f.branch(f.input(0), yes, no);
        check(
            &f,
            vec![
                declare(&f, &local),
                f.declare(&index, f.input(1)),
                branch,
                f.discard(read(&f, &local, f.read(&index))),
            ],
            if mask == 3 {
                Ok(())
            } else {
                Err(CSafetyError::IndexOutOfBounds)
            },
        );
    }
}

#[test]
fn a_guard_inside_a_finished_branch_does_not_dominate_a_later_access() {
    let mut f = Fixture::new(&[CScalarType::Size]);
    let local = array(&mut f, 2, "array");
    let branch = f.branch(
        f.compare(CBinaryOperator::Less, f.input(0), f.size(2)),
        vec![],
        vec![],
    );
    check(
        &f,
        vec![
            declare(&f, &local),
            branch,
            f.discard(read(&f, &local, f.input(0))),
        ],
        Err(CSafetyError::IndexOutOfBounds),
    );
}
