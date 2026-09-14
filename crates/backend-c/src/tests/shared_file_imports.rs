//! Header witnesses authenticate identity before accepting include spelling.
use super::*;
use crate::ast::CFileKey;
use portable_codegen::RelativeOutputPath;
use std::collections::BTreeSet;

fn registered(registry: &mut CRegistry, path: &str, role: CFileRole) -> CFileRef {
    registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(path).unwrap(),
            role,
        })
        .unwrap()
}

#[test]
fn header_witness_retains_exact_identity_path_and_guard() {
    let mut registry = CRegistry::new();
    let source = registered(&mut registry, "out/unit.c", CFileRole::GeneratedSource);
    let header = registered(
        &mut registry,
        "out/polyrust_api.h",
        CFileRole::GeneratedPublicHeader,
    );
    let witness = CGeneratedHeader::resolve(&registry, &source, &header).unwrap();
    assert_eq!(witness.file(), &header);
    assert_eq!(witness.include_path(), "polyrust_api.h");
    assert_eq!(witness.guard().file(), &header);
    assert_eq!(
        witness.guard().identifier().as_str(),
        "POLYRUST_HEADER_6F75742F706F6C79727573745F6170692E68_INCLUDED"
    );
    assert_eq!(
        witness,
        CGeneratedHeader::resolve(&registry, &source, &header).unwrap()
    );
}

#[test]
fn guard_encoding_distinguishes_case_punctuation_and_directory_identity() {
    let mut registry = CRegistry::new();
    let mut guards = BTreeSet::new();
    for directory in ["first", "second"] {
        let source = registered(
            &mut registry,
            &format!("{directory}/unit.c"),
            CFileRole::GeneratedSource,
        );
        for name in ["a.h", "A.h", "a-b.h", "a_b.h", "a.b.h", "ab.h"] {
            let header = registered(
                &mut registry,
                &format!("{directory}/polyrust_{name}"),
                CFileRole::GeneratedPublicHeader,
            );
            let witness = CGeneratedHeader::resolve(&registry, &source, &header).unwrap();
            assert!(guards.insert(witness.guard().identifier().clone()));
            assert!(!witness.guard().identifier().as_str().starts_with("poly_"));
        }
    }
    assert_eq!(guards.len(), 12);
}

#[test]
fn foreign_identity_roles_self_import_and_non_sibling_paths_are_rejected() {
    let mut registry = CRegistry::new();
    let source = registered(&mut registry, "out/unit.c", CFileRole::GeneratedSource);
    let header = registered(
        &mut registry,
        "out/polyrust_api.h",
        CFileRole::GeneratedPublicHeader,
    );
    let mut foreign = CRegistry::new();
    let foreign_header = registered(
        &mut foreign,
        "out/polyrust_api.h",
        CFileRole::GeneratedPublicHeader,
    );
    let foreign_source = registered(&mut foreign, "out/unit.c", CFileRole::GeneratedSource);
    assert!(CGeneratedHeader::resolve(&registry, &source, &foreign_header).is_err());
    assert!(CGeneratedHeader::resolve(&registry, &foreign_source, &header).is_err());
    assert!(CGeneratedHeader::resolve(&registry, &header, &header).is_err());
    for (index, role) in [
        CFileRole::GeneratedSource,
        CFileRole::RuntimeSource,
        CFileRole::TestSource,
    ]
    .into_iter()
    .enumerate()
    {
        let target = registered(&mut registry, &format!("out/not_header{index}.h"), role);
        assert!(CGeneratedHeader::resolve(&registry, &source, &target).is_err());
    }
    let target = registered(
        &mut registry,
        "elsewhere/polyrust_api.h",
        CFileRole::PrivateHeader,
    );
    assert!(CGeneratedHeader::resolve(&registry, &source, &target).is_err());
    for (index, role) in [CFileRole::PrivateHeader, CFileRole::RuntimePublicHeader]
        .into_iter()
        .enumerate()
    {
        let target = registered(
            &mut registry,
            &format!("out/polyrust_allowed{index}.h"),
            role,
        );
        assert!(CGeneratedHeader::resolve(&registry, &source, &target).is_ok());
    }
}

#[test]
fn injection_and_unimplemented_include_forms_never_receive_a_witness() {
    let mut registry = CRegistry::new();
    let source = registered(&mut registry, "out/unit.c", CFileRole::GeneratedSource);
    for name in [
        "a\".h",
        "a\n.h",
        "a\r.h",
        "a\\b.h",
        "a b.h",
        "a\t.h",
        "a>.h",
        "a<.h",
        "a?.h",
        "é.h",
        "a.c",
        ".h",
        "stdint.h",
        "features.h",
        "api.h",
        "polyrust_.h",
        "Polyrust_api.h",
    ] {
        let Ok(path) = RelativeOutputPath::new(format!("out/{name}")) else {
            continue; // Rejected even earlier at the shared path boundary.
        };
        let header = registry
            .register_file(CFileKey {
                path,
                role: CFileRole::GeneratedPublicHeader,
            })
            .unwrap();
        assert!(
            CGeneratedHeader::resolve(&registry, &source, &header).is_err(),
            "{name:?}"
        );
    }
}

#[test]
fn guard_identifier_budget_is_checked_after_injective_encoding() {
    let mut registry = CRegistry::new();
    let source = registered(&mut registry, "unit.c", CFileRole::GeneratedSource);
    // Prefix + suffix use 25 bytes: 115 path bytes yield 255 guard bytes;
    // the next path byte yields 257 and exceeds the 256-byte policy.
    let admitted = registered(
        &mut registry,
        &format!("polyrust_{}.h", "a".repeat(104)),
        CFileRole::GeneratedPublicHeader,
    );
    assert_eq!(
        CGeneratedHeader::resolve(&registry, &source, &admitted)
            .unwrap()
            .guard()
            .identifier()
            .as_str()
            .len(),
        255
    );
    let rejected = registered(
        &mut registry,
        &format!("polyrust_{}.h", "a".repeat(105)),
        CFileRole::GeneratedPublicHeader,
    );
    assert!(CGeneratedHeader::resolve(&registry, &source, &rejected).is_err());
}

#[test]
fn typed_header_vocabulary_does_not_admit_empty_or_standalone_headers() {
    use crate::{ast::CDeclarations, dialect::project_c_package};
    let mut registry = CRegistry::new();
    let source = registered(&mut registry, "unit.c", CFileRole::GeneratedSource);
    let header = registered(
        &mut registry,
        "polyrust_api.h",
        CFileRole::GeneratedPublicHeader,
    );
    let witness = CGeneratedHeader::resolve(&registry, &source, &header).unwrap();
    assert_eq!(witness.file(), &header);
    let files = [&header, &source]
        .into_iter()
        .map(|file| {
            CDeclarations::new(&registry, file.clone())
                .unwrap()
                .source_file(vec![])
                .unwrap()
        })
        .collect();
    assert!(
        project_c_package(registry.freeze(), files)
            .unwrap_err()
            .iter()
            .any(|error| error.message.contains("requires an exported function"))
    );

    let mut registry = CRegistry::new();
    let header = registered(
        &mut registry,
        "polyrust_api.h",
        CFileRole::GeneratedPublicHeader,
    );
    let file = CDeclarations::new(&registry, header)
        .unwrap()
        .source_file(vec![])
        .unwrap();
    assert!(project_c_package(registry.freeze(), vec![file]).is_err());
}
