//! Layout facts use actual members and checked arithmetic, never host sizeof.
use super::{E, Layouts};
use crate::ast::*;
use portable_codegen::RelativeOutputPath;

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}
fn registry() -> (CRegistry, CFileRef) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("src/layout.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    (registry, file)
}
fn array(ty: CObjectType, count: u64) -> CObjectType {
    CObjectType::array(ty, CArrayLength::new(count).unwrap()).unwrap()
}

#[test]
fn exact_scalar_pointer_known_enum_and_alias_layouts() {
    let (mut registry, file) = registry();
    let opaque = registry.declare_struct(&file, key("Opaque")).unwrap();
    let enumeration = registry.declare_enum(&file, key("Tag")).unwrap();
    let member = registry
        .register_enumerator(&enumeration, key("First"), -1)
        .unwrap();
    registry.define_enum(&enumeration, vec![member]).unwrap();
    let alias = registry
        .register_typedef(&file, key("Count"), CObjectType::scalar(CScalarType::Size))
        .unwrap();
    let mut layouts = Layouts::new(&registry);
    for (ty, size) in [
        (CScalarType::Bool, 1),
        (CScalarType::PlainChar, 1),
        (CScalarType::I8, 1),
        (CScalarType::U8, 1),
        (CScalarType::I16, 2),
        (CScalarType::U16, 2),
        (CScalarType::Int, 4),
        (CScalarType::I32, 4),
        (CScalarType::U32, 4),
        (CScalarType::I64, 8),
        (CScalarType::U64, 8),
        (CScalarType::Size, 8),
        (CScalarType::F64, 8),
    ] {
        let actual = layouts.object(&CObjectType::scalar(ty)).unwrap();
        assert_eq!((actual.size(), actual.alignment()), (size, size));
    }
    let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::structure(
        opaque.clone(),
    ))));
    let callback = CObjectType::pointer(CPointerTarget::Function(Box::new(CFunctionType::new(
        CReturnType::Void,
        vec![],
    ))));
    for ty in [
        pointer,
        callback,
        CObjectType::typedef(alias),
        CObjectType::pointer(CPointerTarget::Void(CConstness::Const)),
    ] {
        let actual = layouts.object(&ty).unwrap();
        assert_eq!((actual.size(), actual.alignment()), (8, 8));
    }
    let actual = layouts
        .object(&CObjectType::known(CKnownObject::MaxAlign))
        .unwrap();
    assert_eq!((actual.size(), actual.alignment()), (32, 16));
    assert_eq!(
        layouts.object(&CObjectType::known(CKnownObject::File)),
        Err(E::IncompleteLayout)
    );
    assert_eq!(
        layouts.object(&CObjectType::structure(opaque)),
        Err(E::IncompleteLayout)
    );
    let actual = layouts
        .object(&CObjectType::enumeration(enumeration))
        .unwrap();
    assert_eq!((actual.size(), actual.alignment()), (4, 4));
}

#[test]
fn struct_union_offsets_tail_padding_and_nested_arrays_follow_member_order() {
    for union in [false, true] {
        let (mut registry, file) = registry();
        let (owner, ty) = if union {
            let value = registry.declare_union(&file, key("Record")).unwrap();
            (
                CAggregateRef::Union(value.clone()),
                CObjectType::union(value),
            )
        } else {
            let value = registry.declare_struct(&file, key("Record")).unwrap();
            (
                CAggregateRef::Struct(value.clone()),
                CObjectType::structure(value),
            )
        };
        let mut members = vec![];
        for (name, scalar) in [
            ("first", CScalarType::U8),
            ("middle", CScalarType::U64),
            ("last", CScalarType::U16),
        ] {
            members.push(
                registry
                    .register_member(&owner, key(name), CObjectType::scalar(scalar))
                    .unwrap(),
            );
        }
        registry.define_aggregate(&owner, members.clone()).unwrap();
        let mut layouts = Layouts::new(&registry);
        let actual = layouts.object(&ty).unwrap();
        assert_eq!(
            (actual.size(), actual.alignment()),
            (if union { 8 } else { 24 }, 8)
        );
        assert_eq!(
            members
                .iter()
                .map(|member| layouts.offsets[member])
                .collect::<Vec<_>>(),
            if union { vec![0, 0, 0] } else { vec![0, 8, 16] }
        );
        let repeated = layouts.object(&array(array(ty, 2), 3)).unwrap();
        assert_eq!(repeated.size(), actual.size() * 6);
        assert_eq!(repeated.alignment(), 8);
    }
}

#[test]
fn cycles_incomplete_elements_and_capacity_arithmetic_have_distinct_failures() {
    let (mut registry, file) = registry();
    let first = registry.declare_struct(&file, key("First")).unwrap();
    let second = registry.declare_struct(&file, key("Second")).unwrap();
    for (owner, target) in [
        (first.clone(), second.clone()),
        (second.clone(), first.clone()),
    ] {
        let owner = CAggregateRef::Struct(owner);
        let field = registry
            .register_member(&owner, key("child"), CObjectType::structure(target))
            .unwrap();
        registry.define_aggregate(&owner, vec![field]).unwrap();
    }
    let opaque = registry.declare_struct(&file, key("Opaque")).unwrap();
    let mut layouts = Layouts::new(&registry);
    assert_eq!(
        layouts.object(&CObjectType::structure(first)),
        Err(E::RecursiveLayout)
    );
    assert_eq!(
        layouts.object(&array(CObjectType::structure(opaque), 2)),
        Err(E::IncompleteLayout)
    );
    assert_eq!(
        layouts.object(&array(CObjectType::scalar(CScalarType::U64), u64::MAX)),
        Err(E::LayoutCapacity)
    );
    let largest = array(CObjectType::scalar(CScalarType::U8), u64::MAX);
    assert_eq!(layouts.object(&largest).unwrap().size(), u64::MAX);
    assert_eq!(super::align(u64::MAX, 8), Err(E::LayoutCapacity));
    assert_eq!(super::align(u64::MAX - 7, 8).unwrap(), u64::MAX - 7);
    // Layout representability is deliberately not native compiler capacity.
    assert_eq!(layouts.object(&array(largest, 2)), Err(E::LayoutCapacity));
}

#[test]
fn aggregate_sum_and_tail_padding_overflow_and_deep_graphs_are_checked() {
    for tail in [false, true] {
        let (mut registry, file) = registry();
        let record = registry.declare_struct(&file, key("Huge")).unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        let members = [
            registry
                .register_member(
                    &owner,
                    key("aligner"),
                    CObjectType::scalar(if tail {
                        CScalarType::U64
                    } else {
                        CScalarType::U8
                    }),
                )
                .unwrap(),
            registry
                .register_member(
                    &owner,
                    key("bytes"),
                    array(
                        CObjectType::scalar(CScalarType::U8),
                        if tail { u64::MAX - 8 } else { u64::MAX },
                    ),
                )
                .unwrap(),
        ];
        registry.define_aggregate(&owner, members.to_vec()).unwrap();
        assert_eq!(
            Layouts::new(&registry).object(&CObjectType::structure(record)),
            Err(E::LayoutCapacity)
        );
    }
    let (mut registry, file) = registry();
    let mut ty = CObjectType::scalar(CScalarType::U8);
    for depth in 0..2048 {
        let record = registry
            .declare_struct(&file, key(&format!("Record{depth}")))
            .unwrap();
        let owner = CAggregateRef::Struct(record.clone());
        let member = registry.register_member(&owner, key("child"), ty).unwrap();
        registry.define_aggregate(&owner, vec![member]).unwrap();
        ty = CObjectType::structure(record);
    }
    assert_eq!(Layouts::new(&registry).object(&ty).unwrap().size(), 1);
}
