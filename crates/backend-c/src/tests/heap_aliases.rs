//! Allocation copies, interior addresses and separate heap roots stay distinct.
use super::{
    allocation_fixture::*, heap_fixture::*, numeric_fixture::Fixture, storage_fixture::*, *,
};

#[test]
fn release_invalidates_pointers_stored_in_another_heap_object() {
    for expired in [false, true] {
        let mut f = Fixture::new(&[]);
        let data = raw(&mut f, "data");
        let holder = raw(&mut f, "holder");
        let ty = CObjectType::scalar(CScalarType::Int);
        let a = descriptor(&mut f, "data_object", ty.clone());
        let b = descriptor(&mut f, "holder_object", pointer_type(ty.clone()));
        let pointer = local(&mut f, pointer_type(ty.clone()), "pointer");
        let slot = local(&mut f, pointer_type(pointer_type(ty)), "slot");
        let live_data = require_live(&mut f, &data, vec![]);
        let cleanup = vec![release(&f, f.read(&data))];
        let live_holder = require_live(&mut f, &holder, cleanup);
        let mut body = vec![
            f.declare(&data, allocate(&f, f.size(4))),
            live_data,
            f.declare(&holder, allocate(&f, f.size(8))),
            live_holder,
            f.declare(&pointer, restore(&f, &a, f.read(&data))),
            f.declare(&slot, restore(&f, &b, f.read(&holder))),
            f.ast().assign(pointee(&f, &pointer), f.int(7)).unwrap(),
            f.ast()
                .assign(pointee(&f, &slot), f.read(&pointer))
                .unwrap(),
        ];
        if expired {
            body.push(release(&f, f.read(&data)));
        }
        body.push(f.discard(pointed_read(&f, pointed_read(&f, f.read(&slot)))));
        if !expired {
            body.push(release(&f, f.read(&data)));
        }
        body.push(release(&f, f.read(&holder)));
        check(
            &f,
            body,
            if expired {
                Err(CSafetyError::ExpiredStorage)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn identical_layouts_from_separate_calls_do_not_share_initialization() {
    for initialize_second in [false, true] {
        let mut f = Fixture::new(&[]);
        let a = raw(&mut f, "a");
        let b = raw(&mut f, "b");
        let ty = CObjectType::scalar(CScalarType::Int);
        let object = descriptor(&mut f, "object", ty.clone());
        let pa = local(&mut f, pointer_type(ty.clone()), "pa");
        let pb = local(&mut f, pointer_type(ty), "pb");
        let live_a = require_live(&mut f, &a, vec![]);
        let cleanup = vec![release(&f, f.read(&a))];
        let live_b = require_live(&mut f, &b, cleanup);
        let mut body = vec![
            f.declare(&a, allocate(&f, f.size(4))),
            live_a,
            f.declare(&b, allocate(&f, f.size(4))),
            live_b,
            f.declare(&pa, restore(&f, &object, f.read(&a))),
            f.declare(&pb, restore(&f, &object, f.read(&b))),
            f.ast().assign(pointee(&f, &pa), f.int(7)).unwrap(),
        ];
        if initialize_second {
            body.push(f.ast().assign(pointee(&f, &pb), f.int(8)).unwrap());
        }
        body.extend([
            f.discard(pointed_read(&f, f.read(&pb))),
            release(&f, f.read(&a)),
            release(&f, f.read(&b)),
        ]);
        check(
            &f,
            body,
            if initialize_second {
                Ok(())
            } else {
                Err(CSafetyError::UninitializedStorage)
            },
        );
    }
}

#[test]
fn a_saved_array_element_address_expires_with_its_allocation() {
    for expired in [false, true] {
        let mut f = Fixture::new(&[]);
        let raw = raw(&mut f, "raw");
        let ty = CObjectType::scalar(CScalarType::Int);
        let array = CObjectType::array(ty.clone(), CArrayLength::new(2).unwrap()).unwrap();
        let object = descriptor(&mut f, "array", array.clone());
        let pointer = local(&mut f, pointer_type(array), "pointer");
        let saved = local(&mut f, pointer_type(ty), "saved");
        let live = require_live(&mut f, &raw, vec![]);
        let element = f
            .values()
            .index(
                CIndexBase::Array(Box::new(pointee(&f, &pointer))),
                f.size(1),
            )
            .unwrap();
        let mut body = vec![
            f.declare(&raw, allocate(&f, f.size(8))),
            live,
            f.declare(&pointer, restore(&f, &object, f.read(&raw))),
            f.ast().assign(element.clone(), f.int(7)).unwrap(),
            f.declare(&saved, address(&f, element)),
        ];
        if expired {
            body.push(release(&f, f.read(&raw)));
        }
        body.push(f.discard(pointed_read(&f, f.read(&saved))));
        if !expired {
            body.push(release(&f, f.read(&raw)));
        }
        check(
            &f,
            body,
            if expired {
                Err(CSafetyError::ExpiredStorage)
            } else {
                Ok(())
            },
        );
    }
}
