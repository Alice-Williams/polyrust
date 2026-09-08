//! An opaque tag's public declaration and private completion keep one identity.

use super::declarations::source_file;
use super::registry_nominals::key;
use crate::ast::{
    CAggregateRef, CDeclarationKind, CDeclarations, CFileError, CFileRole, CObjectType, CRegistry,
    CScalarType,
};

#[test]
fn aggregate_completion_checks_every_origin_and_placement_role() {
    use CFileRole as R;
    let roles = [
        R::GeneratedPublicHeader,
        R::GeneratedSource,
        R::RuntimePublicHeader,
        R::RuntimeSource,
        R::PrivateHeader,
        R::TestSource,
    ];
    for origin_role in roles {
        for target_role in roles {
            let mut registry = CRegistry::new();
            let origin = source_file(&mut registry, "origin.h", origin_role);
            let target = source_file(&mut registry, "target.h", target_role);
            let owner =
                CAggregateRef::Struct(registry.declare_struct(&origin, key("Opaque")).unwrap());
            let member = registry
                .register_member(&owner, key("value"), CObjectType::scalar(CScalarType::I32))
                .unwrap();
            registry
                .define_aggregate(&owner, vec![member.clone()])
                .unwrap();
            let public = CDeclarations::new(&registry, origin.clone()).unwrap();
            assert!(public.forward_tag(owner.clone()).is_ok());
            assert!(public.aggregate(owner.clone()).is_ok());
            let result = CDeclarations::new(&registry, target.clone())
                .unwrap()
                .aggregate(owner.clone());
            let permitted = matches!(
                (origin_role, target_role),
                (
                    R::GeneratedPublicHeader,
                    R::GeneratedSource | R::PrivateHeader
                ) | (R::RuntimePublicHeader, R::RuntimeSource | R::PrivateHeader)
                    | (R::PrivateHeader, R::GeneratedSource | R::RuntimeSource)
            );
            if permitted {
                let declaration = result.unwrap();
                assert_eq!(declaration.file(), &target);
                assert_eq!(
                    declaration.kind(),
                    &CDeclarationKind::Aggregate {
                        owner,
                        members: vec![member],
                    }
                );
            } else {
                assert_eq!(
                    result,
                    Err(CFileError::WrongFileRole),
                    "{origin_role:?} -> {target_role:?}"
                );
            }
        }
    }
}
