//! Exact imports, original/consumer ownership and fail-closed projection.
use super::*;
use crate::{ast::*, dialect::*};
use portable_codegen::*;

#[path = "import_callbacks.rs"]
mod callbacks;

use crate::dialect::package_source_fixture as source_fixture;

fn dependency(scalar: CScalarType) -> (CDependencyApi, package_fixture::Fixture) {
    let (public, helper) = source_fixture::origins();
    let fixture = package_fixture::with_source_shape(
        CGeneratedOrigin::RustSource(Arc::new(public)),
        CGeneratedOrigin::RustSource(Arc::new(helper)),
        package_fixture::RecordLayout::Absent,
        scalar,
    );
    let ast = project_c_package(fixture.registry.clone(), fixture.files.clone()).unwrap();
    let checked = verify_unresolved_package(&CDialect, ast).unwrap();
    let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
    let certificate = certify_resolved_package(&CDialect, linked).unwrap();
    (
        CDependencyApi::from_certificate(certificate).unwrap(),
        fixture,
    )
}

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
        name: CIdentifier::new(name).unwrap(),
    }
}

fn file(registry: &mut CRegistry) -> CFileRef {
    registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("consumer.c").unwrap(),
            role: CFileRole::GeneratedSource,
        })
        .unwrap()
}

#[test]
fn scalar_imports_keep_exact_types_and_both_registration_authorities() {
    for scalar in [CScalarType::I32, CScalarType::Bool] {
        let (api, fixture) = dependency(scalar);
        let proof = api.functions().next().unwrap().clone();
        let mut registry = CRegistry::new();
        let original = proof.function().clone();
        let imported = registry.import_function(proof.clone()).unwrap();
        assert_ne!(imported, original);
        assert_eq!(imported.file(), original.file());
        assert_eq!(imported.key(), original.key());
        assert_eq!(imported.signature(), original.signature());
        assert_eq!(
            imported.contract().origin(),
            CCallableContractOrigin::CertifiedDependency
        );
        assert_eq!(registry.imported_function(&imported).unwrap(), &proof);
        assert!(registry.check_owned_function(&imported).is_err());
        assert!(registry.check_file(imported.file()).is_err());
        assert!(registry.check_function(&original).is_err());
        assert!(
            fixture
                .registry
                .registrations()
                .check_function(&imported)
                .is_err()
        );
        assert!(
            fixture
                .registry
                .registrations()
                .check_function(&original)
                .is_ok()
        );
        assert!(
            registry
                .check_callable_contract(imported.contract())
                .is_ok()
        );
        assert_eq!(registry.files().count(), 0);
        assert!(registry.inventory().is_empty());
        assert_eq!(registry.imported_functions().count(), 1);
        let expressions = CExpressions::new(&registry);
        let callable = expressions.direct(imported.clone()).unwrap();
        let i32_value = expressions
            .literal(CLiteral::Signed(CSignedLiteral::I32(7)))
            .unwrap();
        let bool_value = expressions.literal(CLiteral::Bool(true)).unwrap();
        let (right, wrong) = match scalar {
            CScalarType::I32 => (i32_value, bool_value),
            CScalarType::Bool => (bool_value, i32_value),
            _ => unreachable!(),
        };
        assert!(
            expressions
                .call_value(callable.clone(), vec![right])
                .is_ok()
        );
        assert!(
            expressions
                .call_value(callable.clone(), vec![wrong])
                .is_err()
        );
        assert!(expressions.call_value(callable, vec![]).is_err());
        drop(api);
        assert_eq!(registry.imported_function(&imported).unwrap(), &proof);
    }
}

#[test]
fn imported_functions_cannot_acquire_local_bodies_parameters_or_prototypes() {
    let (api, _) = dependency(CScalarType::I32);
    let mut registry = CRegistry::new();
    let file = file(&mut registry);
    let imported = registry
        .import_function(api.functions().next().unwrap().clone())
        .unwrap();
    assert!(
        registry
            .register_parameter(&imported, 0, key("x"), CConstness::Const)
            .is_err()
    );
    assert!(
        registry
            .register_scope(&imported, None, key("body"))
            .is_err()
    );
    assert!(CStatements::new(&registry, imported.clone()).is_err());
    assert!(
        CDeclarations::new(&registry, file.clone())
            .unwrap()
            .function_prototype(imported.clone(), CLinkage::External)
            .is_err()
    );
    assert!(
        registry
            .register_function(&file, imported.key().clone(), imported.signature().clone())
            .is_err()
    );
    let ordinary = registry
        .register_function(&file, key("ordinary"), imported.signature().clone())
        .unwrap();
    let scope = registry
        .register_scope(&ordinary, None, key("body"))
        .unwrap();
    let block = CStatements::new(&registry, ordinary)
        .unwrap()
        .block(scope, vec![])
        .unwrap();
    assert!(
        CDeclarations::new(&registry, file.clone())
            .unwrap()
            .function_definition(imported, CLinkage::External, vec![], block)
            .is_err()
    );
    let source = CDeclarations::new(&registry, file)
        .unwrap()
        .source_file(vec![])
        .unwrap();
    let error = project_c_package(registry.freeze(), vec![source]).unwrap_err();
    assert!(format!("{error:?}").contains("requires a function or scalar constant definition"));
}

#[test]
fn duplicate_conflicting_and_foreign_imports_reject_without_changing_inventory() {
    let (api, _) = dependency(CScalarType::I32);
    let (other_api, _) = dependency(CScalarType::Bool);
    let proof = api.functions().next().unwrap().clone();
    let other = other_api.functions().next().unwrap().clone();
    let mut registry = CRegistry::new();
    let imported = registry.import_function(proof.clone()).unwrap();
    assert!(registry.import_function(proof.clone()).is_err());
    assert!(registry.import_function(other).is_err());
    assert_eq!(registry.imported_functions().count(), 1);
    let mut foreign = CRegistry::new();
    let foreign_import = foreign.import_function(proof.clone()).unwrap();
    assert!(registry.check_function(&foreign_import).is_err());
    assert!(foreign.check_function(&imported).is_err());
    assert!(foreign.imported_function(&imported).is_err());
    assert!(
        registry
            .register_file(proof.public_header().file().key().clone())
            .is_err()
    );
    let mut collision = CRegistry::new();
    collision
        .register_file(proof.public_header().file().key().clone())
        .unwrap();
    assert!(collision.import_function(proof.clone()).is_err());
    assert_eq!(collision.imported_functions().count(), 0);
    let mut owned = CRegistry::new();
    let file = file(&mut owned);
    owned
        .register_function(
            &file,
            proof.function().key().clone(),
            proof.signature().clone(),
        )
        .unwrap();
    assert!(owned.import_function(proof).is_err());
}

#[derive(Clone, Copy, Debug)]
enum Mutation {
    WrongCertificate,
    Deleted,
    Signature,
    OriginalReference,
    PrivateReference,
    ContractOrigin,
    ContractFile,
    ContractBrand,
}

#[test]
fn coordinated_import_map_mutations_cannot_rebind_certificate_evidence() {
    for mutation in [
        Mutation::WrongCertificate,
        Mutation::Deleted,
        Mutation::Signature,
        Mutation::OriginalReference,
        Mutation::PrivateReference,
        Mutation::ContractOrigin,
        Mutation::ContractFile,
        Mutation::ContractBrand,
    ] {
        let (api, fixture) = dependency(CScalarType::I32);
        let mut proof = api.functions().next().unwrap().clone();
        let mut registry = CRegistry::new();
        let consumer_file = file(&mut registry);
        let original = registry.import_function(proof.clone()).unwrap();
        registry.imports.clear();
        let mut forged = original.clone();
        match mutation {
            Mutation::WrongCertificate => {
                let replacement = CDependencyApi::from_certificate(api.package().clone()).unwrap();
                proof = replacement.functions().next().unwrap().clone();
            }
            Mutation::Deleted => {}
            Mutation::Signature => {
                forged.signature = Arc::new(CFunctionType::new(CReturnType::Void, vec![]))
            }
            Mutation::OriginalReference => forged = proof.function().clone(),
            Mutation::PrivateReference => {
                forged.identity = Identity::new(&registry.scope, fixture.helper.key().clone());
                forged.file = fixture.helper.file().clone();
                forged.contract.identity = forged.identity.clone();
                forged.contract.file = forged.file.clone();
            }
            Mutation::ContractOrigin => {
                forged.contract.origin = CCallableContractOrigin::GeneratedBody
            }
            Mutation::ContractFile => forged.contract.file = consumer_file,
            Mutation::ContractBrand => forged.contract.identity.scope = CRegistry::new().scope,
        }
        if !matches!(mutation, Mutation::Deleted) {
            registry.imports.insert(forged.clone(), proof);
        }
        assert!(registry.check_function(&forged).is_err(), "{mutation:?}");
        assert!(
            CExpressions::new(&registry).direct(forged).is_err(),
            "{mutation:?}"
        );
    }
}

#[test]
fn independent_certificates_with_identical_registrations_do_not_share_proof_authority() {
    let (first, _) = dependency(CScalarType::I32);
    let second = CDependencyApi::from_certificate(first.package().clone()).unwrap();
    let first = first.functions().next().unwrap();
    let second = second.functions().next().unwrap();
    assert_eq!(first.function(), second.function());
    assert_ne!(first, second);
    assert!(!first.shares_certificate(second));
    assert_eq!(first, &first.clone());
}
