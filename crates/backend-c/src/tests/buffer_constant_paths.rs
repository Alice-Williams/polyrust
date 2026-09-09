//! Dynamic containers keep element bounds, initialization and base provenance.
use super::{allocation_fixture::*, heap_fixture::*, storage_fixture::*};
use super::{buffer_fixture::Buffer, numeric_fixture::Fixture, *};

#[test]
fn sparse_elements_are_distinct_and_accesses_use_original_count() {
    for index in 0..=2 {
        let mut f = Fixture::new(&[]);
        let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Int));
        let count = f.size(2);
        let mut body = buffer.prefix(&mut f, count);
        body.push(buffer.write(&f, 0, f.int(10)));
        body.push(buffer.write(&f, 1, f.int(20)));
        body.push(f.discard(buffer.read(&f, index)));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if index < 2 {
                Ok(())
            } else {
                Err(CSafetyError::IndexOutOfBounds)
            },
        );
    }
}

#[test]
fn writing_one_element_and_restoring_again_does_not_initialize_its_sibling() {
    for index in [0, 1] {
        let mut f = Fixture::new(&[]);
        let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Int));
        let count = f.size(2);
        let mut body = buffer.prefix(&mut f, count);
        body.push(buffer.write(&f, 0, f.int(7)));
        body.push(f.discard(restore(&f, &buffer.descriptor, f.read(&buffer.raw))));
        body.push(f.discard(buffer.read(&f, index)));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if index == 0 {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}

#[test]
fn base_and_element_zero_release_but_other_interior_addresses_do_not() {
    for index in [None, Some(0), Some(1)] {
        let mut f = Fixture::new(&[]);
        let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Int));
        let count = f.size(2);
        let mut body = buffer.prefix(&mut f, count);
        let pointer = index.map_or_else(
            || f.read(&buffer.data),
            |index| address(&f, buffer.place(&f, f.size(index))),
        );
        body.push(release(&f, erase(&f, pointer)));
        check(
            &f,
            body,
            if index == Some(1) {
                Err(CSafetyError::InvalidAllocationRelease)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn saved_interior_alias_expires_when_the_original_buffer_is_released() {
    let mut f = Fixture::new(&[]);
    let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Int));
    let alias = local(
        &mut f,
        pointer_type(CObjectType::scalar(CScalarType::Int)),
        "saved",
    );
    let count = f.size(2);
    let mut body = buffer.prefix(&mut f, count);
    body.push(buffer.write(&f, 1, f.int(9)));
    body.push(f.declare(&alias, address(&f, buffer.place(&f, f.size(1)))));
    body.push(buffer.release(&f));
    body.push(f.discard(pointed_read(&f, f.read(&alias))));
    check(&f, body, Err(CSafetyError::ExpiredStorage));
}

#[test]
fn nested_fixed_arrays_keep_their_own_bounds_inside_dynamic_elements() {
    for index in [1, 2] {
        let mut f = Fixture::new(&[]);
        let element = CObjectType::array(
            CObjectType::scalar(CScalarType::Int),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let buffer = Buffer::new(&mut f, element);
        let count = f.size(3);
        let mut body = buffer.prefix(&mut f, count);
        let place = f
            .values()
            .index(
                CIndexBase::Array(Box::new(buffer.place(&f, f.size(2)))),
                f.size(index),
            )
            .unwrap();
        body.push(f.ast().assign(place.clone(), f.int(5)).unwrap());
        body.push(f.discard(f.values().read(place).unwrap()));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if index == 1 {
                Ok(())
            } else {
                Err(CSafetyError::IndexOutOfBounds)
            },
        );
    }
}

#[test]
fn buffer_element_numeric_reads_keep_written_overflow_history() {
    for wrapped in [false, true] {
        let mut f = Fixture::new(&[]);
        let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Size));
        let second = raw(&mut f, "second");
        let count = f.size(2);
        let mut body = buffer.prefix(&mut f, count);
        let value = if wrapped {
            f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(3))
        } else {
            f.size(2)
        };
        body.push(buffer.write(&f, 1, value));
        body.push(f.declare(&second, allocate(&f, buffer.read(&f, 1))));
        body.push(release(&f, f.read(&second)));
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
fn buffer_initialization_is_a_must_fact_at_branch_joins() {
    for both in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Bool]);
        let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Int));
        let count = f.size(2);
        let mut body = buffer.prefix(&mut f, count);
        let write = buffer.write(&f, 0, f.int(1));
        let branch = f.branch(
            f.input(0),
            vec![write.clone()],
            if both { vec![write] } else { vec![] },
        );
        body.push(branch);
        body.push(f.discard(buffer.read(&f, 0)));
        body.push(buffer.release(&f));
        check(
            &f,
            body,
            if both {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}
