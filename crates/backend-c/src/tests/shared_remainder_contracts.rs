//! Closed signature, recursive admission and original registry authority.
use super::*;

#[test]
fn remainder_requires_two_exact_double_arguments_and_original_authority() {
    let source = f::fixture(604, &[CScalarType::F64; 2], &[], &[None; 2]);
    let e = CExpressions::new(source.registry.registrations());
    let zero = e
        .literal(CLiteral::F64(
            portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
        ))
        .unwrap();
    let call = || e.known(CKnownCall::FloatRemainder);
    for arguments in [vec![], vec![zero.clone()], vec![zero.clone(); 3]] {
        assert!(e.call_value(call(), arguments).is_err());
    }
    for invalid in [
        e.literal(CLiteral::Bool(false)).unwrap(),
        e.literal(CLiteral::Signed(CSignedLiteral::I64(0))).unwrap(),
    ] {
        assert!(
            e.call_value(call(), vec![invalid.clone(), zero.clone()])
                .is_err()
        );
        assert!(e.call_value(call(), vec![zero.clone(), invalid]).is_err());
    }
    assert!(
        e.binary(CBinaryOperator::Remainder, zero.clone(), zero.clone())
            .is_err()
    );
    let value = e.call_value(call(), vec![zero.clone(), zero]).unwrap();
    assert_eq!(
        value.ty().kind(),
        &CObjectTypeKind::Scalar(CScalarType::F64)
    );
    let owners = chain();
    let middle = owners[1].functions().next().unwrap().clone();
    let foreign = f::fixture(605, &[CScalarType::F64], &[middle], &[None]);
    assert!(e.direct(foreign.imported[0].clone()).is_err());
    let imported = CExpressions::new(foreign.registry.registrations());
    let target = imported.direct(foreign.imported[0].clone()).unwrap();
    assert!(imported.call_value(target, vec![]).is_err());
}

#[test]
fn remainder_does_not_admit_other_known_calls_in_its_operands() {
    for unsupported in [false, true] {
        let mut source = f::fixture(606, &[CScalarType::F64], &[], &[None]);
        let e = CExpressions::new(source.registry.registrations());
        let zero = e
            .literal(CLiteral::F64(
                portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
            ))
            .unwrap();
        let predicate = if unsupported {
            e.call_value(e.known(CKnownCall::IsNan), vec![zero.clone()])
                .unwrap()
        } else {
            e.literal(CLiteral::Signed(CSignedLiteral::Int(0))).unwrap()
        };
        let condition = e
            .binary(
                CBinaryOperator::Equal,
                predicate,
                e.literal(CLiteral::Signed(CSignedLiteral::Int(0))).unwrap(),
            )
            .unwrap();
        let condition = e.numeric_conversion(CScalarType::Bool, condition).unwrap();
        let operand = e
            .conditional(condition, zero.clone(), zero.clone())
            .unwrap();
        let value = e
            .call_value(e.known(CKnownCall::FloatRemainder), vec![operand, zero])
            .unwrap();
        let declaration = CDeclarations::new(
            source.registry.registrations(),
            source.files[1].identity().clone(),
        )
        .unwrap();
        let CFileItem::Definition(definition) = &source.files[1].items()[0] else {
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
        let parameter = e.read(e.parameter(parameters[0].clone()).unwrap()).unwrap();
        let body = statements
            .block(
                body.scope().clone(),
                vec![
                    statements.discard(parameter).unwrap(),
                    statements.return_statement(Some(value)).unwrap(),
                ],
            )
            .unwrap();
        source.files[1] = declaration
            .source_file(vec![CFileItem::Definition(
                declaration
                    .function_definition(function.clone(), *linkage, parameters.clone(), body)
                    .unwrap(),
            )])
            .unwrap();
        let result = crate::dialect::project_c_package(source.registry, source.files);
        if unsupported {
            let errors = result.unwrap_err();
            assert!(
                errors.iter().any(|error| error
                    .message
                    .contains("C shared call profile requires an admitted target")),
                "{errors:?}"
            );
        } else {
            assert!(result.is_ok(), "{result:?}");
        }
    }
}
