//! Actual array lengths and pre-operation index lineage, not matching spellings.
use super::{contextual_reconstruction::key, index_extent_fixture::*, numeric_fixture::Fixture, *};

#[test]
fn exact_bounds_apply_to_reads_writes_and_address_only_places() {
    for length in [1, 3] {
        for index in [-1, 0, length - 1, length, length + 1] {
            for access in 0..3 {
                let mut f = Fixture::new(&[]);
                let local = array(&mut f, length as u64, "array");
                let selected = place(&f, &local, f.int(index));
                let operation = match access {
                    0 => f.discard(f.values().read(selected).unwrap()),
                    1 => f.ast().assign(selected, f.int(7)).unwrap(),
                    _ => f.discard(f.values().address_of(selected).unwrap()),
                };
                check(
                    &f,
                    vec![declare(&f, &local), operation],
                    if index >= 0 && index < length {
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
fn unknown_huge_and_wrapped_indices_are_not_extent_proofs() {
    for kind in 0..3 {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let local = array(&mut f, 2, "array");
        let index = match kind {
            0 => f.input(0),
            1 => f.size(u64::MAX),
            _ => f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(1)),
        };
        check(
            &f,
            vec![declare(&f, &local), f.discard(read(&f, &local, index))],
            Err(if kind == 2 {
                CSafetyError::UnprovedSizeArithmetic
            } else {
                CSafetyError::IndexOutOfBounds
            }),
        );
    }
}

#[test]
fn each_nested_dimension_uses_its_own_actual_length() {
    for (outer, inner, accepted) in [(1, 2, true), (2, 1, false), (1, 3, false)] {
        let mut f = Fixture::new(&[]);
        let row = CObjectType::array(
            CObjectType::scalar(CScalarType::Int),
            CArrayLength::new(3).unwrap(),
        )
        .unwrap();
        let ty = CObjectType::array(row, CArrayLength::new(2).unwrap()).unwrap();
        let local = f
            .registry
            .register_local(&f.scope, key("matrix"), ty)
            .unwrap();
        let row = place(&f, &local, f.int(outer));
        let selected = f
            .values()
            .index(CIndexBase::Array(Box::new(row)), f.int(inner))
            .unwrap();
        check(
            &f,
            vec![
                declare(&f, &local),
                f.discard(f.values().read(selected).unwrap()),
            ],
            if accepted {
                Ok(())
            } else {
                Err(CSafetyError::IndexOutOfBounds)
            },
        );
    }
}

#[test]
fn pointer_indexing_cannot_borrow_a_fixed_array_extent_without_provenance() {
    let mut f = Fixture::new(&[]);
    let local = array(&mut f, 2, "array");
    let pointer = f.values().address_of(place(&f, &local, f.size(0))).unwrap();
    let selected = f
        .values()
        .index(CIndexBase::Pointer(Box::new(pointer)), f.size(0))
        .unwrap();
    check(
        &f,
        vec![
            declare(&f, &local),
            f.discard(f.values().read(selected).unwrap()),
        ],
        Err(CSafetyError::UnprovedPointerExtent),
    );
}

#[test]
fn address_only_indexing_does_not_require_initializing_the_element() {
    let mut f = Fixture::new(&[]);
    let local = array(&mut f, 2, "array");
    let address = f.values().address_of(place(&f, &local, f.size(1))).unwrap();
    check(
        &f,
        vec![f.ast().declare(local, None).unwrap(), f.discard(address)],
        Ok(()),
    );
}

#[test]
fn member_arrays_use_the_selected_member_type_not_the_containing_record() {
    for selected in [0, 1] {
        let mut f = Fixture::new(&[]);
        let record = f.registry.declare_struct(&f.file, key("Record")).unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        let mut members = vec![];
        for (name, length) in [("short_array", 2), ("long_array", 3)] {
            let ty = CObjectType::array(
                CObjectType::scalar(CScalarType::Int),
                CArrayLength::new(length).unwrap(),
            )
            .unwrap();
            members.push(f.registry.register_member(&owner, key(name), ty).unwrap());
        }
        f.registry
            .define_aggregate(&owner, members.clone())
            .unwrap();
        let local = f
            .registry
            .register_local(&f.scope, key("record"), CObjectType::structure(record))
            .unwrap();
        let member = f
            .values()
            .member(
                f.values().local(local.clone()).unwrap(),
                members[selected].clone(),
            )
            .unwrap();
        let element = f
            .values()
            .index(CIndexBase::Array(Box::new(member)), f.size(2))
            .unwrap();
        let mut source = f.source(vec![
            declare(&f, &local),
            f.discard(f.values().read(element).unwrap()),
        ]);
        source.items.insert(
            0,
            CFileItem::Declaration(
                CDeclarations::new(&f.registry, f.file.clone())
                    .unwrap()
                    .aggregate(owner)
                    .unwrap(),
            ),
        );
        assert_eq!(
            f.registry.check_index_extents(&[source]),
            if selected == 0 {
                Err(CSafetyError::IndexOutOfBounds)
            } else {
                Ok(())
            }
        );
    }
}
