//! Definition placement cannot discharge another file's declaration obligation.
use super::contextual_reconstruction::key as test_key;
use super::*;

fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::PlatformAssertion),
        ..test_key(name)
    }
}
#[derive(Clone, Copy, Debug)]
enum Subject {
    Function,
    Object,
    Struct,
    Union,
    IncompleteTag,
}

#[test]
fn split_definition_requires_a_declaration_in_the_registered_owner_file() {
    for subject in [
        Subject::Function,
        Subject::Object,
        Subject::Struct,
        Subject::Union,
        Subject::IncompleteTag,
    ] {
        for include_owner in [false, true] {
            let mut registry = CRegistry::new();
            let header = registry
                .register_file(CFileKey {
                    path: portable_codegen::RelativeOutputPath::new("public.h").unwrap(),
                    role: CFileRole::GeneratedPublicHeader,
                })
                .unwrap();
            let implementation = registry
                .register_file(CFileKey {
                    path: portable_codegen::RelativeOutputPath::new("implementation.c").unwrap(),
                    role: CFileRole::GeneratedSource,
                })
                .unwrap();
            let (owner_item, implementation_item) = match subject {
                Subject::Function => {
                    let function = registry
                        .register_function(
                            &header,
                            key("run"),
                            CFunctionType::new(CReturnType::Void, vec![]),
                        )
                        .unwrap();
                    let scope = registry
                        .register_scope(&function, None, key("root"))
                        .unwrap();
                    let body = CStatements::new(&registry, function.clone())
                        .unwrap()
                        .block(scope, vec![])
                        .unwrap();
                    (
                        CFileItem::Declaration(
                            CDeclarations::new(&registry, header.clone())
                                .unwrap()
                                .function_prototype(function.clone(), CLinkage::External)
                                .unwrap(),
                        ),
                        CFileItem::Definition(
                            CDeclarations::new(&registry, implementation.clone())
                                .unwrap()
                                .function_definition(function, CLinkage::External, vec![], body)
                                .unwrap(),
                        ),
                    )
                }
                Subject::Object => {
                    let ty = CObjectType::scalar(CScalarType::I32);
                    let object = registry
                        .register_object(&header, key("value"), ty.clone())
                        .unwrap();
                    (
                        CFileItem::Declaration(
                            CDeclarations::new(&registry, header.clone())
                                .unwrap()
                                .object_declaration(object.clone())
                                .unwrap(),
                        ),
                        CFileItem::Definition(
                            CDeclarations::new(&registry, implementation.clone())
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
                Subject::Struct | Subject::Union | Subject::IncompleteTag => {
                    let owner = if matches!(subject, Subject::Union) {
                        CAggregateRef::Union(
                            registry.declare_union(&header, key("Payload")).unwrap(),
                        )
                    } else {
                        CAggregateRef::Struct(
                            registry.declare_struct(&header, key("Record")).unwrap(),
                        )
                    };
                    if !matches!(subject, Subject::IncompleteTag) {
                        let member = registry
                            .register_member(
                                &owner,
                                key("field"),
                                CObjectType::scalar(CScalarType::I32),
                            )
                            .unwrap();
                        registry.define_aggregate(&owner, vec![member]).unwrap();
                    }
                    let declaration = CDeclarations::new(&registry, header.clone())
                        .unwrap()
                        .forward_tag(owner.clone())
                        .unwrap();
                    let actual = CDeclarations::new(&registry, implementation.clone()).unwrap();
                    let moved = if matches!(subject, Subject::IncompleteTag) {
                        actual.forward_tag(owner).unwrap()
                    } else {
                        actual.aggregate(owner).unwrap()
                    };
                    (
                        CFileItem::Declaration(declaration),
                        CFileItem::Declaration(moved),
                    )
                }
            };
            let files = [
                CDeclarations::new(&registry, header)
                    .unwrap()
                    .source_file(if include_owner {
                        vec![owner_item]
                    } else {
                        vec![]
                    })
                    .unwrap(),
                CDeclarations::new(&registry, implementation)
                    .unwrap()
                    .source_file(vec![implementation_item])
                    .unwrap(),
            ];
            assert_eq!(
                registry.check_context(&files),
                if include_owner {
                    Ok(())
                } else {
                    Err(CContextError::MissingRegistrationOccurrence)
                },
                "{subject:?}/owner={include_owner}"
            );
        }
    }
}
