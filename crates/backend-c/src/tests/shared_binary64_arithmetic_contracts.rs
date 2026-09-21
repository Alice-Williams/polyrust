//! Arithmetic admission must not grant casts, mixed operands or integer UB.
use super::*;

#[test]
fn adjacent_scalar_operations_remain_outside_arithmetic_admission() {
    let source = f::fixture(504, &[CScalarType::F64], &[], &[None]);
    api(&source);
    let e = CExpressions::new(source.registry.registrations());
    let zero = e
        .literal(CLiteral::F64(
            portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
        ))
        .unwrap();
    let integer = e.literal(CLiteral::Signed(CSignedLiteral::I32(0))).unwrap();
    let one = e.literal(CLiteral::Signed(CSignedLiteral::I32(1))).unwrap();
    assert!(
        e.binary(CBinaryOperator::Remainder, zero.clone(), zero.clone())
            .is_err()
    );
    assert!(
        e.binary(CBinaryOperator::ShiftLeft, zero.clone(), one.clone())
            .is_err()
    );
    let bad = [
        e.binary(CBinaryOperator::Add, zero.clone(), integer.clone())
            .unwrap(),
        e.binary(CBinaryOperator::Multiply, integer.clone(), zero.clone())
            .unwrap(),
        e.binary(CBinaryOperator::Add, one.clone(), one.clone())
            .unwrap(),
        e.binary(CBinaryOperator::Divide, one, integer.clone())
            .unwrap(),
        e.numeric_conversion(CScalarType::F64, integer).unwrap(),
        e.numeric_conversion(CScalarType::I32, zero.clone())
            .unwrap(),
    ];
    for value in bad {
        let mut files = source.files.clone();
        let declaration =
            CDeclarations::new(source.registry.registrations(), files[1].identity().clone())
                .unwrap();
        let CFileItem::Definition(definition) = &files[1].items()[0] else {
            panic!("definition")
        };
        let CDefinitionKind::Function {
            function,
            linkage,
            parameters,
            body,
        } = definition.kind()
        else {
            panic!("function")
        };
        let statements =
            CStatements::new(source.registry.registrations(), function.clone()).unwrap();
        let result = e.read(e.parameter(parameters[0].clone()).unwrap()).unwrap();
        let body = statements
            .block(
                body.scope().clone(),
                vec![
                    statements.discard(value).unwrap(),
                    statements.return_statement(Some(result)).unwrap(),
                ],
            )
            .unwrap();
        files[1] = declaration
            .source_file(vec![CFileItem::Definition(
                declaration
                    .function_definition(function.clone(), *linkage, parameters.clone(), body)
                    .unwrap(),
            )])
            .unwrap();
        assert!(crate::dialect::project_c_package(source.registry.clone(), files).is_err());
    }
}

#[test]
fn arithmetic_call_arity_and_foreign_registry_authority_are_not_relaxed() {
    let leaf = api(&f::fixture(505, &[CScalarType::F64; 2], &[], &[None; 2]));
    let input = fixture::build(
        506,
        &leaf.functions().cloned().collect::<Vec<_>>(),
        fixture::Body::Arithmetic,
    );
    let owner = api(&input);
    let other = f::fixture(507, &[CScalarType::F64], &[], &[None]);
    let e = CExpressions::new(other.registry.registrations());
    assert!(e.direct(input.functions[0].clone()).is_err());
    let imported = f::fixture(
        508,
        &[CScalarType::F64],
        &[owner.functions().next().unwrap().clone()],
        &[None],
    );
    let e = CExpressions::new(imported.registry.registrations());
    let zero = || {
        e.literal(CLiteral::F64(
            portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
        ))
        .unwrap()
    };
    let call = || e.direct(imported.imported[0].clone()).unwrap();
    assert!(e.call_value(call(), vec![zero()]).is_err());
    assert!(e.call_value(call(), vec![zero(), zero(), zero()]).is_err());
}
