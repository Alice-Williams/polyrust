//! These inputs first certify as Java; the narrower owner API must still reject.
use super::source_dependency_fixture::*;
use crate::{ast::*, dialect::JavaDependencyApi};
use portable_codegen::{RustExportName, RustExportNamespace, RustExportTarget};

#[test]
fn missing_public_definition_cannot_be_hidden_by_a_complete_java_certificate() {
    let draft = package_with_exports(7, functions(42), |exports| {
        exports.modules.get_mut(&id(7, 1)).unwrap().insert(
            RustExportName {
                namespace: RustExportNamespace::Value,
                name: "missing".into(),
            },
            RustExportTarget::Declaration(id(7, 99)),
        );
    });
    assert!(
        JavaDependencyApi::from_certificate(certify(draft))
            .unwrap_err()
            .contains("omits a compiler public binding")
    );
}

#[test]
fn private_definition_cannot_be_promoted_by_export_metadata_alone() {
    let draft = package_with_exports(7, functions(42), |exports| {
        exports.modules.get_mut(&id(7, 1)).unwrap().insert(
            RustExportName {
                namespace: RustExportNamespace::Value,
                name: "private".into(),
            },
            RustExportTarget::Declaration(id(7, 13)),
        );
    });
    assert!(
        JavaDependencyApi::from_certificate(certify(draft))
            .unwrap_err()
            .contains("visibility/signature/declaration")
    );
}

#[test]
fn public_java_member_cannot_be_omitted_from_source_exports() {
    let draft = package_with_exports(7, functions(42), |exports| {
        exports
            .modules
            .get_mut(&id(7, 1))
            .unwrap()
            .retain(|_, target| *target != RustExportTarget::Declaration(id(7, 10)));
    });
    assert!(
        JavaDependencyApi::from_certificate(certify(draft))
            .unwrap_err()
            .contains("visibility/signature/declaration")
    );
}

#[test]
fn unsupported_public_category_rejects_instead_of_being_silently_omitted() {
    let draft = package_with_exports(7, functions(42), |exports| {
        exports.modules.get_mut(&id(7, 1)).unwrap().insert(
            RustExportName {
                namespace: RustExportNamespace::Type,
                name: "UnmappedType".into(),
            },
            RustExportTarget::Declaration(id(7, 99)),
        );
    });
    assert!(
        JavaDependencyApi::from_certificate(certify(draft))
            .unwrap_err()
            .contains("no supported local scalar function/constant mapping")
    );
}

#[test]
fn character_export_does_not_acquire_an_integer_dependency_handle() {
    let mut fixture = functions(42);
    fixture[0].result = JavaType::primitive(JavaPrimitive::Char);
    fixture[0].parameters = vec![JavaParameter {
        ty: JavaType::primitive(JavaPrimitive::Char),
        name: name("character"),
        final_parameter: true,
    }];
    fixture[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::local(
        JavaType::primitive(JavaPrimitive::Char),
        name("character"),
    )))]);
    assert!(
        JavaDependencyApi::from_certificate(certify(package(7, fixture)))
            .unwrap_err()
            .contains("visibility/signature/declaration")
    );
}

#[test]
fn same_member_names_across_owners_remain_distinct_typed_paths() {
    let first = JavaDependencyApi::from_certificate(certify(package(7, functions(42)))).unwrap();
    let second = JavaDependencyApi::from_certificate(certify(package(8, functions(42)))).unwrap();
    let left = first.function(id(7, 10)).unwrap();
    let right = second.function(id(8, 10)).unwrap();
    assert_eq!(left.path().member(), right.path().member());
    assert_eq!(left.path().owners(), right.path().owners());
    assert_ne!(left.path().package(), right.path().package());
    assert_ne!(left, right);
}
