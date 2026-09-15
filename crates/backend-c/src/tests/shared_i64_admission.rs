//! Legal C ASTs must not widen the source-owned package admission grammar.
use super::{project_c_package, tests::fixture};
use crate::ast::*;

#[derive(Clone, Copy, Debug)]
enum Case {
    Comparison(CBinaryOperator, CScalarType, bool),
    Conversion(CScalarType),
}

fn admitted(case: Case) -> bool {
    let (registry, source) = fixture(CScalarType::I64);
    let CFileItem::Definition(definition) = &source.items()[1] else {
        panic!()
    };
    let CDefinitionKind::Function {
        function,
        linkage,
        parameters,
        body,
    } = definition.kind()
    else {
        panic!()
    };
    let expressions = CExpressions::new(registry.registrations());
    let input = expressions
        .read(expressions.parameter(parameters[0].clone()).unwrap())
        .unwrap();
    let probe = match case {
        Case::Comparison(operator, scalar, reverse) => {
            let literal = match scalar {
                CScalarType::I64 => CLiteral::Signed(CSignedLiteral::I64(1)),
                CScalarType::I32 => CLiteral::Signed(CSignedLiteral::I32(1)),
                CScalarType::Int => CLiteral::Signed(CSignedLiteral::Int(1)),
                CScalarType::Bool => CLiteral::Bool(true),
                _ => unreachable!(),
            };
            let other = expressions.literal(literal).unwrap();
            let (left, right) = if reverse {
                (other, input.clone())
            } else {
                (input.clone(), other)
            };
            expressions.binary(operator, left, right).unwrap()
        }
        Case::Conversion(target) => expressions
            .numeric_conversion(target, input.clone())
            .unwrap(),
    };
    let statements = CStatements::new(registry.registrations(), function.clone()).unwrap();
    let body = statements
        .block(
            body.scope().clone(),
            vec![
                statements.discard(probe).unwrap(),
                statements.return_statement(Some(input)).unwrap(),
            ],
        )
        .unwrap();
    let declarations =
        CDeclarations::new(registry.registrations(), source.identity().clone()).unwrap();
    let definition = declarations
        .function_definition(function.clone(), *linkage, parameters.clone(), body)
        .unwrap();
    let source = declarations
        .source_file(vec![
            source.items()[0].clone(),
            CFileItem::Definition(definition),
        ])
        .unwrap();
    // Construction/context are genuinely legal C. Rejection must occur at the
    // package profile, not because a malformed AST or foreign identity was used.
    registry
        .registrations()
        .check_context(std::slice::from_ref(&source))
        .unwrap();
    match project_c_package(registry, vec![source]) {
        Ok(_) => true,
        Err(errors) => {
            assert!(
                errors.iter().any(|error| error.message.contains("profile")),
                "{errors:?}"
            );
            false
        }
    }
}

#[test]
fn every_wide_comparison_requires_identical_operand_widths() {
    for operator in [
        CBinaryOperator::Equal,
        CBinaryOperator::NotEqual,
        CBinaryOperator::Less,
        CBinaryOperator::LessEqual,
        CBinaryOperator::Greater,
        CBinaryOperator::GreaterEqual,
    ] {
        for reverse in [false, true] {
            for scalar in [
                CScalarType::I64,
                CScalarType::I32,
                CScalarType::Int,
                CScalarType::Bool,
            ] {
                let case = Case::Comparison(operator, scalar, reverse);
                assert_eq!(admitted(case), scalar == CScalarType::I64, "{case:?}");
            }
        }
    }
}

#[test]
fn admitting_wide_values_does_not_admit_numeric_conversions() {
    for scalar in [
        CScalarType::Bool,
        CScalarType::Int,
        CScalarType::I32,
        CScalarType::I64,
    ] {
        assert!(!admitted(Case::Conversion(scalar)), "{scalar:?}");
    }
}
