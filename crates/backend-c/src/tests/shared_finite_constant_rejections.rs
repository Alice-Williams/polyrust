//! Public builders reject wrong types/files/owners; profile rejects expressions.
use super::*;

#[test]
fn finite_constant_constructors_and_profile_retain_exact_shapes() {
    let source = fixture(&[1]);
    let foreign = fixture(&[1]);
    let expressions = CExpressions::new(source.registry.registrations());
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
    let value = || expressions.literal(literal(1)).unwrap();
    let init = || expressions.expression_initializer(value()).unwrap();
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
                    expressions
                        .expression_initializer(expressions.literal(bad).unwrap())
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
    for initializer in [
        expressions
            .zero_initializer(source.objects[0].ty().clone())
            .unwrap(),
        expressions
            .expression_initializer(
                expressions
                    .binary(CBinaryOperator::Add, value(), value())
                    .unwrap(),
            )
            .unwrap(),
    ] {
        let file = definitions
            .source_file(vec![CFileItem::Definition(
                definitions
                    .object_definition(source.objects[0].clone(), CLinkage::External, initializer)
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
fn nonfinite_witnesses_and_over_capacity_constants_are_rejected() {
    for bits in [
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_0000,
        0xfff0_0000_0000_0001,
    ] {
        assert!(FiniteBinary64::from_bits(bits).is_err());
    }
    let oversized = owned_constant_fixture::literal_fixture_with_origins(
        &[("oversized", literal(1))],
        |origin| origin.documentation = vec![String::new(); 4096],
    );
    let errors = certify_resolved_package(&CDialect, linked(&oversized)).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == portable_diagnostics::DiagnosticCode::TargetResourceLimit)
    );
}
