//! Range normalization must preserve disjoint fields, copies and ABI index aliases.
use super::{
    allocation_fixture::*, buffer_fixture::Buffer, contextual_reconstruction::key,
    numeric_fixture::Fixture, storage_fixture::*, *,
};

#[test]
fn narrowed_element_copies_preserve_disjoint_field_values_and_numeric_losses() {
    for wrapped in [false, true] {
        let mut f = Fixture::new(&[CScalarType::Size]);
        let tag = f.registry.declare_struct(&f.file, key("Element")).unwrap();
        let owner = CAggregateRef::Struct(tag.clone());
        let ty = CObjectType::structure(tag);
        let members = ["first", "second"].map(|name| {
            f.registry
                .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Size))
                .unwrap()
        });
        f.registry
            .define_aggregate(&owner, members.to_vec())
            .unwrap();
        let copy = local(&mut f, ty.clone(), "copy");
        let extra = raw(&mut f, "extra");
        let buffer = Buffer::new(&mut f, ty);
        let count = f.size(2);
        let mut body = buffer.prefix(&mut f, count);
        body.push(f.ast().declare(copy.clone(), None).unwrap());
        let condition = f.compare(
            CBinaryOperator::Less,
            f.input(0),
            f.read(buffer.count.local()),
        );
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        let value = if wrapped {
            f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(8))
        } else {
            f.size(7)
        };
        body.push(
            f.ast()
                .assign(
                    f.values()
                        .member(buffer.place(&f, f.input(0)), members[0].clone())
                        .unwrap(),
                    value,
                )
                .unwrap(),
        );
        body.push(
            f.ast()
                .assign(
                    f.values()
                        .member(buffer.place(&f, f.input(0)), members[1].clone())
                        .unwrap(),
                    f.size(9),
                )
                .unwrap(),
        );
        let condition = f.compare(CBinaryOperator::Less, f.input(0), f.size(1));
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        body.push(
            f.ast()
                .assign(
                    f.values()
                        .member(buffer.place(&f, f.input(0)), members[1].clone())
                        .unwrap(),
                    f.size(11),
                )
                .unwrap(),
        );
        body.push(
            f.ast()
                .assign(
                    f.values().local(copy.clone()).unwrap(),
                    f.values().read(buffer.place(&f, f.input(0))).unwrap(),
                )
                .unwrap(),
        );
        let bytes = f
            .values()
            .read(
                f.values()
                    .member(f.values().local(copy).unwrap(), members[0].clone())
                    .unwrap(),
            )
            .unwrap();
        body.push(f.declare(&extra, allocate(&f, bytes)));
        body.push(release(&f, f.read(&extra)));
        body.push(buffer.release(&f));
        assert_eq!(
            f.registry
                .check_storage_paths(&[aggregate_source(&f, owner, body)]),
            if wrapped {
                Err(CSafetyError::UnprovedSizeArithmetic)
            } else {
                Ok(())
            }
        );
    }
}

#[test]
fn abi_equivalent_u64_and_typedef_index_bindings_support_exact_selected_values() {
    for aliased in [false, true] {
        let mut f = Fixture::new(&[CScalarType::U64]);
        let scalar = CObjectType::scalar(CScalarType::U64);
        let ty = if aliased {
            CObjectType::typedef(
                f.registry
                    .register_typedef(&f.file, key("Index"), scalar)
                    .unwrap(),
            )
        } else {
            scalar
        };
        let index = local(&mut f, ty, "index");
        let extra = raw(&mut f, "extra");
        let buffer = Buffer::new(&mut f, CObjectType::scalar(CScalarType::Size));
        let count = f.size(2);
        let mut body = buffer.prefix(&mut f, count);
        body.push(f.declare(&index, f.input(0)));
        let condition = f.compare(
            CBinaryOperator::Less,
            f.read(&index),
            f.read(buffer.count.local()),
        );
        body.push(f.branch(
            condition,
            vec![],
            vec![buffer.release(&f), f.ast().return_statement(None).unwrap()],
        ));
        body.push(
            f.ast()
                .assign(buffer.place(&f, f.read(&index)), f.size(7))
                .unwrap(),
        );
        let bytes = f.values().read(buffer.place(&f, f.read(&index))).unwrap();
        body.push(f.declare(&extra, allocate(&f, bytes)));
        body.push(release(&f, f.read(&extra)));
        body.push(buffer.release(&f));
        // The local typedef's actual declaration must be present in its file.
        let source = f.source(body);
        if aliased {
            let alias = match index.ty().kind() {
                CObjectTypeKind::Typedef(alias) => alias,
                _ => unreachable!(),
            };
            let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
            let mut items = vec![CFileItem::Declaration(
                declarations.typedef(alias.clone()).unwrap(),
            )];
            items.extend(source.items().iter().cloned());
            let source = declarations.source_file(items).unwrap();
            assert_eq!(f.registry.check_storage_paths(&[source]), Ok(()));
        } else {
            assert_eq!(f.registry.check_storage_paths(&[source]), Ok(()));
        }
    }
}
