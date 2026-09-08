//! Registry-derived declarations, source roles and exact function definitions.

use super::registry_nominals::{key, registry};
use crate::ast::{
    CAggregateRef, CConstness, CDeclarationKind, CDeclarations, CExpressions, CFileError as E,
    CFileKey, CFileRef, CFileRole, CFunctionType, CLinkage, CObjectType, CParameterType, CRegistry,
    CRegistryError, CReturnType, CReturnValue, CScalarType, CStatements, CStorage,
};
use portable_codegen::RelativeOutputPath;

pub(super) fn source_file(registry: &mut CRegistry, name: &str, role: CFileRole) -> CFileRef {
    registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new(name).unwrap(),
            role,
        })
        .unwrap()
}
fn scalar() -> CObjectType {
    CObjectType::scalar(CScalarType::I32)
}

#[test]
fn nominal_declarations_derive_the_complete_inventory_and_owning_file() {
    let (mut registry, header) = registry();
    let source = source_file(&mut registry, "src/generated.c", CFileRole::GeneratedSource);
    let record = registry.declare_struct(&header, key("Record")).unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    assert!(
        CDeclarations::new(&registry, source.clone())
            .unwrap()
            .forward_tag(owner.clone())
            .is_ok()
    );
    assert_eq!(
        CDeclarations::new(&registry, header.clone())
            .unwrap()
            .aggregate(owner.clone()),
        Err(E::IncompleteDefinition)
    );
    let member = registry
        .register_member(&owner, key("value"), scalar())
        .unwrap();
    registry
        .define_aggregate(&owner, vec![member.clone()])
        .unwrap();
    let enumeration = registry.declare_enum(&header, key("Status")).unwrap();
    let value = registry
        .register_enumerator(&enumeration, key("Success"), 0)
        .unwrap();
    registry
        .define_enum(&enumeration, vec![value.clone()])
        .unwrap();
    let alias = registry
        .register_typedef(&header, key("Alias"), CObjectType::structure(record))
        .unwrap();
    let declarations = CDeclarations::new(&registry, header.clone()).unwrap();
    let aggregate = declarations.aggregate(owner.clone()).unwrap();
    assert_eq!(
        aggregate.kind(),
        &CDeclarationKind::Aggregate {
            owner: owner.clone(),
            members: vec![member]
        }
    );
    assert_eq!(aggregate.linkage(), CLinkage::None);
    assert_eq!(aggregate.storage(), None);
    assert_eq!(
        declarations
            .enumeration(enumeration.clone())
            .unwrap()
            .kind(),
        &CDeclarationKind::Enum {
            owner: enumeration,
            values: vec![value]
        }
    );
    assert!(declarations.typedef(alias.clone()).is_ok());
    let declarations = CDeclarations::new(&registry, source.clone()).unwrap();
    let private_definition = declarations.aggregate(owner.clone()).unwrap();
    assert_eq!(private_definition.file(), &source);
    assert!(matches!(private_definition.kind(),
        CDeclarationKind::Aggregate { owner: actual, .. } if actual == &owner));
    assert_eq!(declarations.typedef(alias), Err(E::WrongFile));
    let (foreign, _) = super::registry_nominals::registry();
    assert!(matches!(
        CDeclarations::new(&foreign, header),
        Err(E::Registry(CRegistryError::CrossRegistry))
    ));
}

#[test]
fn function_definitions_require_exact_parameters_root_scope_and_source_role() {
    let (mut registry, header) = registry();
    let source = source_file(&mut registry, "src/generated.c", CFileRole::GeneratedSource);
    let test = source_file(&mut registry, "tests/test.c", CFileRole::TestSource);
    let signature = CFunctionType::new(
        CReturnType::Void,
        vec![CParameterType::new(scalar()).unwrap(); 2],
    );
    let function = registry
        .register_function(&header, key("run"), signature)
        .unwrap();
    let parameters = (0..2)
        .map(|index| {
            registry
                .register_parameter(
                    &function,
                    index,
                    key(&format!("arg{index}")),
                    CConstness::Unqualified,
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    let scope = registry
        .register_scope(&function, None, key("root"))
        .unwrap();
    let child = registry
        .register_scope(&function, Some(&scope), key("child"))
        .unwrap();
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let body = statements
        .block(scope, vec![statements.return_statement(None).unwrap()])
        .unwrap();
    let declaration = CDeclarations::new(&registry, header.clone()).unwrap();
    assert_eq!(
        declaration.function_prototype(function.clone(), CLinkage::None),
        Err(E::InvalidLinkage)
    );
    assert_eq!(
        declaration.function_prototype(function.clone(), CLinkage::Internal),
        Err(E::InvalidLinkage)
    );
    assert_eq!(
        declaration
            .function_prototype(function.clone(), CLinkage::External)
            .unwrap()
            .storage(),
        Some(CStorage::Extern)
    );
    assert_eq!(
        declaration.function_definition(
            function.clone(),
            CLinkage::External,
            parameters.clone(),
            body.clone()
        ),
        Err(E::WrongFileRole)
    );
    let declaration = CDeclarations::new(&registry, source).unwrap();
    let definition = declaration
        .function_definition(
            function.clone(),
            CLinkage::External,
            parameters.clone(),
            body.clone(),
        )
        .unwrap();
    assert_eq!(definition.storage(), None);
    for bad in [
        vec![],
        vec![parameters[1].clone(), parameters[0].clone()],
        vec![parameters[0].clone(); 2],
    ] {
        assert_eq!(
            declaration.function_definition(
                function.clone(),
                CLinkage::External,
                bad,
                body.clone()
            ),
            Err(E::ParameterInventory)
        );
    }
    assert_eq!(
        declaration.function_definition(
            function.clone(),
            CLinkage::External,
            parameters.clone(),
            statements.block(child, vec![]).unwrap()
        ),
        Err(E::InvalidFunctionRoot)
    );
    assert_eq!(
        CDeclarations::new(&registry, test)
            .unwrap()
            .function_definition(function, CLinkage::External, parameters, body),
        Err(E::WrongFileRole)
    );
}

#[test]
fn object_definitions_have_initializers_and_derive_valid_storage() {
    let (mut registry, header) = registry();
    let source = source_file(&mut registry, "src/generated.c", CFileRole::GeneratedSource);
    let global = registry
        .register_object(&header, key("global"), scalar())
        .unwrap();
    let internal = registry
        .register_object(&source, key("internal"), scalar())
        .unwrap();
    let function = registry
        .register_function(
            &header,
            key("value"),
            CFunctionType::new(
                CReturnType::Value(CReturnValue::new(scalar()).unwrap()),
                vec![],
            ),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let initializer = ast.zero_initializer(scalar()).unwrap();
    let header_declarations = CDeclarations::new(&registry, header).unwrap();
    assert_eq!(
        header_declarations
            .object_declaration(global.clone())
            .unwrap()
            .storage(),
        Some(CStorage::Extern)
    );
    let declarations = CDeclarations::new(&registry, source).unwrap();
    assert!(
        declarations
            .object_definition(global.clone(), CLinkage::External, initializer.clone())
            .is_ok()
    );
    assert_eq!(
        declarations.object_definition(global.clone(), CLinkage::Internal, initializer.clone()),
        Err(E::InvalidLinkage)
    );
    assert_eq!(
        declarations
            .object_definition(internal, CLinkage::Internal, initializer)
            .unwrap()
            .storage(),
        Some(CStorage::Static)
    );
    let call = ast
        .call_value(ast.direct(function).unwrap(), vec![])
        .unwrap();
    assert_eq!(
        declarations.object_definition(
            global,
            CLinkage::External,
            ast.expression_initializer(call).unwrap()
        ),
        Err(E::ExpectedStaticInitializer)
    );
}
