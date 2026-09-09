//! Same-sized and similarly named types do not authorize heap reinterpretation.
use super::{
    allocation_fixture::*, contextual_reconstruction::key, heap_fixture::*,
    numeric_fixture::Fixture, storage_fixture::*, *,
};

#[test]
fn distinct_nominal_objects_do_not_share_an_allocation_binding() {
    let mut f = Fixture::new(&[]);
    let tags = ["A", "B"].map(|name| f.registry.declare_struct(&f.file, key(name)).unwrap());
    let owners = tags.clone().map(CAggregateRef::Struct);
    for owner in &owners {
        let member = f
            .registry
            .register_member(owner, key("value"), CObjectType::scalar(CScalarType::Int))
            .unwrap();
        f.registry.define_aggregate(owner, vec![member]).unwrap();
    }
    let a = descriptor(&mut f, "object_a", CObjectType::structure(tags[0].clone()));
    let b = descriptor(&mut f, "object_b", CObjectType::structure(tags[1].clone()));
    let raw = raw(&mut f, "raw");
    let live = require_live(&mut f, &raw, vec![]);
    let mut source = f.source(vec![
        f.declare(&raw, allocate(&f, f.size(4))),
        live,
        f.discard(restore(&f, &a, f.read(&raw))),
        f.discard(restore(&f, &b, f.read(&raw))),
        release(&f, f.read(&raw)),
    ]);
    for owner in owners {
        source.items.insert(
            0,
            CFileItem::Declaration(
                CDeclarations::new(&f.registry, f.file.clone())
                    .unwrap()
                    .aggregate(owner)
                    .unwrap(),
            ),
        );
    }
    assert_eq!(
        f.registry.check_storage_paths(&[source]),
        Err(CSafetyError::StorageTypeMismatch)
    );
}

#[test]
fn pointer_object_restore_preserves_deeper_qualification() {
    for qualified in [false, true] {
        let mut f = Fixture::new(&[]);
        let scalar = CObjectType::scalar(CScalarType::Int);
        let a = descriptor(&mut f, "object_a", pointer_type(scalar.clone()));
        let target = scalar
            .with_constness(if qualified {
                CConstness::Const
            } else {
                CConstness::Unqualified
            })
            .unwrap();
        let b = descriptor(&mut f, "object_b", pointer_type(target));
        let raw = raw(&mut f, "raw");
        let live = require_live(&mut f, &raw, vec![]);
        check(
            &f,
            vec![
                f.declare(&raw, allocate(&f, f.size(8))),
                live,
                f.discard(restore(&f, &a, f.read(&raw))),
                f.discard(restore(&f, &b, f.read(&raw))),
                release(&f, f.read(&raw)),
            ],
            if qualified {
                Err(CSafetyError::StorageTypeMismatch)
            } else {
                Ok(())
            },
        );
    }
}

#[test]
fn typed_heap_addresses_cannot_escape_through_global_slots() {
    for interior in [false, true] {
        let mut f = Fixture::new(&[]);
        let scalar = CObjectType::scalar(CScalarType::Int);
        let array = CObjectType::array(scalar.clone(), CArrayLength::new(2).unwrap()).unwrap();
        let object = descriptor(&mut f, "object", array.clone());
        let raw = raw(&mut f, "raw");
        let pointer = local(&mut f, pointer_type(array), "pointer");
        let output_type = if interior {
            pointer_type(scalar)
        } else {
            pointer.ty().clone()
        };
        let output = f
            .registry
            .register_object(&f.file, key("output"), output_type)
            .unwrap();
        let address = if interior {
            address(
                &f,
                f.values()
                    .index(
                        CIndexBase::Array(Box::new(pointee(&f, &pointer))),
                        f.size(1),
                    )
                    .unwrap(),
            )
        } else {
            f.read(&pointer)
        };
        let live = require_live(&mut f, &raw, vec![]);
        let mut source = f.source(vec![
            f.declare(&raw, allocate(&f, f.size(8))),
            live,
            f.declare(&pointer, restore(&f, &object, f.read(&raw))),
            f.ast()
                .assign(f.values().global(output.clone()).unwrap(), address)
                .unwrap(),
            release(&f, f.read(&raw)),
        ]);
        source.items.insert(
            0,
            CFileItem::Definition(
                CDeclarations::new(&f.registry, f.file.clone())
                    .unwrap()
                    .object_definition(
                        output.clone(),
                        CLinkage::Internal,
                        f.values().zero_initializer(output.ty().clone()).unwrap(),
                    )
                    .unwrap(),
            ),
        );
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            Err(CSafetyError::UnprovedAllocation)
        );
    }
}
