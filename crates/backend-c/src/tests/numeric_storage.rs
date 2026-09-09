//! Numeric projections distinguish fields and elements and invalidate whole roots.
use super::{contextual_reconstruction::key, numeric_fixture::Fixture, *};
use crate::dialect::CKnownCall;

#[test]
fn array_initializer_selectors_are_exact_and_unknown_writes_kill_all_elements() {
    for changed in [false, true] {
        let f = &mut Fixture::new(&[CScalarType::Size]);
        let ty = CObjectType::array(
            CObjectType::scalar(CScalarType::Int),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let array = f
            .registry
            .register_local(&f.scope, key("array"), ty.clone())
            .unwrap();
        let slot = |index| {
            f.values()
                .index(
                    CIndexBase::Array(Box::new(f.values().local(array.clone()).unwrap())),
                    index,
                )
                .unwrap()
        };
        let initializer = f
            .values()
            .array_initializer(
                ty,
                vec![
                    f.values().expression_initializer(f.int(1)).unwrap(),
                    f.values().expression_initializer(f.int(0)).unwrap(),
                ],
            )
            .unwrap();
        let mut body = vec![f.ast().declare(array.clone(), Some(initializer)).unwrap()];
        if changed {
            body.push(f.ast().assign(slot(f.input(0)), f.int(0)).unwrap());
        }
        body.push(f.discard(f.binary(
            CBinaryOperator::Divide,
            f.int(1),
            f.values().read(slot(f.size(0))).unwrap(),
        )));
        let result = f.check(body);
        if changed {
            assert_eq!(result, Err(CSafetyError::DivisionByZero));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn structure_initializer_member_identity_is_not_a_whole_object_range() {
    for selected in [0, 1] {
        let mut f = Fixture::new(&[]);
        let record = f.registry.declare_struct(&f.file, key("Record")).unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        let members: Vec<_> = ["one", "zero"]
            .iter()
            .map(|name| {
                f.registry
                    .register_member(&owner, key(name), CObjectType::scalar(CScalarType::Int))
                    .unwrap()
            })
            .collect();
        f.registry
            .define_aggregate(&owner, members.clone())
            .unwrap();
        let local = f
            .registry
            .register_local(
                &f.scope,
                key("record"),
                CObjectType::structure(record.clone()),
            )
            .unwrap();
        let initializer = f
            .values()
            .struct_initializer(
                record,
                vec![
                    (
                        members[0].clone(),
                        f.values().expression_initializer(f.int(1)).unwrap(),
                    ),
                    (
                        members[1].clone(),
                        f.values().expression_initializer(f.int(0)).unwrap(),
                    ),
                ],
            )
            .unwrap();
        let member = f
            .values()
            .member(
                f.values().local(local.clone()).unwrap(),
                members[selected].clone(),
            )
            .unwrap();
        let mut source = f.source(vec![
            f.ast().declare(local, Some(initializer)).unwrap(),
            f.discard(f.binary(
                CBinaryOperator::Divide,
                f.int(1),
                f.values().read(member).unwrap(),
            )),
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
        let result = f.registry.check_numeric_flow(&[source]);
        if selected == 0 {
            result.unwrap();
        } else {
            assert_eq!(result, Err(CSafetyError::DivisionByZero));
        }
    }
}

#[test]
fn opaque_calls_kill_global_ranges_without_requiring_an_explicit_address() {
    for opaque in [false, true] {
        let mut f = Fixture::new(&[]);
        let ty = CObjectType::scalar(CScalarType::Int);
        let object = f
            .registry
            .register_object(&f.file, key("global"), ty)
            .unwrap();
        let place = f.values().global(object.clone()).unwrap();
        let mut body = vec![f.ast().assign(place.clone(), f.int(1)).unwrap()];
        if opaque {
            body.push(
                f.discard(
                    f.values()
                        .call_value(f.values().known(CKnownCall::Allocate), vec![f.size(1)])
                        .unwrap(),
                ),
            );
        }
        body.push(f.discard(f.binary(
            CBinaryOperator::Divide,
            f.int(1),
            f.values().read(place).unwrap(),
        )));
        let mut source = f.source(body);
        source.items.insert(
            0,
            CFileItem::Definition(
                CDeclarations::new(&f.registry, f.file.clone())
                    .unwrap()
                    .object_definition(
                        object,
                        CLinkage::External,
                        f.values().expression_initializer(f.int(0)).unwrap(),
                    )
                    .unwrap(),
            ),
        );
        let result = f.registry.check_numeric_flow(&[source]);
        if opaque {
            assert_eq!(result, Err(CSafetyError::DivisionByZero));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn address_only_place_traversal_still_checks_hidden_index_arithmetic() {
    let mut f = Fixture::new(&[CScalarType::Int]);
    let ty = CObjectType::array(
        CObjectType::scalar(CScalarType::Int),
        CArrayLength::new(2).unwrap(),
    )
    .unwrap();
    let array = f
        .registry
        .register_local(&f.scope, key("array"), ty)
        .unwrap();
    let index = f.binary(CBinaryOperator::Divide, f.int(1), f.input(0));
    let place = f
        .values()
        .index(
            CIndexBase::Array(Box::new(f.values().local(array.clone()).unwrap())),
            index,
        )
        .unwrap();
    let address = f.discard(f.values().address_of(place).unwrap());
    assert_eq!(
        f.check(vec![f.ast().declare(array, None).unwrap(), address]),
        Err(CSafetyError::DivisionByZero)
    );
}
