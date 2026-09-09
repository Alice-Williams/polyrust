//! Every static initializer family evaluates actual arithmetic leaves.
use super::contextual_reconstruction::{fixture, key, package};
use super::*;

#[derive(Clone, Copy)]
enum Shape {
    Scalar,
    Struct,
    Union,
}

#[test]
fn scalar_struct_union_and_zero_initializer_branches_are_checked() {
    for shape in [Shape::Scalar, Shape::Struct, Shape::Union] {
        for unsafe_operation in [false, true] {
            let (mut registry, file, function, scope) = fixture();
            let scalar = CObjectType::scalar(CScalarType::I32);
            let aggregate = match shape {
                Shape::Scalar => None,
                Shape::Struct => Some(CAggregateRef::Struct(
                    registry.declare_struct(&file, key("Record")).unwrap(),
                )),
                Shape::Union => Some(CAggregateRef::Union(
                    registry.declare_union(&file, key("Record")).unwrap(),
                )),
            };
            let member = aggregate.as_ref().map(|owner| {
                let member = registry
                    .register_member(owner, key("field"), scalar.clone())
                    .unwrap();
                registry
                    .define_aggregate(owner, vec![member.clone()])
                    .unwrap();
                member
            });
            let ty = match &aggregate {
                None => scalar,
                Some(CAggregateRef::Struct(value)) => CObjectType::structure(value.clone()),
                Some(CAggregateRef::Union(value)) => CObjectType::union(value.clone()),
            };
            let object = registry
                .register_object(&file, key("object"), ty.clone())
                .unwrap();
            let values = CExpressions::new(&registry);
            let int = |value| {
                values
                    .literal(CLiteral::Signed(CSignedLiteral::I32(value)))
                    .unwrap()
            };
            let expression = values
                .binary(
                    CBinaryOperator::Divide,
                    int(3),
                    int(if unsafe_operation { 0 } else { 1 }),
                )
                .unwrap();
            let leaf = values.expression_initializer(expression).unwrap();
            let initializer = match &aggregate {
                None => leaf,
                Some(CAggregateRef::Struct(owner)) => values
                    .struct_initializer(owner.clone(), vec![(member.clone().unwrap(), leaf)])
                    .unwrap(),
                Some(CAggregateRef::Union(owner)) => values
                    .union_initializer(owner.clone(), member.clone().unwrap(), leaf)
                    .unwrap(),
            };
            let declarations = CDeclarations::new(&registry, file.clone()).unwrap();
            for (initializer, expected) in [
                (
                    initializer,
                    if unsafe_operation {
                        Err(CSafetyError::DivisionByZero)
                    } else {
                        Ok(())
                    },
                ),
                (values.zero_initializer(ty.clone()).unwrap(), Ok(())),
            ] {
                let mut source = package(
                    &registry,
                    file.clone(),
                    function.clone(),
                    scope.clone(),
                    vec![],
                );
                if let Some(owner) = &aggregate {
                    source.items.insert(
                        0,
                        CFileItem::Declaration(declarations.aggregate(owner.clone()).unwrap()),
                    );
                }
                source.items.push(CFileItem::Definition(
                    declarations
                        .object_definition(object.clone(), CLinkage::External, initializer)
                        .unwrap(),
                ));
                registry
                    .check_context(std::slice::from_ref(&source))
                    .unwrap();
                assert_eq!(registry.check_constants_and_layout(&[source]), expected);
            }
        }
    }
}

#[test]
fn static_address_indices_check_numeric_operations_without_inventing_extent_proof() {
    for unsafe_operation in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let scalar = CObjectType::scalar(CScalarType::I32);
        let array = CObjectType::array(scalar.clone(), CArrayLength::new(1).unwrap()).unwrap();
        let storage = registry
            .register_object(&file, key("storage"), array.clone())
            .unwrap();
        let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(scalar)));
        let target = registry
            .register_object(&file, key("pointer"), pointer)
            .unwrap();
        let values = CExpressions::new(&registry);
        let size = |value| {
            values
                .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(value)))
                .unwrap()
        };
        let index = values
            .binary(
                CBinaryOperator::Divide,
                size(0),
                size(if unsafe_operation { 0 } else { 1 }),
            )
            .unwrap();
        let place = values
            .index(
                CIndexBase::Array(Box::new(values.global(storage.clone()).unwrap())),
                index,
            )
            .unwrap();
        let address = values.address_of(place).unwrap();
        let declarations = CDeclarations::new(&registry, file.clone()).unwrap();
        let mut source = package(&registry, file, function, scope, vec![]);
        source.items.extend([
            CFileItem::Definition(
                declarations
                    .object_definition(
                        storage,
                        CLinkage::External,
                        values.zero_initializer(array).unwrap(),
                    )
                    .unwrap(),
            ),
            CFileItem::Definition(
                declarations
                    .object_definition(
                        target,
                        CLinkage::External,
                        values.expression_initializer(address).unwrap(),
                    )
                    .unwrap(),
            ),
        ]);
        registry
            .check_context(std::slice::from_ref(&source))
            .unwrap();
        assert_eq!(
            registry.check_constants_and_layout(&[source]),
            if unsafe_operation {
                Err(CSafetyError::DivisionByZero)
            } else {
                Ok(())
            }
        );
    }
}

#[test]
fn sizeof_alignof_numeric_values_and_alias_hidden_array_capacity_are_checked() {
    let (registry, file, function, scope) = fixture();
    let values = CExpressions::new(&registry);
    let array = CObjectType::array(
        CObjectType::scalar(CScalarType::U64),
        CArrayLength::new(3).unwrap(),
    )
    .unwrap();
    let declarations = CDeclarations::new(&registry, file.clone()).unwrap();
    let mut source = package(&registry, file, function, scope, vec![]);
    for (value, expected) in [
        (values.size_of(array.clone()).unwrap(), 24),
        (values.align_of(array).unwrap(), 8),
        (
            values
                .size_of(CObjectType::known(CKnownObject::MaxAlign))
                .unwrap(),
            32,
        ),
    ] {
        let expected = values
            .literal(CLiteral::Unsigned(CUnsignedLiteral::Size(expected)))
            .unwrap();
        source.items.push(CFileItem::StaticAssert(
            declarations
                .static_assert(
                    values
                        .binary(CBinaryOperator::Equal, value, expected)
                        .unwrap(),
                    CAssertDiagnostic::new("layout value"),
                )
                .unwrap(),
        ));
    }
    registry.check_constants_and_layout(&[source]).unwrap();
    for overflow in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let array = CObjectType::array(
            CObjectType::scalar(if overflow {
                CScalarType::U64
            } else {
                CScalarType::U8
            }),
            CArrayLength::new(u64::MAX).unwrap(),
        )
        .unwrap();
        let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(array)));
        let alias = registry
            .register_typedef(&file, key("ArrayPointer"), pointer)
            .unwrap();
        let declarations = CDeclarations::new(&registry, file.clone()).unwrap();
        let mut source = package(&registry, file, function, scope, vec![]);
        source.items.insert(
            0,
            CFileItem::Declaration(declarations.typedef(alias).unwrap()),
        );
        registry
            .check_context(std::slice::from_ref(&source))
            .unwrap();
        assert_eq!(
            registry.check_constants_and_layout(&[source]),
            if overflow {
                Err(CSafetyError::LayoutCapacity)
            } else {
                Ok(())
            }
        );
        // The non-overflowing bound still needs stage04 compiler-capacity proof.
    }
}
