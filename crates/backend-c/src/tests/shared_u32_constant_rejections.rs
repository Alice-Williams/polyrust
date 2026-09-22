//! Builder, exact-profile, original-owner and resource rejection boundaries.
use super::*;

#[test]
fn u32_constant_constructors_and_inventory_reject_wrong_types_and_expressions() {
    let source = fixture(&[0x10000]);
    let foreign = fixture(&[0x10000]);
    let ast = CExpressions::new(source.registry.registrations());
    let definitions = CDeclarations::new(
        source.registry.registrations(),
        source.files[1].identity().clone(),
    )
    .unwrap();
    let declarations = CDeclarations::new(
        source.registry.registrations(),
        source.files[0].identity().clone(),
    )
    .unwrap();
    let value = || ast.literal(literal(0x10000)).unwrap();
    let init = || ast.expression_initializer(value()).unwrap();
    for bad in [
        CLiteral::Bool(false),
        CLiteral::Signed(CSignedLiteral::I32(0)),
        CLiteral::Signed(CSignedLiteral::I64(0)),
        CLiteral::Unsigned(CUnsignedLiteral::U64(0)),
    ] {
        assert!(
            definitions
                .object_definition(
                    source.objects[0].clone(),
                    CLinkage::External,
                    ast.expression_initializer(ast.literal(bad).unwrap())
                        .unwrap()
                )
                .is_err()
        );
    }
    assert!(
        definitions
            .object_definition(foreign.objects[0].clone(), CLinkage::External, init())
            .is_err()
    );
    assert!(
        declarations
            .object_definition(source.objects[0].clone(), CLinkage::External, init())
            .is_err()
    );
    for linkage in [CLinkage::Internal, CLinkage::None] {
        assert!(
            definitions
                .object_definition(source.objects[0].clone(), linkage, init())
                .is_err()
        );
    }
    for bad in [
        ast.binary(CBinaryOperator::Add, value(), value()).unwrap(),
        ast.numeric_conversion(CScalarType::U32, value()).unwrap(),
    ] {
        assert!(CScalarConstantValue::from_expression(&bad).is_none());
        let file = definitions
            .source_file(vec![CFileItem::Definition(
                definitions
                    .object_definition(
                        source.objects[0].clone(),
                        CLinkage::External,
                        ast.expression_initializer(bad).unwrap(),
                    )
                    .unwrap(),
            )])
            .unwrap();
        assert!(
            project_c_package(source.registry.clone(), vec![source.files[0].clone(), file])
                .is_err()
        );
    }
    let zero = ast
        .zero_initializer(source.objects[0].ty().clone())
        .unwrap();
    let file = definitions
        .source_file(vec![CFileItem::Definition(
            definitions
                .object_definition(source.objects[0].clone(), CLinkage::External, zero)
                .unwrap(),
        )])
        .unwrap();
    assert!(
        project_c_package(source.registry.clone(), vec![source.files[0].clone(), file]).is_err()
    );
}

#[test]
fn u32_imports_authenticate_original_owner_and_exact_unsigned_type() {
    let owned = api(&fixture(&[0x10ffff]));
    let other = api(&fixture(&[0x10ffff]));
    let values: Vec<_> = owned.constants().cloned().collect();
    let reader = constant_consumer_fixture::fixture(
        923,
        &values,
        None,
        constant_consumer_fixture::Usage::Read,
    );
    let projected = project_c_package(reader.registry, reader.files).unwrap();
    let catalogue = CDialect.package_symbol_catalogue(&projected).unwrap();
    for wrong_type in [false, true] {
        let mut changed = catalogue.clone();
        if wrong_type {
            changed.dependency_values[0].ty =
                TargetTypeRef::Primitive(crate::dialect::CPrimitiveType::Scalar(CScalarType::I32));
        } else {
            changed.dependency_values[0].owner =
                other.constants().next().unwrap().package_identity();
        }
        assert!(changed.verify(&CDialect).is_err());
    }
}

#[test]
fn u32_constants_cannot_bypass_resource_limits() {
    let source = owned_constant_fixture::literal_fixture_with_origins(
        &[("oversized", literal(0x10ffff))],
        |origin| origin.documentation = vec![String::new(); 4096],
    );
    let errors = certify_resolved_package(&CDialect, linked(&source)).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == portable_diagnostics::DiagnosticCode::TargetResourceLimit)
    );
}
