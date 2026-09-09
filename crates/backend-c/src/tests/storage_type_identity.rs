//! Compatibility-key equality must match the existing authenticated relation.
use super::{
    contextual_reconstruction::key, numeric_fixture::Fixture, storage_fixture::pointer_type, *,
};

fn callback(ty: CObjectType) -> CObjectType {
    CObjectType::pointer(CPointerTarget::Function(Box::new(CFunctionType::new(
        CReturnType::Value(CReturnValue::new(ty.clone()).unwrap()),
        vec![CParameterType::new(ty).unwrap()],
    ))))
}

#[test]
fn normalized_pointee_identity_agrees_with_compatibility_for_every_scalar_shape_pair() {
    let f = Fixture::new(&[]);
    let mut types = vec![];
    for scalar in CScalarType::ALL {
        for qualifier in [CConstness::Unqualified, CConstness::Const] {
            let value = CObjectType::scalar(scalar)
                .with_constness(qualifier)
                .unwrap();
            types.extend([
                value.clone(),
                pointer_type(value.clone()),
                pointer_type(pointer_type(value.clone())),
                CObjectType::array(value.clone(), CArrayLength::new(2).unwrap()).unwrap(),
                pointer_type(
                    CObjectType::array(value.clone(), CArrayLength::new(3).unwrap()).unwrap(),
                ),
                callback(pointer_type(value)),
            ]);
        }
    }
    let identities: Vec<_> = types
        .iter()
        .map(|ty| f.registry.pointee_storage_identity(ty).unwrap())
        .collect();
    for (a, left) in types.iter().enumerate() {
        assert_eq!(
            f.registry.pointee_storage_identity(&identities[a]).unwrap(),
            identities[a]
        );
        for (b, right) in types.iter().enumerate() {
            assert_eq!(
                identities[a] == identities[b],
                f.registry.pointee_types_match(left, right).unwrap(),
                "left={left:?}, right={right:?}"
            );
        }
    }
}

#[test]
fn callback_alias_metadata_normalizes_only_after_origin_authentication() {
    let mut f = Fixture::new(&[]);
    let alias = f
        .registry
        .register_typedef(&f.file, key("Alias"), CObjectType::scalar(CScalarType::I32))
        .unwrap();
    let aliased = callback(CObjectType::typedef(alias));
    let direct = callback(CObjectType::scalar(CScalarType::Int));
    assert!(f.registry.pointee_types_match(&aliased, &direct).unwrap());
    assert_eq!(
        f.registry.pointee_storage_identity(&aliased).unwrap(),
        f.registry.pointee_storage_identity(&direct).unwrap()
    );
    let foreign = Fixture::new(&[]);
    assert_eq!(
        foreign.registry.pointee_storage_identity(&aliased),
        Err(CRegistryError::CrossRegistry)
    );
}

#[test]
fn nominal_identity_and_array_lengths_do_not_collapse_to_layout_identity() {
    let mut f = Fixture::new(&[]);
    let a = f.registry.declare_struct(&f.file, key("A")).unwrap();
    let b = f.registry.declare_struct(&f.file, key("B")).unwrap();
    let a = CObjectType::structure(a);
    let b = CObjectType::structure(b);
    assert_ne!(
        f.registry.pointee_storage_identity(&a).unwrap(),
        f.registry.pointee_storage_identity(&b).unwrap()
    );
    let array = |count| CObjectType::array(a.clone(), CArrayLength::new(count).unwrap()).unwrap();
    assert_ne!(
        f.registry.pointee_storage_identity(&array(2)).unwrap(),
        f.registry.pointee_storage_identity(&array(3)).unwrap()
    );
}
