//! Independently inspect compiler-to-certified public field and callable identities.
use crate::source_capabilities::{ConstantDeclarationInput, ScalarConstantValue};
use crate::source_origin::{
    Cache,
    public_api::{DeclarationKind, Inventory},
};
use portable_backend_java::{
    ast::*,
    dialect::{JavaDependencyApi, JavaDialect},
};
use portable_codegen::{RenderReadyPackage, RustExportTarget};
use rustc_middle::ty::TyCtxt;

pub(super) fn check(tcx: TyCtxt<'_>, certificate: &RenderReadyPackage<JavaDialect>) {
    let inventory = Inventory::read(tcx, &mut Cache::default()).unwrap();
    let api = JavaDependencyApi::from_certificate(certificate.clone()).unwrap();
    let mut constants = 0;
    for (id, declaration) in inventory.declarations() {
        let (kind, path) = match declaration.kind() {
            DeclarationKind::Constant => {
                constants += 1;
                let input = ConstantDeclarationInput::read(
                    tcx,
                    &inventory,
                    declaration.definition().to_def_id(),
                )
                .unwrap();
                let constant = api.constant(*id).unwrap();
                let (ty, value) = match input.value() {
                    ScalarConstantValue::Bool(v) => {
                        (JavaPrimitive::Boolean, JavaLiteral::Boolean(v))
                    }
                    ScalarConstantValue::I32(v) => (JavaPrimitive::Int, JavaLiteral::I32(v)),
                    ScalarConstantValue::I64(v) => (JavaPrimitive::Long, JavaLiteral::I64(v)),
                    ScalarConstantValue::F64(v) => (JavaPrimitive::Double, JavaLiteral::F64(v)),
                };
                assert_eq!(constant.value().literal(), Some(value));
                assert_eq!(constant.ty(), &JavaType::primitive(ty));
                assert_eq!(constant.source().declaration, *id);
                assert!(constant.source().externally_reachable);
                assert_eq!(constant.package_identity(), api.package_identity());
                ("constant", constant.path())
            }
            DeclarationKind::Function => {
                assert!(
                    ConstantDeclarationInput::read(
                        tcx,
                        &inventory,
                        declaration.definition().to_def_id()
                    )
                    .is_err()
                );
                ("function", api.function(*id).unwrap().path())
            }
        };
        let package = path.package().name();
        let full = std::iter::once(package.as_ref())
            .chain(path.owners().iter().map(|x| x.as_str()))
            .chain(std::iter::once(path.member().as_str()))
            .collect::<Vec<_>>()
            .join(".");
        println!(
            "TARGET\t{kind}\t{:016x}:{:016x}\t{full}",
            id.crate_id, id.definition_path_hash
        );
    }
    assert_eq!(api.constants().len(), constants);
    for (module, bindings) in &inventory.exports().modules {
        for (name, target) in bindings {
            if let RustExportTarget::Declaration(id) = target {
                println!(
                    "BIND\t{:016x}:{:016x}\t{}\t{:016x}:{:016x}",
                    module.crate_id,
                    module.definition_path_hash,
                    name.name,
                    id.crate_id,
                    id.definition_path_hash
                );
            }
        }
    }
    println!("PUBLIC_CONSTANT_CERTIFIED\t{constants}");
}
