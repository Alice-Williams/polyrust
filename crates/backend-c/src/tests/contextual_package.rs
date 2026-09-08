//! Registration coverage and complete-object graph controls.

use super::contextual_reconstruction::{fixture, key, package};
use super::*;

fn scalar() -> CObjectType {
    CObjectType::scalar(CScalarType::I32)
}

#[test]
fn package_inventory_rejects_missing_and_duplicated_function_or_file() {
    let (registry, file, function, scope) = fixture();
    let valid = package(&registry, file, function, scope, vec![]);
    registry
        .check_package_structure(std::slice::from_ref(&valid))
        .unwrap();
    assert_eq!(
        registry.check_package_structure(&[]),
        Err(CContextError::MissingRegistrationOccurrence)
    );
    assert_eq!(
        registry.check_package_structure(&[valid.clone(), valid.clone()]),
        Err(CContextError::DuplicateOccurrence)
    );
    let mut missing = valid.clone();
    missing.items.clear();
    assert_eq!(
        registry.check_package_structure(&[missing]),
        Err(CContextError::MissingRegistrationOccurrence)
    );
    let mut duplicate = valid;
    duplicate.items.push(duplicate.items[0].clone());
    assert_eq!(
        registry.check_package_structure(&[duplicate]),
        Err(CContextError::DuplicateOccurrence)
    );
}

#[test]
fn deleting_scope_and_local_together_cannot_delete_registration_obligations() {
    let (mut registry, file, function, scope) = fixture();
    let child = registry
        .register_scope(&function, Some(&scope), key("child"))
        .unwrap();
    let local = registry
        .register_local(&child, key("value"), scalar())
        .unwrap();
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let block = statements
        .block(child, vec![statements.declare(local, None).unwrap()])
        .unwrap();
    let valid = package(
        &registry,
        file,
        function,
        scope,
        vec![statements.nested_block(block).unwrap()],
    );
    registry
        .check_package_structure(std::slice::from_ref(&valid))
        .unwrap();
    let mut missing = valid;
    if let CFileItem::Definition(value) = &mut missing.items[0] {
        if let CDefinitionKind::Function { body, .. } = &mut value.kind {
            body.statements.clear();
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    }
    assert_eq!(
        registry.check_package_structure(&[missing]),
        Err(CContextError::MissingRegistrationOccurrence)
    );
}

#[test]
fn incomplete_nominal_pointers_are_allowed() {
    let (mut registry, file, function, scope) = fixture();
    let record = registry.declare_struct(&file, key("Opaque")).unwrap();
    let pointer = CObjectType::pointer(CPointerTarget::Object(Box::new(CObjectType::structure(
        record.clone(),
    ))));
    let local = registry
        .register_local(&scope, key("pointer"), pointer)
        .unwrap();
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let mut valid = package(
        &registry,
        file.clone(),
        function.clone(),
        scope.clone(),
        vec![statements.declare(local, None).unwrap()],
    );
    let declarations = CDeclarations::new(&registry, file.clone()).unwrap();
    valid.items.insert(
        0,
        CFileItem::Declaration(
            declarations
                .forward_tag(CAggregateRef::Struct(record.clone()))
                .unwrap(),
        ),
    );
    registry.check_package_structure(&[valid]).unwrap();
}

fn aggregate_package(recursive_pointer: bool) -> (CRegistry, CSourceFile) {
    let (mut registry, file, function, scope) = fixture();
    let record = registry.declare_struct(&file, key("Node")).unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let ty = CObjectType::structure(record);
    let member_type = if recursive_pointer {
        CObjectType::pointer(CPointerTarget::Object(Box::new(ty)))
    } else {
        ty
    };
    let member = registry
        .register_member(&owner, key("next"), member_type)
        .unwrap();
    registry.define_aggregate(&owner, vec![member]).unwrap();
    let mut source = package(&registry, file.clone(), function, scope, vec![]);
    source.items.insert(
        0,
        CFileItem::Declaration(
            CDeclarations::new(&registry, file)
                .unwrap()
                .aggregate(owner)
                .unwrap(),
        ),
    );
    (registry, source)
}

#[test]
fn complete_object_graph_distinguishes_pointer_recursion_from_by_value_cycles() {
    let (registry, source) = aggregate_package(true);
    registry.check_package_structure(&[source]).unwrap();
    let (registry, source) = aggregate_package(false);
    assert_eq!(
        registry.check_package_structure(&[source]),
        Err(CContextError::RecursiveObject)
    );
}

#[test]
fn incomplete_object_and_pointer_to_array_are_independent_negative_controls() {
    for array_pointer in [false, true] {
        let (mut registry, file, function, scope) = fixture();
        let record = registry.declare_struct(&file, key("Opaque")).unwrap();
        let object = CObjectType::structure(record.clone());
        let ty = if array_pointer {
            CObjectType::pointer(CPointerTarget::Object(Box::new(
                CObjectType::array(object, CArrayLength::new(2).unwrap()).unwrap(),
            )))
        } else {
            object
        };
        let local = registry
            .register_local(&scope, key("slot"), ty.clone())
            .unwrap();
        let ast = CExpressions::new(&registry);
        let statement = CStatements::new(&registry, function.clone())
            .unwrap()
            .declare(local, Some(ast.zero_initializer(ty).unwrap()))
            .unwrap();
        let mut source = package(&registry, file.clone(), function, scope, vec![statement]);
        source.items.insert(
            0,
            CFileItem::Declaration(
                CDeclarations::new(&registry, file)
                    .unwrap()
                    .forward_tag(CAggregateRef::Struct(record))
                    .unwrap(),
            ),
        );
        assert_eq!(
            registry.check_package_structure(&[source]),
            Err(CContextError::IncompleteObject)
        );
    }
}

#[test]
fn local_prototype_definition_linkage_must_agree() {
    let (registry, file, function, scope) = fixture();
    let mut source = package(&registry, file.clone(), function.clone(), scope, vec![]);
    source.items.insert(
        0,
        CFileItem::Declaration(
            CDeclarations::new(&registry, file)
                .unwrap()
                .function_prototype(function, CLinkage::Internal)
                .unwrap(),
        ),
    );
    assert_eq!(
        registry.check_package_structure(&[source]),
        Err(CContextError::LinkageMismatch)
    );
}
