//! Independent recursion-arm controls for allocation alias expiry and escape.
use super::{
    allocation_fixture::*,
    contextual_reconstruction::key,
    numeric_fixture::Fixture,
    storage_fixture::{aggregate_source, local},
    *,
};

fn container(
    f: &mut Fixture,
    pointer: &CLocalRef,
    union: bool,
    array: bool,
) -> (CAggregateRef, CLocalRef, CPlace) {
    let owner = if union {
        CAggregateRef::Union(f.registry.declare_union(&f.file, key("Container")).unwrap())
    } else {
        CAggregateRef::Struct(
            f.registry
                .declare_struct(&f.file, key("Container"))
                .unwrap(),
        )
    };
    let ty = match &owner {
        CAggregateRef::Struct(tag) => CObjectType::structure(tag.clone()),
        CAggregateRef::Union(tag) => CObjectType::union(tag.clone()),
    };
    let element = pointer.ty().clone();
    let field_type = if array {
        CObjectType::array(element, CArrayLength::new(2).unwrap()).unwrap()
    } else {
        element
    };
    let field = f
        .registry
        .register_member(&owner, key("pointer"), field_type)
        .unwrap();
    f.registry
        .define_aggregate(&owner, vec![field.clone()])
        .unwrap();
    let value = local(f, ty, "container");
    let place = f
        .values()
        .member(f.values().local(value.clone()).unwrap(), field)
        .unwrap();
    let place = if array {
        f.values()
            .index(CIndexBase::Array(Box::new(place)), f.size(1))
            .unwrap()
    } else {
        place
    };
    (owner, value, place)
}
fn preparation(
    f: &Fixture,
    pointer: &CLocalRef,
    value: &CLocalRef,
    place: CPlace,
) -> Vec<CStatement> {
    vec![
        f.declare(pointer, allocate(f, f.size(8))),
        f.ast()
            .declare(
                value.clone(),
                Some(f.values().zero_initializer(value.ty().clone()).unwrap()),
            )
            .unwrap(),
        f.ast().assign(place, f.read(pointer)).unwrap(),
    ]
}

#[test]
fn release_expires_all_record_union_and_array_copies_but_not_before_release() {
    for union in [false, true] {
        for array in [false, true] {
            for after in [false, true] {
                let mut f = Fixture::new(&[]);
                let pointer = raw(&mut f, "pointer");
                let (owner, value, place) = container(&mut f, &pointer, union, array);
                let mut body = preparation(&f, &pointer, &value, place);
                let read = f.discard(f.read(&value));
                if !after {
                    body.push(read.clone());
                }
                body.push(release(&f, f.read(&pointer)));
                if after {
                    body.push(read);
                }
                let source = aggregate_source(&f, owner, body);
                f.registry
                    .check_numeric_flow(std::slice::from_ref(&source))
                    .unwrap();
                assert_eq!(
                    f.registry.check_storage_paths(&[source]),
                    if after {
                        Err(CSafetyError::ExpiredStorage)
                    } else {
                        Ok(())
                    },
                    "union={union}, array={array}, after={after}"
                );
            }
        }
    }
}

#[test]
fn no_aggregate_recursion_arm_can_smuggle_a_raw_allocation_into_a_global() {
    for union in [false, true] {
        for array in [false, true] {
            let mut f = Fixture::new(&[]);
            let pointer = raw(&mut f, "pointer");
            let (owner, value, place) = container(&mut f, &pointer, union, array);
            let output = f
                .registry
                .register_object(&f.file, key("output"), value.ty().clone())
                .unwrap();
            let mut body = preparation(&f, &pointer, &value, place);
            body.push(
                f.ast()
                    .assign(f.values().global(output.clone()).unwrap(), f.read(&value))
                    .unwrap(),
            );
            body.push(release(&f, f.read(&pointer)));
            let mut source = aggregate_source(&f, owner, body);
            source.items.push(CFileItem::Definition(
                CDeclarations::new(&f.registry, f.file.clone())
                    .unwrap()
                    .object_definition(
                        output.clone(),
                        CLinkage::Internal,
                        f.values().zero_initializer(output.ty().clone()).unwrap(),
                    )
                    .unwrap(),
            ));
            f.registry
                .check_numeric_flow(std::slice::from_ref(&source))
                .unwrap();
            assert!(
                f.registry.check_storage_paths(&[source]).is_err(),
                "union={union}, array={array}"
            );
        }
    }
}
