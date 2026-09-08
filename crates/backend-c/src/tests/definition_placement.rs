//! Full source-role/linkage matrix for both function and object definitions.
use super::{declarations::source_file, registry_nominals::key};
use crate::ast::{
    CDeclarations, CDefinitionKind, CExpressions, CFileError as E, CFileRole as R, CFunctionType,
    CLinkage as L, CObjectType, CRegistry, CReturnType, CScalarType, CStatements,
};
#[test]
fn every_definition_role_and_linkage_combination_is_checked() {
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
            for same_file in [false, true] {
                if same_file && origin_role != target_role {
                    continue;
                }
                let mut registry = CRegistry::new();
                let origin = source_file(&mut registry, "origin.h", origin_role);
                let target = if same_file {
                    origin.clone()
                } else {
                    source_file(&mut registry, "target.c", target_role)
                };
                let function = registry
                    .register_function(
                        &origin,
                        key("run"),
                        CFunctionType::new(CReturnType::Void, vec![]),
                    )
                    .unwrap();
                let object = registry
                    .register_object(&origin, key("value"), CObjectType::scalar(CScalarType::I32))
                    .unwrap();
                let scope = registry
                    .register_scope(&function, None, key("root"))
                    .unwrap();
                let statements = CStatements::new(&registry, function.clone()).unwrap();
                let body = statements
                    .block(scope, vec![statements.return_statement(None).unwrap()])
                    .unwrap();
                let initializer = CExpressions::new(&registry)
                    .zero_initializer(object.ty().clone())
                    .unwrap();
                let declarations = CDeclarations::new(&registry, target.clone()).unwrap();
                let legal_role = match origin_role {
                    R::GeneratedPublicHeader => target_role == R::GeneratedSource,
                    R::RuntimePublicHeader => target_role == R::RuntimeSource,
                    R::PrivateHeader => {
                        matches!(target_role, R::GeneratedSource | R::RuntimeSource)
                    }
                    R::GeneratedSource | R::RuntimeSource | R::TestSource => same_file,
                };
                for linkage in [L::External, L::Internal, L::None] {
                    let invalid_linkage = linkage == L::None
                        || (linkage == L::Internal
                            && matches!(
                                origin_role,
                                R::GeneratedPublicHeader | R::RuntimePublicHeader
                            ));
                    let error = if !legal_role {
                        Some(E::WrongFileRole)
                    } else if invalid_linkage {
                        Some(E::InvalidLinkage)
                    } else {
                        None
                    };
                    let function_result = declarations.function_definition(
                        function.clone(),
                        linkage,
                        vec![],
                        body.clone(),
                    );
                    let object_result = declarations.object_definition(
                        object.clone(),
                        linkage,
                        initializer.clone(),
                    );
                    if let Some(error) = error {
                        assert_eq!(function_result, Err(error));
                        assert_eq!(object_result, Err(error));
                    } else {
                        let actual = function_result.unwrap();
                        assert_eq!(actual.file(), &target);
                        assert_eq!(
                            actual.kind(),
                            &CDefinitionKind::Function {
                                function: function.clone(),
                                linkage,
                                parameters: vec![],
                                body: Box::new(body.clone()),
                            }
                        );
                        let actual = object_result.unwrap();
                        assert_eq!(actual.file(), &target);
                        assert_eq!(
                            actual.kind(),
                            &CDefinitionKind::Object {
                                object: object.clone(),
                                linkage,
                                initializer: Box::new(initializer.clone()),
                            }
                        );
                    }
                }
            }
        }
    }
}
