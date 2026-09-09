//! Actual allocations keep their byte snapshot and never share numeric history.
use super::{
    allocation_fixture::*, heap_fixture::*, numeric_fixture::Fixture, numeric_memory_fixture::*,
    storage_fixture::*, *,
};

#[test]
fn copied_numeric_values_survive_release_without_losing_their_history() {
    for lossy in [false, true] {
        let mut f = Fixture::new(&[]);
        let heap = Heap::new(&mut f, "heap", CObjectType::scalar(CScalarType::Size));
        let saved = f.local(CScalarType::Size, "saved");
        let mut body = heap.prefix(&mut f, 8, vec![]);
        body.extend([
            heap.write(&f, if lossy { wrapped(&f) } else { f.size(2) }),
            f.declare(&saved, heap.read(&f)),
            heap.release(&f),
        ]);
        let bytes = f.read(&saved);
        body.extend(consume(&mut f, bytes));
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
fn released_memory_cannot_supply_a_size_even_if_a_numeric_fact_was_cached() {
    let mut f = Fixture::new(&[]);
    let heap = Heap::new(&mut f, "heap", CObjectType::scalar(CScalarType::Size));
    let mut body = heap.prefix(&mut f, 8, vec![]);
    body.extend([
        heap.write(&f, f.size(2)),
        f.discard(heap.read(&f)),
        heap.release(&f),
    ]);
    let bytes = heap.read(&f);
    body.extend(consume(&mut f, bytes));
    check(&f, body, Err(CSafetyError::ExpiredStorage));
}

#[test]
fn a_second_same_layout_allocation_cannot_borrow_the_firsts_numeric_value() {
    for initialized in [false, true] {
        let mut f = Fixture::new(&[]);
        let ty = CObjectType::scalar(CScalarType::Size);
        let first = Heap::new(&mut f, "first", ty.clone());
        let second = Heap::new(&mut f, "second", ty);
        let mut body = first.prefix(&mut f, 8, vec![]);
        body.push(first.write(&f, f.size(2)));
        let cleanup = vec![first.release(&f)];
        body.extend(second.prefix(&mut f, 8, cleanup));
        if initialized {
            body.push(second.write(&f, f.size(3)));
        }
        let bytes = second.read(&f);
        body.extend(consume(&mut f, bytes));
        body.extend([second.release(&f), first.release(&f)]);
        check(
            &f,
            body,
            if initialized {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}

#[test]
fn changing_the_heap_size_source_cannot_resize_an_existing_request() {
    for original in [4, 8] {
        let mut f = Fixture::new(&[]);
        let heap = Heap::new(&mut f, "heap", CObjectType::scalar(CScalarType::Size));
        let next = raw(&mut f, "next");
        let target = descriptor(&mut f, "target", CObjectType::scalar(CScalarType::Size));
        let mut body = heap.prefix(&mut f, 8, vec![]);
        body.extend([
            heap.write(&f, f.size(original)),
            f.declare(&next, allocate(&f, heap.read(&f))),
        ]);
        let cleanup = vec![heap.release(&f)];
        body.push(require_live(&mut f, &next, cleanup));
        body.extend([
            heap.write(&f, f.size(8)),
            f.discard(restore(&f, &target, f.read(&next))),
            release(&f, f.read(&next)),
            heap.release(&f),
        ]);
        check(
            &f,
            body,
            if original == 8 {
                Ok(())
            } else {
                Err(CSafetyError::UnprovedAllocationSize)
            },
        );
    }
}

#[test]
fn heap_branch_joins_retain_a_loss_from_either_incoming_write() {
    for lossy in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Bool]);
        let heap = Heap::new(&mut f, "heap", CObjectType::scalar(CScalarType::Size));
        let mut body = heap.prefix(&mut f, 8, vec![]);
        let yes = heap.write(&f, f.size(2));
        let no = heap.write(&f, if lossy { wrapped(&f) } else { f.size(3) });
        body.push(f.branch(f.input(0), vec![yes], vec![no]));
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
fn exact_overwrite_replaces_history_instead_of_permanently_tainting_the_heap() {
    let mut f = Fixture::new(&[]);
    let heap = Heap::new(&mut f, "heap", CObjectType::scalar(CScalarType::Size));
    let mut body = heap.prefix(&mut f, 8, vec![]);
    body.extend([heap.write(&f, wrapped(&f)), heap.write(&f, f.size(2))]);
    let bytes = heap.read(&f);
    body.extend(consume(&mut f, bytes));
    body.push(heap.release(&f));
    check(&f, body, Ok(()));
}
