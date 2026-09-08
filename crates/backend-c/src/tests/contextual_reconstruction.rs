//! Inside ast's privacy boundary: corrupt caches, children and inventories.

use super::*;
use portable_codegen::RelativeOutputPath;

pub(super) fn key(name: &str) -> CDeclarationKey {
    CDeclarationKey {
        name: CIdentifier::new(name).unwrap(),
        origin: CGeneratedOrigin::Synthesized(CSynthesisReason::TestHarness),
    }
}
pub(super) fn fixture() -> (CRegistry, CFileRef, CFunctionRef, CScopeRef) {
    fixture_return(CReturnType::Void)
}

pub(super) fn fixture_return(
    result: CReturnType,
) -> (CRegistry, CFileRef, CFunctionRef, CScopeRef) {
    let mut registry = CRegistry::new();
    let file = registry
        .register_file(CFileKey {
            path: RelativeOutputPath::new("src/checked.c").unwrap(),
            role: CFileRole::TestSource,
        })
        .unwrap();
    let function = registry
        .register_function(&file, key("run"), CFunctionType::new(result, vec![]))
        .unwrap();
    let scope = registry
        .register_scope(&function, None, key("root"))
        .unwrap();
    (registry, file, function, scope)
}
pub(super) fn int(ast: &CExpressions<'_>) -> CValue {
    ast.literal(CLiteral::Signed(CSignedLiteral::I32(3)))
        .unwrap()
}
pub(super) fn package(
    registry: &CRegistry,
    file: CFileRef,
    function: CFunctionRef,
    scope: CScopeRef,
    statements: Vec<CStatement>,
) -> CSourceFile {
    let body = CStatements::new(registry, function.clone())
        .unwrap()
        .block(scope, statements)
        .unwrap();
    let ast = CDeclarations::new(registry, file).unwrap();
    let definition = ast
        .function_definition(function, CLinkage::External, vec![], body)
        .unwrap();
    ast.source_file(vec![CFileItem::Definition(definition)])
        .unwrap()
}

#[test]
fn reconstruction_rejects_forged_result_and_descendant_types() {
    let (registry, file, function, scope) = fixture();
    let ast = CExpressions::new(&registry);
    let statement = CStatements::new(&registry, function.clone()).unwrap();
    let valid = ast
        .binary(CBinaryOperator::Add, int(&ast), int(&ast))
        .unwrap();
    let make = |value| {
        package(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            vec![statement.discard(value).unwrap()],
        )
    };
    registry
        .check_local_structure(&[make(valid.clone())])
        .unwrap();
    let mut bad = valid.clone();
    bad.ty = CObjectType::scalar(CScalarType::U64);
    assert_eq!(
        registry.check_local_structure(&[make(bad)]),
        Err(CContextError::StoredStructureMismatch)
    );
    let mut bad = valid;
    if let CValueKind::Binary { left, .. } = &mut bad.kind {
        // The parent cache is still correct: checking only that cache misses this.
        left.ty = CObjectType::scalar(CScalarType::U32);
    } else {
        unreachable!()
    }
    assert_eq!(
        registry.check_local_structure(&[make(bad)]),
        Err(CContextError::StoredStructureMismatch)
    );
}

#[test]
fn reconstruction_authenticates_place_origins_even_with_copied_brands() {
    let (mut registry, file, function, scope) = fixture();
    let (mut foreign, _, foreign_function, foreign_scope) = fixture();
    let local = registry
        .register_local(&scope, key("slot"), CObjectType::scalar(CScalarType::I32))
        .unwrap();
    let other = foreign
        .register_local(
            &foreign_scope,
            key("slot"),
            CObjectType::scalar(CScalarType::I32),
        )
        .unwrap();
    assert_ne!(function, foreign_function);
    let ast = CExpressions::new(&registry);
    let mut place = ast.local(local).unwrap();
    place.kind = CPlaceKind::Local(other);
    let statement = CStatements::new(&registry, function.clone()).unwrap();
    let file = package(
        &registry,
        file,
        function,
        scope,
        vec![statement.discard(ast.read(place).unwrap()).unwrap()],
    );
    assert!(matches!(
        registry.check_local_structure(&[file]),
        Err(CContextError::Expression(CExpressionError::Registry(
            CRegistryError::CrossRegistry
        )))
    ));
}

#[test]
fn reconstruction_checks_place_and_initializer_caches() {
    let (mut registry, file, function, scope) = fixture();
    let local = registry
        .register_local(&scope, key("slot"), CObjectType::scalar(CScalarType::I32))
        .unwrap();
    let ast = CExpressions::new(&registry);
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let mut place = ast.local(local.clone()).unwrap();
    place.ty = CObjectType::scalar(CScalarType::U64);
    let corrupt_read = statements.discard(ast.read(place).unwrap()).unwrap();
    assert_eq!(
        registry.check_local_structure(&[package(
            &registry,
            file.clone(),
            function.clone(),
            scope.clone(),
            vec![corrupt_read]
        )]),
        Err(CContextError::StoredStructureMismatch)
    );
    let mut declaration = statements
        .declare(local, Some(ast.expression_initializer(int(&ast)).unwrap()))
        .unwrap();
    if let CStatementKind::Declare(value) = &mut declaration.kind {
        value.initializer.as_mut().unwrap().ty = CObjectType::scalar(CScalarType::U64);
    } else {
        unreachable!()
    }
    assert_eq!(
        registry.check_local_structure(&[package(
            &registry,
            file,
            function,
            scope,
            vec![declaration]
        )]),
        Err(CContextError::StoredStructureMismatch)
    );
}

#[test]
fn reconstruction_derives_member_inventory_instead_of_trusting_projection() {
    let (mut registry, file, _, _) = fixture();
    let record = registry.declare_struct(&file, key("Record")).unwrap();
    let owner = CAggregateRef::Struct(record);
    let member = registry
        .register_member(&owner, key("field"), CObjectType::scalar(CScalarType::I32))
        .unwrap();
    registry.define_aggregate(&owner, vec![member]).unwrap();
    let ast = CDeclarations::new(&registry, file).unwrap();
    let declaration = ast.aggregate(owner).unwrap();
    let valid = ast
        .source_file(vec![CFileItem::Declaration(declaration.clone())])
        .unwrap();
    registry.check_local_structure(&[valid]).unwrap();
    let mut bad = declaration;
    if let CDeclarationKind::Aggregate { members, .. } = &mut bad.kind {
        members.clear();
    } else {
        unreachable!()
    }
    assert_eq!(
        registry
            .check_local_structure(&[ast.source_file(vec![CFileItem::Declaration(bad)]).unwrap()]),
        Err(CContextError::StoredStructureMismatch)
    );
}

#[test]
fn reconstruction_rechecks_nested_calls_and_static_expression_categories() {
    let (mut registry, file, function, scope) = fixture();
    let callee = registry
        .register_function(
            &file,
            key("callee"),
            CFunctionType::new(
                CReturnType::Value(
                    CReturnValue::new(CObjectType::scalar(CScalarType::I32)).unwrap(),
                ),
                vec![CParameterType::new(CObjectType::scalar(CScalarType::I32)).unwrap()],
            ),
        )
        .unwrap();
    let ast = CExpressions::new(&registry);
    let value = ast
        .call_value(ast.direct(callee).unwrap(), vec![int(&ast)])
        .unwrap();
    let statements = CStatements::new(&registry, function.clone()).unwrap();
    let mut bad = value.clone();
    if let CValueKind::Call(call) = &mut bad.kind {
        call.arguments.clear();
    } else {
        unreachable!()
    }
    assert!(matches!(
        registry.check_local_structure(&[package(
            &registry,
            file.clone(),
            function,
            scope,
            vec![statements.discard(bad).unwrap()]
        )]),
        Err(CContextError::Expression(
            CExpressionError::ArityMismatch { .. }
        ))
    ));
    let declarations = CDeclarations::new(&registry, file).unwrap();
    let mut assertion = declarations
        .static_assert(int(&ast), CAssertDiagnostic::new("condition"))
        .unwrap();
    assertion.condition = value;
    assert!(matches!(
        registry.check_local_structure(&[declarations
            .source_file(vec![CFileItem::StaticAssert(assertion)])
            .unwrap()]),
        Err(CContextError::File(
            CFileError::ExpectedIntegerConstantExpression
        ))
    ));
}
