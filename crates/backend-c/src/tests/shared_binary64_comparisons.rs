//! Six ordered/unordered comparisons over a primitive double parameter.
use super::*;

pub(super) const OPERATORS: [CBinaryOperator; 6] = [
    CBinaryOperator::Equal,
    CBinaryOperator::NotEqual,
    CBinaryOperator::Less,
    CBinaryOperator::LessEqual,
    CBinaryOperator::Greater,
    CBinaryOperator::GreaterEqual,
];

pub(super) fn comparisons() -> CDependencyApi {
    let mut fixture = f::boolean_results(93, OPERATORS.len());
    let expressions = CExpressions::new(fixture.registry.registrations());
    let declarations = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.files[1].identity().clone(),
    )
    .unwrap();
    let items = fixture.files[1]
        .items()
        .iter()
        .zip(OPERATORS)
        .map(|(item, operator)| {
            let CFileItem::Definition(definition) = item else {
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
                CStatements::new(fixture.registry.registrations(), function.clone()).unwrap();
            let input = expressions
                .read(expressions.parameter(parameters[0].clone()).unwrap())
                .unwrap();
            let zero = expressions
                .literal(CLiteral::F64(
                    portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
                ))
                .unwrap();
            let comparison = expressions.binary(operator, input, zero).unwrap();
            let value = expressions
                .numeric_conversion(CScalarType::Bool, comparison)
                .unwrap();
            let body = statements
                .block(
                    body.scope().clone(),
                    vec![statements.return_statement(Some(value)).unwrap()],
                )
                .unwrap();
            CFileItem::Definition(
                declarations
                    .function_definition(function.clone(), *linkage, parameters.clone(), body)
                    .unwrap(),
            )
        })
        .collect();
    fixture.files[1] = declarations.source_file(items).unwrap();
    api(&fixture)
}

/// Independent category/sign oracle against zero, without host FP operations.
pub(super) fn cases() -> Vec<(u64, [bool; 6])> {
    let mut bits = values();
    bits.extend([
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_0001,
        0xfff8_0000_0000_0100,
        0x7ff0_0000_0000_0001,
        0xfff0_0000_0000_0001,
    ]);
    bits.into_iter()
        .map(|bits| {
            let magnitude = bits & 0x7fff_ffff_ffff_ffff;
            let nan = magnitude > 0x7ff0_0000_0000_0000;
            let equal = magnitude == 0;
            let less = bits >> 63 == 1 && !equal && !nan;
            let greater = bits >> 63 == 0 && !equal && !nan;
            (
                bits,
                [
                    equal,
                    !equal,
                    less,
                    less || equal,
                    greater,
                    greater || equal,
                ],
            )
        })
        .collect()
}

#[test]
fn comparison_signatures_remain_exact_and_boolean() {
    let owner = comparisons();
    assert_eq!(owner.functions().count(), 6);
    for function in owner.functions() {
        assert_eq!(
            function.signature().parameters()[0].declared_type(),
            &CObjectType::scalar(CScalarType::F64)
        );
        assert!(
            matches!(function.signature().return_type(), CReturnType::Value(result)
            if result.declared_type() == &CObjectType::scalar(CScalarType::Bool))
        );
    }
}
