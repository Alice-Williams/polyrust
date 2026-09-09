//! Fixed heap projections use actual member identity, extent and active payload.
use super::{
    allocation_fixture::*, contextual_reconstruction::key, heap_fixture::*,
    numeric_fixture::Fixture, storage_fixture::*, *,
};

#[test]
fn record_and_union_reads_require_the_selected_initialized_member() {
    for union in [false, true] {
        for selected in 0..2 {
            let mut f = Fixture::new(&[]);
            let owner = if union {
                CAggregateRef::Union(f.registry.declare_union(&f.file, key("Object")).unwrap())
            } else {
                CAggregateRef::Struct(f.registry.declare_struct(&f.file, key("Object")).unwrap())
            };
            let ty = match &owner {
                CAggregateRef::Struct(tag) => CObjectType::structure(tag.clone()),
                CAggregateRef::Union(tag) => CObjectType::union(tag.clone()),
            };
            let members = ["first", "second"].map(|name| {
                f.registry
                    .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Int))
                    .unwrap()
            });
            f.registry
                .define_aggregate(&owner, members.to_vec())
                .unwrap();
            let raw = raw(&mut f, "raw");
            let descriptor = descriptor(&mut f, "object", ty.clone());
            let typed = local(&mut f, pointer_type(ty), "typed");
            let field = |i: usize| {
                f.values()
                    .member(pointee(&f, &typed), members[i].clone())
                    .unwrap()
            };
            let actions = vec![
                f.ast().assign(field(0), f.int(7)).unwrap(),
                f.discard(f.values().read(field(selected)).unwrap()),
            ];
            let body = guarded(&mut f, &raw, &typed, &descriptor, 8, actions);
            let source = aggregate_source(&f, owner, body);
            assert_eq!(
                f.registry.check_storage_paths(&[source]),
                if selected == 0 {
                    Ok(())
                } else if union {
                    Err(CSafetyError::InactiveUnionMember)
                } else {
                    Err(CSafetyError::UninitializedStorage)
                }
            );
        }
    }
}

#[test]
fn fixed_heap_array_paths_keep_extent_and_per_element_initialization() {
    for selected in 0..3 {
        let mut f = Fixture::new(&[]);
        let raw = raw(&mut f, "raw");
        let array = CObjectType::array(
            CObjectType::scalar(CScalarType::Int),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let descriptor = descriptor(&mut f, "object", array.clone());
        let typed = local(&mut f, pointer_type(array), "typed");
        let element = f
            .values()
            .index(CIndexBase::Array(Box::new(pointee(&f, &typed))), f.size(0))
            .unwrap();
        let base = address(&f, element.clone());
        let actions = vec![
            f.ast().assign(element, f.int(7)).unwrap(),
            f.discard(f.values().read(pointer_index(&f, base, selected)).unwrap()),
        ];
        let body = guarded(&mut f, &raw, &typed, &descriptor, 64, actions);
        check(
            &f,
            body,
            match selected {
                0 => Ok(()),
                1 => Err(CSafetyError::UninitializedStorage),
                _ => Err(CSafetyError::IndexOutOfBounds),
            },
        );
    }
}

#[test]
fn releasing_or_restoring_an_interior_pointer_cannot_claim_another_base() {
    for operation in 0..2 {
        let mut f = Fixture::new(&[]);
        let raw = raw(&mut f, "raw");
        let array = CObjectType::array(
            CObjectType::scalar(CScalarType::Int),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let descriptor = descriptor(&mut f, "object", array.clone());
        let scalar = super::heap_fixture::descriptor(
            &mut f,
            "scalar",
            CObjectType::scalar(CScalarType::Int),
        );
        let typed = local(&mut f, pointer_type(array), "typed");
        let element = f
            .values()
            .index(CIndexBase::Array(Box::new(pointee(&f, &typed))), f.size(0))
            .unwrap();
        let interior = erase(&f, address(&f, element));
        let action = if operation == 0 {
            release(&f, interior)
        } else {
            f.discard(restore(&f, &scalar, interior))
        };
        let body = guarded(&mut f, &raw, &typed, &descriptor, 8, vec![action]);
        check(
            &f,
            body,
            if operation == 0 {
                Err(CSafetyError::InvalidAllocationRelease)
            } else {
                Err(CSafetyError::UnprovedAllocation)
            },
        );
    }
}
