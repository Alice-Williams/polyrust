//! Alias writes, interval initialization and nominally distinct roots.
use super::{index_extent_fixture as arrays, numeric_fixture::Fixture, storage_fixture::*, *};

#[test]
fn an_alias_write_replaces_the_stored_pointer_provenance() {
    let mut f = Fixture::new(&[]);
    let value = f.local(CScalarType::Int, "value");
    let pointer = local(&mut f, pointer_type(value.ty().clone()), "pointer");
    let alias = address(&f, f.values().local(pointer.clone()).unwrap());
    let null = f
        .values()
        .literal(CLiteral::NullPointer(
            CNullPointer::new(pointer.ty().clone()).unwrap(),
        ))
        .unwrap();
    check(
        &f,
        vec![
            f.declare(&value, f.int(1)),
            f.declare(&pointer, address(&f, f.values().local(value).unwrap())),
            f.ast()
                .assign(f.values().dereference(alias).unwrap(), null)
                .unwrap(),
            f.discard(pointed_read(&f, f.read(&pointer))),
        ],
        Err(CSafetyError::NullStorage),
    );
}

#[test]
fn sparse_prefix_covers_an_interval_but_does_not_cover_unwritten_neighbors() {
    for bound in [2, 3] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let array = arrays::array(&mut f, 3, "array");
        let base = address(&f, arrays::place(&f, &array, f.size(0)));
        let read = f
            .values()
            .read(
                f.values()
                    .index(CIndexBase::Pointer(Box::new(base.clone())), f.input(0))
                    .unwrap(),
            )
            .unwrap();
        let branch = f.branch(
            f.compare(CBinaryOperator::Less, f.input(0), f.size(bound)),
            vec![f.discard(read)],
            vec![],
        );
        check(
            &f,
            vec![
                f.ast().declare(array, None).unwrap(),
                f.ast()
                    .assign(pointer_index(&f, base.clone(), 0), f.int(1))
                    .unwrap(),
                f.ast()
                    .assign(pointer_index(&f, base, 1), f.int(1))
                    .unwrap(),
                branch,
            ],
            if bound == 2 {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}

#[test]
fn unknown_index_write_cannot_claim_any_specific_element_initialized() {
    let mut f = Fixture::new(&[CScalarType::Size]);
    let array = arrays::array(&mut f, 2, "array");
    let base = address(&f, arrays::place(&f, &array, f.size(0)));
    let write = f
        .ast()
        .assign(
            f.values()
                .index(CIndexBase::Pointer(Box::new(base.clone())), f.input(0))
                .unwrap(),
            f.int(1),
        )
        .unwrap();
    let read = f.discard(f.values().read(pointer_index(&f, base, 0)).unwrap());
    let branch = f.branch(
        f.compare(CBinaryOperator::Less, f.input(0), f.size(2)),
        vec![write, read],
        vec![],
    );
    check(
        &f,
        vec![f.ast().declare(array, None).unwrap(), branch],
        Err(CSafetyError::UninitializedStorage),
    );
}

#[test]
fn pointer_join_requires_actual_backing_object_agreement_not_equal_types_or_values() {
    for dereference in [false, true] {
        for same in [false, true] {
            let mut f = Fixture::new(&[CScalarType::Bool]);
            let first = f.local(CScalarType::Int, "first");
            let second = f.local(CScalarType::Int, "second");
            let pointer = local(&mut f, pointer_type(first.ty().clone()), "pointer");
            let destination = f.values().local(pointer.clone()).unwrap();
            let left = f
                .ast()
                .assign(
                    destination.clone(),
                    address(&f, f.values().local(first.clone()).unwrap()),
                )
                .unwrap();
            let right = f
                .ast()
                .assign(
                    destination,
                    address(
                        &f,
                        f.values()
                            .local(if same { first.clone() } else { second.clone() })
                            .unwrap(),
                    ),
                )
                .unwrap();
            let branch = f.branch(f.input(0), vec![left], vec![right]);
            check(
                &f,
                vec![
                    f.declare(&first, f.int(1)),
                    f.declare(&second, f.int(1)),
                    arrays::declare(&f, &pointer),
                    branch,
                    f.discard(if dereference {
                        pointed_read(&f, f.read(&pointer))
                    } else {
                        f.read(&pointer)
                    }),
                ],
                if same {
                    Ok(())
                } else {
                    Err(CSafetyError::UnprovedStorage)
                },
            );
        }
    }
}

#[test]
fn discarded_conditional_pointer_results_still_require_known_provenance() {
    for same in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Bool]);
        let a = f.local(CScalarType::Int, "a");
        let b = f.local(CScalarType::Int, "b");
        let left = address(&f, f.values().local(a.clone()).unwrap());
        let right = address(
            &f,
            f.values()
                .local(if same { a.clone() } else { b.clone() })
                .unwrap(),
        );
        let value = f.values().conditional(f.input(0), left, right).unwrap();
        check(
            &f,
            vec![
                f.declare(&a, f.int(1)),
                f.declare(&b, f.int(1)),
                f.discard(value),
            ],
            if same {
                Ok(())
            } else {
                Err(CSafetyError::UnprovedStorage)
            },
        );
    }
}

#[test]
fn discarded_interval_pointer_read_requires_all_selected_origins_to_agree() {
    for same in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let a = f.local(CScalarType::Int, "a");
        let b = f.local(CScalarType::Int, "b");
        let pointers = local(
            &mut f,
            CObjectType::array(pointer_type(a.ty().clone()), CArrayLength::new(2).unwrap())
                .unwrap(),
            "pointers",
        );
        let base = f.values().local(pointers.clone()).unwrap();
        let slot = |index| {
            f.values()
                .index(CIndexBase::Array(Box::new(base.clone())), index)
                .unwrap()
        };
        let left = f
            .ast()
            .assign(
                slot(f.size(0)),
                address(&f, f.values().local(a.clone()).unwrap()),
            )
            .unwrap();
        let right = f
            .ast()
            .assign(
                slot(f.size(1)),
                address(
                    &f,
                    f.values()
                        .local(if same { a.clone() } else { b.clone() })
                        .unwrap(),
                ),
            )
            .unwrap();
        let read = f.discard(f.values().read(slot(f.input(0))).unwrap());
        let branch = f.branch(
            f.compare(CBinaryOperator::Less, f.input(0), f.size(2)),
            vec![read],
            vec![],
        );
        check(
            &f,
            vec![
                f.declare(&a, f.int(1)),
                f.declare(&b, f.int(1)),
                arrays::declare(&f, &pointers),
                left,
                right,
                branch,
            ],
            if same {
                Ok(())
            } else {
                Err(CSafetyError::UnprovedStorage)
            },
        );
    }
}
