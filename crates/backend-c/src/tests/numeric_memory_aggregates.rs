//! Snapshots and weak updates do not turn stored arithmetic loss into clean data.
use super::{
    contextual_reconstruction::key, numeric_fixture::Fixture, numeric_memory_fixture::*,
    storage_fixture::*, *,
};

#[test]
fn heap_aggregate_copies_preserve_each_field_and_its_loss_history() {
    for union in [false, true] {
        for lossy in [false, true] {
            let mut f = Fixture::new(&[]);
            let owner = if union {
                CAggregateRef::Union(f.registry.declare_union(&f.file, key("Payload")).unwrap())
            } else {
                CAggregateRef::Struct(f.registry.declare_struct(&f.file, key("Payload")).unwrap())
            };
            let ty = match &owner {
                CAggregateRef::Struct(tag) => CObjectType::structure(tag.clone()),
                CAggregateRef::Union(tag) => CObjectType::union(tag.clone()),
            };
            let members = ["bytes", "other"].map(|name| {
                f.registry
                    .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Size))
                    .unwrap()
            });
            f.registry
                .define_aggregate(&owner, members.to_vec())
                .unwrap();
            let input = local(&mut f, ty.clone(), "input");
            let output = local(&mut f, ty.clone(), "output");
            let heap = Heap::new(&mut f, "heap", ty);
            let mut body = heap.prefix(&mut f, 16, vec![]);
            let bytes = if lossy { wrapped(&f) } else { f.size(2) };
            let bytes = f.values().expression_initializer(bytes).unwrap();
            let initializer = match &owner {
                CAggregateRef::Union(tag) => f
                    .values()
                    .union_initializer(tag.clone(), members[0].clone(), bytes)
                    .unwrap(),
                CAggregateRef::Struct(tag) => f
                    .values()
                    .struct_initializer(
                        tag.clone(),
                        vec![
                            (members[0].clone(), bytes),
                            (
                                members[1].clone(),
                                f.values().expression_initializer(f.size(3)).unwrap(),
                            ),
                        ],
                    )
                    .unwrap(),
            };
            body.extend([
                f.ast().declare(input.clone(), Some(initializer)).unwrap(),
                heap.write(&f, f.read(&input)),
                f.declare(&output, heap.read(&f)),
            ]);
            let field = f
                .values()
                .member(f.values().local(output).unwrap(), members[0].clone())
                .unwrap();
            let bytes = f.values().read(field).unwrap();
            body.extend(consume(&mut f, bytes));
            body.push(heap.release(&f));
            assert_eq!(
                f.registry
                    .check_storage_paths(&[aggregate_source(&f, owner, body)]),
                if lossy {
                    Err(CSafetyError::UnprovedSizeArithmetic)
                } else {
                    Ok(())
                }
            );
        }
    }
}

#[test]
fn writing_a_record_sibling_preserves_the_other_fields_known_value() {
    let mut f = Fixture::new(&[]);
    let tag = f.registry.declare_struct(&f.file, key("Payload")).unwrap();
    let owner = CAggregateRef::Struct(tag.clone());
    let members = ["bytes", "other"].map(|name| {
        f.registry
            .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Size))
            .unwrap()
    });
    f.registry
        .define_aggregate(&owner, members.to_vec())
        .unwrap();
    let heap = Heap::new(&mut f, "heap", CObjectType::structure(tag));
    let mut body = heap.prefix(&mut f, 16, vec![]);
    for (i, member) in members.iter().enumerate() {
        body.push(
            f.ast()
                .assign(
                    f.values().member(heap.place(&f), member.clone()).unwrap(),
                    f.size(i as u64 + 2),
                )
                .unwrap(),
        );
    }
    let bytes = f
        .values()
        .read(
            f.values()
                .member(heap.place(&f), members[0].clone())
                .unwrap(),
        )
        .unwrap();
    body.extend(consume(&mut f, bytes));
    body.push(heap.release(&f));
    assert_eq!(
        f.registry
            .check_storage_paths(&[aggregate_source(&f, owner, body)]),
        Ok(())
    );
}

#[test]
fn ambiguous_heap_array_writes_cannot_leave_stale_exact_elements() {
    for weak in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let ty = CObjectType::array(
            CObjectType::scalar(CScalarType::Size),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let heap = Heap::new(&mut f, "heap", ty);
        let mut body = heap.prefix(&mut f, 16, vec![]);
        let element = |index| {
            f.values()
                .index(CIndexBase::Array(Box::new(heap.place(&f))), index)
                .unwrap()
        };
        let first = element(f.size(0));
        let second = element(f.size(1));
        let destination = element(if weak { f.input(0) } else { f.size(1) });
        body.extend([
            f.ast().assign(first.clone(), f.size(2)).unwrap(),
            f.ast().assign(second, f.size(3)).unwrap(),
        ]);
        let guard = f.compare(CBinaryOperator::Less, f.input(0), f.size(2));
        let cleanup = vec![heap.release(&f)];
        body.push(assume(&mut f, guard, cleanup));
        body.push(f.ast().assign(destination, wrapped(&f)).unwrap());
        let bytes = f.values().read(first).unwrap();
        body.extend(consume(&mut f, bytes));
        body.push(heap.release(&f));
        check(
            &f,
            body,
            if weak {
                Err(CSafetyError::UnprovedSizeArithmetic)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn an_ambiguous_read_cannot_drop_the_wrapped_exact_element() {
    let mut f = Fixture::new(&[CScalarType::Size]);
    let ty = CObjectType::array(
        CObjectType::scalar(CScalarType::Size),
        CArrayLength::new(2).unwrap(),
    )
    .unwrap();
    let heap = Heap::new(&mut f, "heap", ty);
    let mut body = heap.prefix(&mut f, 16, vec![]);
    for i in 0..2 {
        let element = f
            .values()
            .index(CIndexBase::Array(Box::new(heap.place(&f))), f.size(i))
            .unwrap();
        body.push(
            f.ast()
                .assign(element, if i == 0 { wrapped(&f) } else { f.size(2) })
                .unwrap(),
        );
    }
    let guard = f.compare(CBinaryOperator::Less, f.input(0), f.size(2));
    let cleanup = vec![heap.release(&f)];
    body.push(assume(&mut f, guard, cleanup));
    let element = f
        .values()
        .index(CIndexBase::Array(Box::new(heap.place(&f))), f.input(0))
        .unwrap();
    let bytes = f.values().read(element).unwrap();
    body.extend(consume(&mut f, bytes));
    body.push(heap.release(&f));
    check(&f, body, Err(CSafetyError::UnprovedSizeArithmetic));
}
