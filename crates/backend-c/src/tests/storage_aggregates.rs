//! Aggregate copies retain pointer origins and the exact active payload.
use super::{
    contextual_reconstruction::key, index_extent_fixture as arrays, numeric_fixture::Fixture,
    storage_fixture::*, *,
};

#[test]
fn zero_and_explicit_null_join_recursively_in_record_union_and_array_fields() {
    for union in [false, true] {
        for array in [false, true] {
            let mut f = Fixture::new(&[CScalarType::Bool]);
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
            let pointer = pointer_type(CObjectType::scalar(CScalarType::Int));
            let field_type = if array {
                CObjectType::array(pointer.clone(), CArrayLength::new(2).unwrap()).unwrap()
            } else {
                pointer.clone()
            };
            let member = f
                .registry
                .register_member(&owner, key("pointer"), field_type)
                .unwrap();
            f.registry
                .define_aggregate(&owner, vec![member.clone()])
                .unwrap();
            let value = local(&mut f, ty, "value");
            let field = f
                .values()
                .member(f.values().local(value.clone()).unwrap(), member)
                .unwrap();
            let field = if array {
                f.values()
                    .index(CIndexBase::Array(Box::new(field)), f.size(1))
                    .unwrap()
            } else {
                field
            };
            let null = f
                .values()
                .literal(CLiteral::NullPointer(CNullPointer::new(pointer).unwrap()))
                .unwrap();
            let assignment = f.ast().assign(field.clone(), null).unwrap();
            let branch = f.branch(f.input(0), vec![assignment], vec![]);
            let source = aggregate_source(
                &f,
                owner,
                vec![
                    arrays::declare(&f, &value),
                    branch,
                    f.discard(f.read(&value)),
                    f.discard(f.values().read(field).unwrap()),
                ],
            );
            assert_eq!(
                f.registry.check_storage_paths(&[source]),
                Ok(()),
                "union={union}, array={array}"
            );
        }
    }
}

#[test]
fn union_assignment_copies_active_member_without_activating_other_payloads() {
    for read_member in 0..2 {
        let mut f = Fixture::new(&[]);
        let tag = f.registry.declare_union(&f.file, key("Payload")).unwrap();
        let owner = CAggregateRef::Union(tag.clone());
        let members = ["first", "second"].map(|name| {
            f.registry
                .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Int))
                .unwrap()
        });
        f.registry
            .define_aggregate(&owner, members.to_vec())
            .unwrap();
        let a = local(&mut f, CObjectType::union(tag.clone()), "a");
        let b = local(&mut f, CObjectType::union(tag), "b");
        let selected = f
            .values()
            .member(
                f.values().local(b.clone()).unwrap(),
                members[read_member].clone(),
            )
            .unwrap();
        let source = aggregate_source(
            &f,
            owner,
            vec![
                arrays::declare(&f, &a),
                f.declare(&b, f.read(&a)),
                f.discard(f.values().read(selected).unwrap()),
            ],
        );
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            if read_member == 0 {
                Ok(())
            } else {
                Err(CSafetyError::InactiveUnionMember)
            }
        );
    }
}

#[test]
fn union_join_preserves_only_common_active_member_and_initialization() {
    for same in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Bool]);
        let tag = f.registry.declare_union(&f.file, key("Payload")).unwrap();
        let owner = CAggregateRef::Union(tag.clone());
        let members = ["first", "second"].map(|name| {
            f.registry
                .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Int))
                .unwrap()
        });
        f.registry
            .define_aggregate(&owner, members.to_vec())
            .unwrap();
        let value = local(&mut f, CObjectType::union(tag), "value");
        let selected = |i: usize| {
            f.values()
                .member(f.values().local(value.clone()).unwrap(), members[i].clone())
                .unwrap()
        };
        let left = f.ast().assign(selected(0), f.int(1)).unwrap();
        let right = f
            .ast()
            .assign(selected(usize::from(!same)), f.int(1))
            .unwrap();
        let read = f.discard(f.values().read(selected(0)).unwrap());
        let branch = f.branch(f.input(0), vec![left], vec![right]);
        let source = aggregate_source(&f, owner, vec![arrays::declare(&f, &value), branch, read]);
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            if same {
                Ok(())
            } else {
                Err(CSafetyError::InactiveUnionMember)
            }
        );
    }
}

#[test]
fn copied_record_pointer_is_not_rebased_to_a_same_typed_destination_record() {
    for expires in [false, true] {
        let mut f = Fixture::new(&[]);
        let tag = f.registry.declare_struct(&f.file, key("Record")).unwrap();
        let owner = CAggregateRef::Struct(tag.clone());
        let data = f
            .registry
            .register_member(&owner, key("data"), CObjectType::scalar(CScalarType::Int))
            .unwrap();
        let pointer = f
            .registry
            .register_member(&owner, key("pointer"), pointer_type(data.ty().clone()))
            .unwrap();
        f.registry
            .define_aggregate(&owner, vec![data.clone(), pointer.clone()])
            .unwrap();
        let inner = f
            .registry
            .register_scope(&f.function, Some(&f.scope), key("inner"))
            .unwrap();
        let source_scope = if expires { &inner } else { &f.scope };
        let source = f
            .registry
            .register_local(
                source_scope,
                key("source"),
                CObjectType::structure(tag.clone()),
            )
            .unwrap();
        let copied = local(&mut f, CObjectType::structure(tag), "copied");
        let field = |local: &CLocalRef, member: &CMemberRef| {
            f.values()
                .member(f.values().local(local.clone()).unwrap(), member.clone())
                .unwrap()
        };
        let preparation = vec![
            arrays::declare(&f, &source),
            f.ast()
                .assign(field(&source, &pointer), address(&f, field(&source, &data)))
                .unwrap(),
            f.ast()
                .assign(f.values().local(copied.clone()).unwrap(), f.read(&source))
                .unwrap(),
        ];
        let mut body = vec![arrays::declare(&f, &copied)];
        if expires {
            body.push(
                f.ast()
                    .nested_block(f.ast().block(inner, preparation).unwrap())
                    .unwrap(),
            );
        } else {
            body.extend(preparation);
            body.push(
                f.ast()
                    .nested_block(f.ast().block(inner, vec![]).unwrap())
                    .unwrap(),
            );
        }
        body.push(f.discard(pointed_read(
            &f,
            f.values().read(field(&copied, &pointer)).unwrap(),
        )));
        let source = aggregate_source(&f, owner, body);
        assert_eq!(
            f.registry.check_storage_paths(&[source]),
            if expires {
                Err(CSafetyError::ExpiredStorage)
            } else {
                Ok(())
            }
        );
    }
}

#[test]
fn automatic_pointer_nested_in_record_array_or_union_cannot_be_stored_globally() {
    for array in [false, true] {
        for union in [false, true] {
            let mut f = Fixture::new(&[]);
            let value = f.local(CScalarType::Int, "value");
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
            let pointer = pointer_type(value.ty().clone());
            let field_type = if array {
                CObjectType::array(pointer, CArrayLength::new(2).unwrap()).unwrap()
            } else {
                pointer
            };
            let member = f
                .registry
                .register_member(&owner, key("pointer"), field_type)
                .unwrap();
            f.registry
                .define_aggregate(&owner, vec![member.clone()])
                .unwrap();
            let container = local(&mut f, ty.clone(), "container");
            let output = f
                .registry
                .register_object(&f.file, key("output"), ty)
                .unwrap();
            let field = f
                .values()
                .member(f.values().local(container.clone()).unwrap(), member)
                .unwrap();
            let field = if array {
                f.values()
                    .index(CIndexBase::Array(Box::new(field)), f.size(1))
                    .unwrap()
            } else {
                field
            };
            let mut source = aggregate_source(
                &f,
                owner,
                vec![
                    f.declare(&value, f.int(1)),
                    arrays::declare(&f, &container),
                    f.ast()
                        .assign(field, address(&f, f.values().local(value).unwrap()))
                        .unwrap(),
                    f.ast()
                        .assign(
                            f.values().global(output.clone()).unwrap(),
                            f.read(&container),
                        )
                        .unwrap(),
                ],
            );
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
            assert_eq!(
                f.registry.check_storage_paths(&[source]),
                Err(CSafetyError::AutomaticAddressEscape)
            );
        }
    }
}
