//! Fail closed on shape, type, owner and resource mutations.
use super::*;

#[test]
fn infinity_is_arithmetic_but_not_integer_constant_syntax() {
    let registry = CRegistry::new();
    let ast = CExpressions::new(&registry);
    let value = expression(&ast, Binary64Sign::Positive);
    assert_eq!(value.ty(), &CObjectType::scalar(CScalarType::F64));
    assert!(CKnownConstant::DoubleInfinity.is_arithmetic_constant_expression());
    assert!(!CKnownConstant::DoubleInfinity.is_integer_constant_expression());
    assert_eq!(
        CKnownConstant::DoubleInfinity.header(),
        crate::dialect::CHeader::Math
    );
    assert_eq!(CKnownConstant::DoubleInfinity.spelling(), "HUGE_VAL");
    assert!(ast.numeric_conversion(CScalarType::I64, value).is_ok());
    for bits in [
        0x7ff0000000000000,
        0xfff0000000000000,
        0x7ff8000000000000,
        0xfff0000000000001,
    ] {
        assert!(FiniteBinary64::from_bits(bits).is_err());
    }
}

#[test]
fn only_exact_infinity_constant_forms_enter_inventory() {
    let source = fixture(&[Binary64Sign::Positive]);
    let foreign = fixture(&[Binary64Sign::Positive]);
    let ast = CExpressions::new(source.registry.registrations());
    let definitions = CDeclarations::new(
        source.registry.registrations(),
        source.files[1].identity().clone(),
    )
    .unwrap();
    let value = || expression(&ast, Binary64Sign::Positive);
    let init = || ast.expression_initializer(value()).unwrap();
    assert!(
        definitions
            .static_assert(
                ast.numeric_conversion(CScalarType::I64, value()).unwrap(),
                CAssertDiagnostic::new(b"infinity is not an integer constant expression".to_vec()),
            )
            .is_err()
    );
    assert!(
        definitions
            .object_definition(foreign.objects[0].clone(), CLinkage::External, init())
            .is_err()
    );
    for bad in [
        CLiteral::Bool(false),
        CLiteral::Signed(CSignedLiteral::I32(0)),
        CLiteral::Signed(CSignedLiteral::I64(0)),
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
    for linkage in [CLinkage::Internal, CLinkage::None] {
        assert!(
            definitions
                .object_definition(source.objects[0].clone(), linkage, init())
                .is_err()
        );
    }
    let foreign_ast = CExpressions::new(foreign.registry.registrations());
    assert!(
        ast.unary(
            CUnaryOperator::Negate,
            expression(&foreign_ast, Binary64Sign::Positive)
        )
        .is_err()
    );
    for bad in [
        ast.binary(CBinaryOperator::Add, value(), value()).unwrap(),
        ast.numeric_conversion(CScalarType::F64, value()).unwrap(),
        ast.unary(
            CUnaryOperator::Negate,
            expression(&ast, Binary64Sign::Negative),
        )
        .unwrap(),
        ast.numeric_conversion(CScalarType::F64, ast.known_constant(CKnownConstant::IntMax))
            .unwrap(),
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
    let owned = api(&source);
    let other = api(&foreign);
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
                TargetTypeRef::Primitive(crate::dialect::CPrimitiveType::Scalar(CScalarType::I64));
        } else {
            changed.dependency_values[0].owner =
                other.constants().next().unwrap().package_identity();
        }
        assert!(changed.verify(&CDialect).is_err());
    }
}

#[test]
fn infinity_constants_cannot_bypass_resource_limits() {
    let mut source = owned_constant_fixture::literal_fixture_with_origins(
        &[(
            "oversized",
            CLiteral::F64(FiniteBinary64::from_bits(0).unwrap()),
        )],
        |origin| origin.documentation = vec![String::new(); 4096],
    );
    set_values(&mut source, &[Binary64Sign::Negative]);
    let errors = certify_resolved_package(&CDialect, linked(&source)).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == portable_diagnostics::DiagnosticCode::TargetResourceLimit)
    );
}
