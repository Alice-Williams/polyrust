use super::*;
use crate::{dialect::JavaDependencyApi, tests::source_dependency_fixture as f};

#[test]
fn all_source_functions_are_described_but_private_functions_are_not_importable() {
    let api =
        JavaDependencyApi::from_certificate(f::certify(f::package(7, f::functions(42)))).unwrap();
    let descriptions = api.source_descriptions().unwrap();
    assert_eq!(descriptions.len(), 4);
    assert_eq!(
        descriptions
            .iter()
            .map(|item| item.source().declaration)
            .collect::<Vec<_>>(),
        (10..14).map(|hash| f::id(7, hash)).collect::<Vec<_>>()
    );
    for item in descriptions {
        let JavaSourceDescriptionKind::Function { parameters, result } = item.kind() else {
            panic!("function")
        };
        let JavaSourceTarget::Declaration(path) = item.target() else {
            panic!("declaration path")
        };
        if let Some(public) = api.function(item.source().declaration) {
            assert_eq!(item.source(), public.source());
            assert_eq!(path, public.path());
            assert_eq!(
                parameters.iter().map(|p| p.ty.clone()).collect::<Vec<_>>(),
                public.declaration_signature().parameters
            );
            assert_eq!(result, &public.declaration_signature().result);
        } else {
            assert_eq!(item.source().declaration, f::id(7, 13));
            assert!(!item.source().externally_reachable);
        }
    }
}

#[test]
fn private_record_and_field_metadata_retain_exact_origins_and_typed_paths() {
    let api = JavaDependencyApi::from_certificate(f::certify(f::record_package(|_| {}))).unwrap();
    let descriptions = api.source_descriptions().unwrap();
    assert_eq!(
        descriptions
            .iter()
            .map(|item| item.source().declaration)
            .collect::<Vec<_>>(),
        (2..6).map(|hash| f::id(7, hash)).collect::<Vec<_>>()
    );
    let record = &descriptions[0];
    assert_eq!(record.kind(), JavaSourceDescriptionKind::Record);
    let JavaSourceTarget::Declaration(record_path) = record.target() else {
        panic!("record path")
    };
    for field in &descriptions[1..3] {
        let JavaSourceDescriptionKind::Field { owner, ty } = field.kind() else {
            panic!("field")
        };
        assert_eq!(owner, record.source().declaration);
        assert!([f::int(), f::boolean()].contains(ty));
        let JavaSourceTarget::Field {
            owner: target_owner,
            member,
        } = field.target()
        else {
            panic!("field path")
        };
        assert_eq!(target_owner, record_path);
        assert!(!member.as_str().is_empty());
        assert!(!field.source().documentation.is_empty());
        assert!(api.function(field.source().declaration).is_none());
        assert!(!field.source().externally_reachable);
    }
    assert!(api.function(record.source().declaration).is_none());
    assert_eq!(api.source_descriptions().unwrap(), descriptions);
}
