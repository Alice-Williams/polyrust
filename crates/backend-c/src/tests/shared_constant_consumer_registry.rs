//! Registry brands and producer authority cannot be replaced with copied metadata.
use super::{
    constant_consumer_fixture::producer,
    owned_constant_fixture::{Shape, key},
};
use crate::ast::*;
use portable_codegen::RelativeOutputPath;

fn files(registry: &mut CRegistry) -> (CFileRef, CFileRef) {
    let mut file = |path, role| {
        registry
            .register_file(CFileKey {
                path: RelativeOutputPath::new(path).unwrap(),
                role,
            })
            .unwrap()
    };
    (
        file("reader.h", CFileRole::GeneratedPublicHeader),
        file("reader.c", CFileRole::GeneratedSource),
    )
}

#[test]
fn imports_are_foreign_readonly_objects_never_owned_declarations() {
    let owner = producer(Shape::ConstantsOnly);
    let dependency = owner.constants().next().unwrap().clone();
    let mut first = CRegistry::new();
    let (header, source) = files(&mut first);
    let object = first.import_constant(dependency.clone()).unwrap();
    assert_eq!(first.imported_constant(&object).unwrap(), &dependency);
    let ast = CExpressions::new(&first);
    let read = ast.read(ast.global(object.clone()).unwrap()).unwrap();
    assert_eq!(read.ty(), dependency.read_type());
    assert!(
        CDeclarations::new(&first, header)
            .unwrap()
            .object_declaration(object.clone())
            .is_err()
    );
    assert!(
        CDeclarations::new(&first, source)
            .unwrap()
            .object_definition(
                object.clone(),
                CLinkage::External,
                ast.expression_initializer(
                    ast.literal(dependency.value().literal().unwrap()).unwrap()
                )
                .unwrap()
            )
            .is_err()
    );
    let mut second = CRegistry::new();
    let other = second.import_constant(dependency).unwrap();
    assert_ne!(object, other);
    assert_eq!(
        second.check_object(&object),
        Err(CRegistryError::CrossRegistry)
    );
    assert!(CExpressions::new(&second).global(object).is_err());
    assert!(first.check_object(&other).is_err());
}

#[test]
fn value_and_function_registration_share_authority_and_paths_in_both_orders() {
    let owner = producer(Shape::Mixed);
    let other = producer(Shape::Mixed);
    for function_first in [false, true] {
        let mut registry = CRegistry::new();
        let value = owner.constants().next().unwrap().clone();
        let call = owner.functions().next().unwrap().clone();
        if function_first {
            registry.import_function(call).unwrap();
            registry.import_constant(value).unwrap();
            assert!(
                registry
                    .import_constant(other.constants().nth(1).unwrap().clone())
                    .is_err()
            );
        } else {
            registry.import_constant(value).unwrap();
            registry.import_function(call).unwrap();
            assert!(
                registry
                    .import_function(other.functions().nth(1).unwrap().clone())
                    .is_err()
            );
        }
        assert!(
            registry
                .import_constant(owner.constants().next().unwrap().clone())
                .is_err()
        );
        assert!(
            registry
                .register_file(CFileKey {
                    path: RelativeOutputPath::new("other/polyrust_constants.h").unwrap(),
                    role: CFileRole::GeneratedPublicHeader,
                })
                .is_err()
        );
    }
    let mut registry = CRegistry::new();
    registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("polyrust_constants.h").unwrap(),
            role: CFileRole::GeneratedPublicHeader,
        })
        .unwrap();
    assert!(
        registry
            .import_constant(owner.constants().next().unwrap().clone())
            .is_err()
    );
}

#[test]
fn copying_an_imported_key_cannot_register_owned_storage_or_functions() {
    let owner = producer(Shape::ConstantsOnly);
    let dependency = owner.constants().next().unwrap().clone();
    for imported_first in [false, true] {
        let mut registry = CRegistry::new();
        let (header, _) = files(&mut registry);
        if imported_first {
            registry.import_constant(dependency.clone()).unwrap();
            assert!(
                registry
                    .register_object(
                        &header,
                        dependency.object().key().clone(),
                        dependency.object().ty().clone()
                    )
                    .is_err()
            );
            assert!(
                registry
                    .register_function(
                        &header,
                        dependency.object().key().clone(),
                        CFunctionType::new(
                            CReturnType::Value(
                                CReturnValue::new(dependency.read_type().clone()).unwrap()
                            ),
                            vec![]
                        )
                    )
                    .is_err()
            );
        } else {
            registry
                .register_object(
                    &header,
                    dependency.object().key().clone(),
                    dependency.object().ty().clone(),
                )
                .unwrap();
            assert!(registry.import_constant(dependency.clone()).is_err());
        }
    }
    let mut registry = CRegistry::new();
    let (header, _) = files(&mut registry);
    registry
        .register_object(
            &header,
            key("ordinary"),
            CObjectType::scalar(CScalarType::I32),
        )
        .unwrap();
    registry.import_constant(dependency).unwrap();
}
