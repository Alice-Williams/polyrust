//! Actual definition-family ownership and same-registry typedef relocation.
use super::contextual_reconstruction::key as neutral_key;
use super::*;

fn key(name: &str, origin: CGeneratedOrigin) -> CDeclarationKey {
    CDeclarationKey {
        origin,
        ..neutral_key(name)
    }
}
fn neutral() -> CGeneratedOrigin {
    CGeneratedOrigin::Synthesized(CSynthesisReason::PlatformAssertion)
}
fn file(registry: &mut CRegistry, path: &str, role: CFileRole) -> CFileRef {
    registry
        .register_file(CFileKey {
            path: portable_codegen::RelativeOutputPath::new(path).unwrap(),
            role,
        })
        .unwrap()
}
#[derive(Clone, Copy, Debug)]
enum Subject {
    Object,
    Struct,
    Union,
    StructMember,
    UnionMember,
}

#[test]
fn actual_object_aggregate_and_member_definitions_retain_their_origin_family() {
    let core = super::contextual_origins::core();
    let production = CGeneratedOrigin::CoreDeclaration(
        *core
            .module()
            .declarations
            .iter()
            .find(|d| matches!(d, portable_core_ir::CoreDeclaration::Function(_)))
            .unwrap(),
    );
    for subject in [
        Subject::Object,
        Subject::Struct,
        Subject::Union,
        Subject::StructMember,
        Subject::UnionMember,
    ] {
        for runtime in [false, true] {
            for matching in [false, true] {
                let mut registry = CRegistry::new();
                let header = file(&mut registry, "private.h", CFileRole::PrivateHeader);
                let source = file(
                    &mut registry,
                    "body.c",
                    if runtime == matching {
                        CFileRole::RuntimeSource
                    } else {
                        CFileRole::GeneratedSource
                    },
                );
                let origin = if runtime {
                    CGeneratedOrigin::Synthesized(CSynthesisReason::Runtime)
                } else {
                    production.clone()
                };
                let (declaration, definition) = match subject {
                    Subject::Object => {
                        let ty = CObjectType::scalar(CScalarType::I32);
                        let object = registry
                            .register_object(&header, key("storage", origin), ty.clone())
                            .unwrap();
                        (
                            CDeclarations::new(&registry, header.clone())
                                .unwrap()
                                .object_declaration(object.clone())
                                .unwrap(),
                            CFileItem::Definition(
                                CDeclarations::new(&registry, source.clone())
                                    .unwrap()
                                    .object_definition(
                                        object,
                                        CLinkage::External,
                                        CExpressions::new(&registry).zero_initializer(ty).unwrap(),
                                    )
                                    .unwrap(),
                            ),
                        )
                    }
                    Subject::Struct
                    | Subject::Union
                    | Subject::StructMember
                    | Subject::UnionMember => {
                        let member_case =
                            matches!(subject, Subject::StructMember | Subject::UnionMember);
                        let owner_key = key(
                            "Record",
                            if member_case {
                                neutral()
                            } else {
                                origin.clone()
                            },
                        );
                        let owner = if matches!(subject, Subject::Union | Subject::UnionMember) {
                            CAggregateRef::Union(
                                registry.declare_union(&header, owner_key).unwrap(),
                            )
                        } else {
                            CAggregateRef::Struct(
                                registry.declare_struct(&header, owner_key).unwrap(),
                            )
                        };
                        let member = registry
                            .register_member(
                                &owner,
                                key("field", if member_case { origin } else { neutral() }),
                                CObjectType::scalar(CScalarType::I32),
                            )
                            .unwrap();
                        registry.define_aggregate(&owner, vec![member]).unwrap();
                        (
                            CDeclarations::new(&registry, header.clone())
                                .unwrap()
                                .forward_tag(owner.clone())
                                .unwrap(),
                            CFileItem::Declaration(
                                CDeclarations::new(&registry, source.clone())
                                    .unwrap()
                                    .aggregate(owner)
                                    .unwrap(),
                            ),
                        )
                    }
                };
                let files = [
                    CDeclarations::new(&registry, header)
                        .unwrap()
                        .source_file(vec![CFileItem::Declaration(declaration)])
                        .unwrap(),
                    CDeclarations::new(&registry, source)
                        .unwrap()
                        .source_file(vec![definition])
                        .unwrap(),
                ];
                registry.check_local_structure(&files).unwrap();
                assert_eq!(
                    registry.check_context(&files),
                    if matching {
                        Ok(())
                    } else {
                        Err(CContextError::OriginRoleMismatch)
                    },
                    "{subject:?}, runtime={runtime}, matching={matching}"
                );
            }
        }
    }
}

#[test]
fn same_registry_typedef_relocation_is_rejected_before_origin_checking() {
    for role in [
        CFileRole::GeneratedSource,
        CFileRole::RuntimeSource,
        CFileRole::TestSource,
    ] {
        let mut registry = CRegistry::new();
        let header = file(&mut registry, "private.h", CFileRole::PrivateHeader);
        let source = file(&mut registry, "body.c", role);
        let alias = registry
            .register_typedef(
                &header,
                key("Alias", neutral()),
                CObjectType::scalar(CScalarType::I32),
            )
            .unwrap();
        let h = CDeclarations::new(&registry, header).unwrap();
        let c = CDeclarations::new(&registry, source.clone()).unwrap();
        let declaration = h.typedef(alias).unwrap();
        let valid = [
            h.source_file(vec![CFileItem::Declaration(declaration.clone())])
                .unwrap(),
            c.source_file(vec![]).unwrap(),
        ];
        registry.check_context(&valid).unwrap();
        let mut moved = declaration;
        moved.file = source;
        let invalid = [
            h.source_file(vec![]).unwrap(),
            c.source_file(vec![CFileItem::Declaration(moved)]).unwrap(),
        ];
        assert_eq!(
            registry.check_context(&invalid),
            Err(CContextError::File(CFileError::WrongFile))
        );
    }
}
