//! Deliberately target-only metadata: no claim of rustc authentication.
use super::*;
use crate::tests::source_dependency_fixture as fixture;
use portable_codegen::{RustDeclarationId, RustFieldTypes, RustFunctionTypes};
use std::collections::BTreeMap;

use crate::tests::source_package_fixture as package_fixture;

fn facts(api: &JavaDependencyApi) -> RustSourceTypes {
    let kind = |ty: &JavaType| match ty {
        JavaType::Primitive(JavaPrimitive::Int) => RustScalarKind::I32,
        JavaType::Primitive(JavaPrimitive::Boolean) => RustScalarKind::Bool,
        _ => panic!("fixture scalar"),
    };
    let mut functions = BTreeMap::new();
    let mut fields = BTreeMap::new();
    for description in api.source_descriptions().unwrap() {
        let id = description.source().declaration;
        match description.kind() {
            JavaSourceDescriptionKind::Function { parameters, result } => {
                functions.insert(
                    id,
                    RustFunctionTypes {
                        parameters: parameters.iter().map(|p| kind(&p.ty)).collect(),
                        result: RustResultKind::Scalar(kind(result)),
                    },
                );
            }
            JavaSourceDescriptionKind::Field { ty, owner } => {
                fields.insert(
                    id,
                    RustFieldTypes {
                        owner,
                        kind: kind(ty),
                    },
                );
            }
            JavaSourceDescriptionKind::Record => {}
            _ => panic!("fixture description"),
        }
    }
    RustSourceTypes::new(api.root(), functions, fields).unwrap()
}

fn check(api: &JavaDependencyApi, facts: &RustSourceTypes) -> Result<(), String> {
    verify_inventory(api.root(), &api.source_descriptions().unwrap(), facts)
}

#[test]
fn original_char_and_i32_have_equal_target_storage_but_distinct_descriptions() {
    let api = JavaDependencyApi::from_certificate(fixture::certify(fixture::package(
        7,
        fixture::functions(42),
    )))
    .unwrap();
    let ordinary = facts(&api);
    assert!(!ordinary.contains_char());
    check(&api, &ordinary).unwrap();
    let mut functions = ordinary.functions().clone();
    functions.get_mut(&fixture::id(7, 10)).unwrap().result =
        RustResultKind::Scalar(RustScalarKind::Char);
    let characters = RustSourceTypes::new(api.root(), functions, BTreeMap::new()).unwrap();
    check(&api, &characters).unwrap();
    assert_ne!(characters, ordinary);
    assert!(characters.contains_char());
    // The compiler adapter separately authenticates original Rust types.
    assert!(api.source_types().is_none());
    assert!(
        api.function(fixture::id(7, 10))
            .unwrap()
            .source_signature()
            .is_none()
    );
}

#[test]
fn function_metadata_rejects_missing_extra_owner_arity_parameter_and_result_faults() {
    let api = JavaDependencyApi::from_certificate(fixture::certify(fixture::package(
        7,
        fixture::functions(42),
    )))
    .unwrap();
    let baseline = facts(&api);
    for fault in 0..7 {
        let mut root = baseline.root();
        let mut functions = baseline.functions().clone();
        match fault {
            0 => {
                functions.remove(&fixture::id(7, 10));
            }
            1 => {
                let signature = functions[&fixture::id(7, 10)].clone();
                functions.insert(fixture::id(7, 999), signature);
            }
            2 => root.definition_path_hash ^= 1,
            3 => {
                functions
                    .get_mut(&fixture::id(7, 11))
                    .unwrap()
                    .parameters
                    .pop()
                    .unwrap();
            }
            4 => {
                functions.get_mut(&fixture::id(7, 11)).unwrap().parameters[0] = RustScalarKind::F64
            }
            5 => {
                functions.get_mut(&fixture::id(7, 10)).unwrap().result =
                    RustResultKind::Scalar(RustScalarKind::I64)
            }
            6 => functions.get_mut(&fixture::id(7, 10)).unwrap().result = RustResultKind::Unit,
            _ => unreachable!(),
        }
        let changed = RustSourceTypes::new(root, functions, BTreeMap::new()).unwrap();
        assert!(check(&api, &changed).is_err(), "fault {fault}");
    }
}

#[test]
fn field_metadata_rejects_missing_extra_wrong_owner_and_wrong_representation() {
    let api =
        JavaDependencyApi::from_certificate(fixture::certify(fixture::record_package(|_| {})))
            .unwrap();
    let baseline = facts(&api);
    check(&api, &baseline).unwrap();
    assert_eq!(baseline.fields().len(), 2);
    for fault in 0..4 {
        let mut fields = baseline.fields().clone();
        let id = *fields.keys().next().unwrap();
        match fault {
            0 => {
                fields.remove(&id);
            }
            1 => {
                fields.insert(
                    RustDeclarationId {
                        definition_path_hash: 999,
                        ..id
                    },
                    fields[&id],
                );
            }
            2 => {
                fields.get_mut(&id).unwrap().kind = RustScalarKind::F64;
            }
            3 => fields.get_mut(&id).unwrap().owner.definition_path_hash = 998,
            _ => unreachable!(),
        }
        let changed =
            RustSourceTypes::new(api.root(), baseline.functions().clone(), fields).unwrap();
        assert!(check(&api, &changed).is_err(), "fault {fault}");
    }
}

#[test]
fn api_retains_facts_from_exact_certificate_and_rejects_a_wrong_representation() {
    use crate::ast::JavaSourcePackage;
    use package_fixture::rebuild;
    let package = fixture::package(7, fixture::functions(42));
    let api = JavaDependencyApi::from_certificate(fixture::certify(package.clone())).unwrap();
    let baseline = facts(&api);
    let exports = api
        .function(fixture::id(7, 10))
        .unwrap()
        .source()
        .crate_exports
        .clone();
    let attach = |types: RustSourceTypes| {
        let package = rebuild(package.clone(), |file| {
            for item in &mut file.items {
                if let JavaFileItem::Type {
                    package_metadata, ..
                } = item
                {
                    *package_metadata = Some(
                        JavaSourcePackage::new(exports.clone())
                            .with_source_types(Arc::new(types.clone()))
                            .into(),
                    );
                }
            }
        });
        JavaDependencyApi::from_certificate(fixture::certify(package))
    };
    let retained = attach(baseline.clone()).unwrap();
    assert_eq!(retained.source_types(), Some(&baseline));
    assert_eq!(
        retained
            .function(fixture::id(7, 10))
            .unwrap()
            .source_signature(),
        baseline.functions().get(&fixture::id(7, 10))
    );
    let mut functions = baseline.functions().clone();
    functions.get_mut(&fixture::id(7, 10)).unwrap().result =
        RustResultKind::Scalar(RustScalarKind::F64);
    assert!(
        attach(RustSourceTypes::new(baseline.root(), functions, BTreeMap::new()).unwrap()).is_err()
    );
}
