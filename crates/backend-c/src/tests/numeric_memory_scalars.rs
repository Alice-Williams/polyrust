//! Exact memory identities preserve numeric values, conversions and loss history.
use super::{
    heap_fixture::*, numeric_fixture::Fixture, numeric_memory_fixture::*, storage_fixture::*, *,
};

#[test]
fn automatic_and_heap_aliases_share_the_actual_numeric_write() {
    for heap in [false, true] {
        for lossy in [false, true] {
            let mut f = Fixture::new(&[]);
            let ty = CObjectType::scalar(CScalarType::Size);
            let mut body = vec![];
            let storage = heap.then(|| Heap::new(&mut f, "heap", ty.clone()));
            let (place, alias) = if let Some(storage) = &storage {
                body.extend(storage.prefix(&mut f, 8, vec![]));
                let alias = local(&mut f, pointer_type(ty), "alias");
                body.push(f.declare(&alias, f.read(&storage.typed)));
                (storage.place(&f), alias)
            } else {
                let value = f.local(CScalarType::Size, "value");
                let alias = local(&mut f, pointer_type(ty), "alias");
                body.push(f.ast().declare(value.clone(), None).unwrap());
                let place = f.values().local(value).unwrap();
                body.push(f.declare(&alias, address(&f, place.clone())));
                (place, alias)
            };
            body.push(
                f.ast()
                    .assign(place, if lossy { wrapped(&f) } else { f.size(2) })
                    .unwrap(),
            );
            let value = pointed_read(&f, f.read(&alias));
            body.extend(consume(&mut f, value));
            if let Some(storage) = storage {
                body.push(storage.release(&f));
            }
            check(
                &f,
                body,
                if lossy {
                    Err(CSafetyError::UnprovedSizeArithmetic)
                } else {
                    Ok(())
                },
            );
        }
    }
}

#[test]
fn compatible_heap_scalar_spellings_keep_domains_through_join_and_arithmetic() {
    for (first, second) in [
        (CScalarType::Size, CScalarType::U64),
        (CScalarType::Int, CScalarType::I32),
    ] {
        let mut f = Fixture::new(&[CScalarType::Bool]);
        let heap = Heap::new(&mut f, "heap", CObjectType::scalar(first));
        let other = descriptor(&mut f, "other", CObjectType::scalar(second));
        let alias = local(&mut f, pointer_type(other.object_type().clone()), "alias");
        let mut body = heap.prefix(&mut f, 8, vec![]);
        body.push(f.declare(&alias, restore(&f, &other, f.read(&heap.raw))));
        let convert = |value| {
            f.values()
                .numeric_conversion(second, f.size(value))
                .unwrap()
        };
        let yes = heap.write(&f, f.values().numeric_conversion(first, f.size(2)).unwrap());
        let no = f.ast().assign(pointee(&f, &alias), convert(3)).unwrap();
        let one = convert(1);
        body.push(f.branch(f.input(0), vec![yes], vec![no]));
        let value = f.binary(CBinaryOperator::Add, pointed_read(&f, f.read(&alias)), one);
        let bytes = f
            .values()
            .numeric_conversion(CScalarType::Size, value)
            .unwrap();
        body.extend(consume(&mut f, bytes));
        body.push(heap.release(&f));
        check(&f, body, Ok(()));
    }
}

#[test]
fn refining_an_indirect_value_never_erases_earlier_wrap_history() {
    for lossy in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let heap = Heap::new(&mut f, "heap", CObjectType::scalar(CScalarType::Size));
        let mut body = heap.prefix(&mut f, 8, vec![]);
        body.push(heap.write(&f, if lossy { wrapped(&f) } else { f.input(0) }));
        let guard = positive(&f, heap.read(&f));
        let cleanup = vec![heap.release(&f)];
        body.push(assume(&mut f, guard, cleanup));
        let bytes = heap.read(&f);
        body.extend(consume(&mut f, bytes));
        body.push(heap.release(&f));
        check(
            &f,
            body,
            if lossy {
                Err(CSafetyError::UnprovedSizeArithmetic)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn mutation_through_an_alias_invalidates_a_numeric_guard() {
    for mutate in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let heap = Heap::new(&mut f, "heap", CObjectType::scalar(CScalarType::Size));
        let alias = local(&mut f, heap.typed.ty().clone(), "alias");
        let mut body = heap.prefix(&mut f, 8, vec![]);
        body.extend([
            f.declare(&alias, f.read(&heap.typed)),
            heap.write(&f, f.input(0)),
        ]);
        let guard = f.compare(CBinaryOperator::Less, heap.read(&f), f.size(10));
        let cleanup = vec![heap.release(&f)];
        body.push(assume(&mut f, guard, cleanup));
        if mutate {
            body.push(
                f.ast()
                    .assign(pointee(&f, &alias), f.size(u64::MAX))
                    .unwrap(),
            );
        }
        let bytes = f.binary(CBinaryOperator::Add, heap.read(&f), f.size(1));
        body.extend(consume(&mut f, bytes));
        body.push(heap.release(&f));
        check(
            &f,
            body,
            if mutate {
                Err(CSafetyError::UnprovedSizeArithmetic)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn algebraic_guards_resolve_the_same_heap_terms_as_arithmetic() {
    for operation in [CBinaryOperator::Add, CBinaryOperator::Multiply] {
        for mutate in [false, true] {
            let mut f = Fixture::new(&[CScalarType::Size, CScalarType::Size]);
            let heap = Heap::new(&mut f, "heap", CObjectType::scalar(CScalarType::Size));
            let mut body = heap.prefix(&mut f, 8, vec![]);
            body.push(heap.write(&f, f.input(0)));
            let lower = positive(&f, heap.read(&f));
            let rhs = positive(&f, f.input(1));
            let cleanup = vec![heap.release(&f)];
            let both = f.boolean(f.binary(CBinaryOperator::LogicalAnd, lower, rhs));
            body.push(assume(&mut f, both, cleanup));
            let limit = f.binary(
                if operation == CBinaryOperator::Add {
                    CBinaryOperator::Subtract
                } else {
                    CBinaryOperator::Divide
                },
                f.size(u64::MAX),
                f.input(1),
            );
            let guard = f.compare(CBinaryOperator::LessEqual, heap.read(&f), limit);
            let cleanup = vec![heap.release(&f)];
            body.push(assume(&mut f, guard, cleanup));
            if mutate {
                body.push(heap.write(&f, f.size(u64::MAX)));
            }
            let bytes = f.binary(operation, heap.read(&f), f.input(1));
            body.extend(consume(&mut f, bytes));
            body.push(heap.release(&f));
            check(
                &f,
                body,
                if mutate {
                    Err(CSafetyError::UnprovedSizeArithmetic)
                } else {
                    Ok(())
                },
            );
        }
    }
}
