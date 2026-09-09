//! File-scope address initializers cannot bypass actual index-extent checking.
use super::*;
use crate::ast::{numeric_fixture::Fixture, *};

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}

fn fixture(index: i32, wrapped: bool) -> (Fixture, [CSourceFile; 1]) {
    shaped_fixture(index, wrapped, Shape::Plain)
}

#[derive(Clone, Copy)]
enum Shape {
    Plain,
    Array,
    Struct,
    Union,
    Conversions,
}

fn decorate(
    f: &mut Fixture,
    address: CValue,
    shape: Shape,
) -> (CInitializer, Option<CAggregateRef>) {
    let leaf = f.values().expression_initializer(address.clone()).unwrap();
    match shape {
        Shape::Plain => (leaf, None),
        Shape::Array => {
            let ty =
                CObjectType::array(address.ty().clone(), CArrayLength::new(1).unwrap()).unwrap();
            (f.values().array_initializer(ty, vec![leaf]).unwrap(), None)
        }
        Shape::Struct | Shape::Union => {
            let owner = if matches!(shape, Shape::Struct) {
                CAggregateRef::Struct(
                    f.registry
                        .declare_struct(&f.file, key("Container"))
                        .unwrap(),
                )
            } else {
                CAggregateRef::Union(f.registry.declare_union(&f.file, key("Container")).unwrap())
            };
            let member = f
                .registry
                .register_member(&owner, key("pointer"), address.ty().clone())
                .unwrap();
            f.registry
                .define_aggregate(&owner, vec![member.clone()])
                .unwrap();
            let initializer = match &owner {
                CAggregateRef::Struct(record) => f
                    .values()
                    .struct_initializer(record.clone(), vec![(member, leaf)])
                    .unwrap(),
                CAggregateRef::Union(union) => f
                    .values()
                    .union_initializer(union.clone(), member, leaf)
                    .unwrap(),
            };
            (initializer, Some(owner))
        }
        Shape::Conversions => {
            let qualified = CObjectType::pointer(CPointerTarget::Object(Box::new(
                CObjectType::scalar(CScalarType::Int)
                    .with_constness(CConstness::Const)
                    .unwrap(),
            )));
            let address = f.values().add_const(qualified, address).unwrap();
            let address = f
                .values()
                .object_to_void(
                    CObjectType::pointer(CPointerTarget::Void(CConstness::Const)),
                    address,
                )
                .unwrap();
            (f.values().expression_initializer(address).unwrap(), None)
        }
    }
}

fn shaped_fixture(index: i32, wrapped: bool, shape: Shape) -> (Fixture, [CSourceFile; 1]) {
    let mut f = Fixture::new(&[]);
    let scalar = CObjectType::scalar(CScalarType::Int);
    let array_type = CObjectType::array(scalar.clone(), CArrayLength::new(2).unwrap()).unwrap();
    let array = f
        .registry
        .register_object(&f.file, key("array"), array_type.clone())
        .unwrap();
    let index = if wrapped {
        f.binary(CBinaryOperator::Add, f.size(u64::MAX), f.size(1))
    } else {
        f.int(index)
    };
    let place = f
        .values()
        .index(
            CIndexBase::Array(Box::new(f.values().global(array.clone()).unwrap())),
            index,
        )
        .unwrap();
    let address = f.values().address_of(place).unwrap();
    let (initializer, owner) = decorate(&mut f, address, shape);
    let pointer = f
        .registry
        .register_object(&f.file, key("pointer"), initializer.ty().clone())
        .unwrap();
    let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
    let mut items = f.source(vec![]).items().to_vec();
    items.insert(
        0,
        CFileItem::Definition(
            declarations
                .object_definition(
                    array,
                    CLinkage::External,
                    f.values().zero_initializer(array_type).unwrap(),
                )
                .unwrap(),
        ),
    );
    items.insert(
        1,
        CFileItem::Definition(
            declarations
                .object_definition(pointer, CLinkage::External, initializer)
                .unwrap(),
        ),
    );
    if let Some(owner) = owner {
        items.insert(
            0,
            CFileItem::Declaration(declarations.aggregate(owner).unwrap()),
        );
    }
    let source = declarations.source_file(items).unwrap();
    (f, [source])
}

#[test]
fn static_initializer_wrappers_and_qualifiers_cannot_hide_an_invalid_index() {
    for shape in [
        Shape::Array,
        Shape::Struct,
        Shape::Union,
        Shape::Conversions,
    ] {
        for (index, wrapped, expected) in [
            (1, false, Ok(())),
            (2, false, Err(E::IndexOutOfBounds)),
            (0, true, Err(E::UnprovedSizeArithmetic)),
        ] {
            let (f, files) = shaped_fixture(index, wrapped, shape);
            assert_eq!(f.registry.check_index_extents(&files), expected);
        }
    }
}

#[test]
fn a_static_member_address_checks_indices_in_its_containing_place() {
    for index in [0, 2] {
        let mut f = Fixture::new(&[]);
        let record = f.registry.declare_struct(&f.file, key("Element")).unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        let member = f
            .registry
            .register_member(&owner, key("value"), CObjectType::scalar(CScalarType::Int))
            .unwrap();
        f.registry
            .define_aggregate(&owner, vec![member.clone()])
            .unwrap();
        let array_type = CObjectType::array(
            CObjectType::structure(record),
            CArrayLength::new(2).unwrap(),
        )
        .unwrap();
        let array = f
            .registry
            .register_object(&f.file, key("array"), array_type.clone())
            .unwrap();
        let base = f
            .values()
            .index(
                CIndexBase::Array(Box::new(f.values().global(array.clone()).unwrap())),
                f.int(index),
            )
            .unwrap();
        let place = f.values().member(base, member).unwrap();
        let address = f.values().address_of(place).unwrap();
        let pointer = f
            .registry
            .register_object(&f.file, key("pointer"), address.ty().clone())
            .unwrap();
        let declarations = CDeclarations::new(&f.registry, f.file.clone()).unwrap();
        let mut items = vec![
            CFileItem::Declaration(declarations.aggregate(owner).unwrap()),
            CFileItem::Definition(
                declarations
                    .object_definition(
                        array,
                        CLinkage::External,
                        f.values().zero_initializer(array_type).unwrap(),
                    )
                    .unwrap(),
            ),
            CFileItem::Definition(
                declarations
                    .object_definition(
                        pointer,
                        CLinkage::External,
                        f.values().expression_initializer(address).unwrap(),
                    )
                    .unwrap(),
            ),
        ];
        items.extend_from_slice(f.source(vec![]).items());
        let source = declarations.source_file(items).unwrap();
        assert_eq!(
            f.registry.check_index_extents(&[source]),
            if index == 0 {
                Ok(())
            } else {
                Err(E::IndexOutOfBounds)
            }
        );
    }
}

#[test]
fn static_address_indices_require_the_actual_declared_extent() {
    for index in [0, 1, -1, 2, 3] {
        let (f, files) = fixture(index, false);
        assert_eq!(
            f.registry.check_index_extents(&files),
            if (0..2).contains(&index) {
                Ok(())
            } else {
                Err(E::IndexOutOfBounds)
            }
        );
    }
}

#[test]
fn static_address_indices_cannot_lose_unsigned_wrap_history() {
    let (f, files) = fixture(0, true);
    assert_eq!(
        f.registry.check_index_extents(&files),
        Err(E::UnprovedSizeArithmetic)
    );
}

#[test]
fn storage_composition_rechecks_static_indices_in_every_initializer_shape() {
    for shape in [
        Shape::Plain,
        Shape::Array,
        Shape::Struct,
        Shape::Union,
        Shape::Conversions,
    ] {
        for (index, wrapped) in [(0, false), (2, false), (0, true)] {
            let (f, files) = shaped_fixture(index, wrapped, shape);
            assert_eq!(
                f.registry.check_storage_paths(&files),
                if wrapped {
                    Err(E::UnprovedSizeArithmetic)
                } else if index == 2 {
                    Err(E::IndexOutOfBounds)
                } else {
                    Ok(())
                }
            );
        }
    }
}

#[test]
fn static_observation_requires_its_actual_initializer_and_place_occurrence() {
    let (f, files) = fixture(1, false);
    let mut facts = NumericFacts::check(&f.registry, &files).unwrap();
    assert_eq!(facts.static_indices.len(), 1);
    let clone = facts.static_indices[0].location.place.clone();
    facts.static_indices[0].location.place = &clone;
    assert_eq!(facts.validate_sites(), Err(E::InvalidNumericSite));
    let mut facts = NumericFacts::check(&f.registry, &files).unwrap();
    let clone = facts.static_indices[0].location.initializer.clone();
    facts.static_indices[0].location.initializer = &clone;
    assert_eq!(facts.validate_sites(), Err(E::InvalidNumericSite));
    let mut facts = NumericFacts::check(&f.registry, &files).unwrap();
    facts.static_indices.clear();
    assert_eq!(facts.validate_sites(), Err(E::InvalidNumericSite));
}
