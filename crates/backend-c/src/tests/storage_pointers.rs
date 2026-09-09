//! Useful derived pointer reads, exact extents, nulls and qualification controls.
use super::{index_extent_fixture as arrays, numeric_fixture::Fixture, storage_fixture::*, *};

#[test]
fn fresh_storage_address_is_not_a_read_and_pointer_writes_establish_initialization() {
    for write in [false, true] {
        let mut f = Fixture::new(&[]);
        let value = f.local(CScalarType::Int, "value");
        let pointer = local(&mut f, pointer_type(value.ty().clone()), "pointer");
        let mut body = vec![
            f.ast().declare(value.clone(), None).unwrap(),
            f.declare(&pointer, address(&f, f.values().local(value).unwrap())),
        ];
        if write {
            body.push(
                f.ast()
                    .assign(f.values().dereference(f.read(&pointer)).unwrap(), f.int(7))
                    .unwrap(),
            );
        }
        body.push(f.discard(pointed_read(&f, f.read(&pointer))));
        check(
            &f,
            body,
            if write {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}

#[test]
fn copied_array_element_pointer_keeps_origin_offset_and_actual_extent() {
    for origin in [0, 1, 2] {
        for offset in [0, 1, 2, 3] {
            let mut f = Fixture::new(&[]);
            let array = arrays::array(&mut f, 3, "array");
            let pointer = local(
                &mut f,
                pointer_type(CObjectType::scalar(CScalarType::Int)),
                "pointer",
            );
            let copied = local(&mut f, pointer.ty().clone(), "copied");
            let body = vec![
                arrays::declare(&f, &array),
                f.declare(
                    &pointer,
                    address(&f, arrays::place(&f, &array, f.size(origin))),
                ),
                f.declare(&copied, f.read(&pointer)),
                f.discard(
                    f.values()
                        .read(pointer_index(&f, f.read(&copied), offset))
                        .unwrap(),
                ),
            ];
            check(
                &f,
                body,
                if origin + offset < 3 {
                    Ok(())
                } else {
                    Err(CSafetyError::IndexOutOfBounds)
                },
            );
        }
    }
}

#[test]
fn scalar_pointer_cannot_acquire_an_array_extent_and_null_cannot_be_dereferenced() {
    for offset in [0, 1] {
        let mut f = Fixture::new(&[]);
        let value = f.local(CScalarType::Int, "value");
        let pointer = address(&f, f.values().local(value.clone()).unwrap());
        check(
            &f,
            vec![
                f.declare(&value, f.int(5)),
                f.discard(f.values().read(pointer_index(&f, pointer, offset)).unwrap()),
            ],
            if offset == 0 {
                Ok(())
            } else {
                Err(CSafetyError::IndexOutOfBounds)
            },
        );
    }
    let f = Fixture::new(&[]);
    let null = f
        .values()
        .literal(CLiteral::NullPointer(
            CNullPointer::new(pointer_type(CObjectType::scalar(CScalarType::Int))).unwrap(),
        ))
        .unwrap();
    check(
        &f,
        vec![f.discard(pointed_read(&f, null))],
        Err(CSafetyError::NullStorage),
    );
}

#[test]
fn const_qualification_preserves_address_and_dereference_proof() {
    let mut f = Fixture::new(&[]);
    let value = f.local(CScalarType::Int, "value");
    let pointer = address(&f, f.values().local(value.clone()).unwrap());
    let qualified = f
        .values()
        .add_const(
            pointer_type(
                value
                    .ty()
                    .clone()
                    .with_constness(CConstness::Const)
                    .unwrap(),
            ),
            pointer,
        )
        .unwrap();
    check(
        &f,
        vec![
            f.declare(&value, f.int(5)),
            f.discard(pointed_read(&f, qualified)),
        ],
        Ok(()),
    );
}

#[test]
fn guarded_interval_address_can_be_read_through_a_derived_pointer() {
    for bound in [3, 4] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let array = arrays::array(&mut f, 3, "array");
        let pointer = address(&f, arrays::place(&f, &array, f.input(0)));
        let read = f.discard(pointed_read(&f, pointer));
        let branch = f.branch(
            f.compare(CBinaryOperator::Less, f.input(0), f.size(bound)),
            vec![read],
            vec![],
        );
        check(
            &f,
            vec![arrays::declare(&f, &array), branch],
            if bound == 3 {
                Ok(())
            } else {
                Err(CSafetyError::IndexOutOfBounds)
            },
        );
    }
}

#[test]
fn zero_arrays_are_compressed_even_for_large_extents() {
    let mut f = Fixture::new(&[]);
    let array = arrays::array(&mut f, 1_000_000_000_000, "array");
    let pointer = address(&f, arrays::place(&f, &array, f.size(999_999_999_999)));
    check(
        &f,
        vec![
            arrays::declare(&f, &array),
            f.discard(pointed_read(&f, pointer)),
        ],
        Ok(()),
    );
}

#[test]
fn nested_pointer_offsets_keep_row_and_element_extents_separate() {
    for rows in [false, true] {
        for offset in [0, 1, 2] {
            let mut f = Fixture::new(&[]);
            let row_type = CObjectType::array(
                CObjectType::scalar(CScalarType::Int),
                CArrayLength::new(3).unwrap(),
            )
            .unwrap();
            let matrix = local(
                &mut f,
                CObjectType::array(row_type, CArrayLength::new(2).unwrap()).unwrap(),
                "matrix",
            );
            let row = arrays::place(&f, &matrix, f.size(0));
            let selected = if rows {
                let shifted = pointer_index(&f, address(&f, row), offset);
                f.values()
                    .index(CIndexBase::Array(Box::new(shifted)), f.size(2))
                    .unwrap()
            } else {
                let last = f
                    .values()
                    .index(CIndexBase::Array(Box::new(row)), f.size(2))
                    .unwrap();
                pointer_index(&f, address(&f, last), offset)
            };
            check(
                &f,
                vec![
                    arrays::declare(&f, &matrix),
                    f.discard(f.values().read(selected).unwrap()),
                ],
                if offset == 0 || (rows && offset == 1) {
                    Ok(())
                } else {
                    Err(CSafetyError::IndexOutOfBounds)
                },
            );
        }
    }
}
