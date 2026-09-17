//! Valid C trees still obey the narrower bitwise source profile.
use super::{project_c_package, tests::fixture};
use crate::ast::*;

#[derive(Clone, Copy, Debug)]
enum Op {
    Unary(CUnaryOperator),
    Binary(CBinaryOperator, CScalarType),
}

fn literal(scalar: CScalarType) -> CLiteral {
    match scalar {
        CScalarType::I32 => CLiteral::Signed(CSignedLiteral::I32(1)),
        CScalarType::I64 => CLiteral::Signed(CSignedLiteral::I64(1)),
        CScalarType::Int => CLiteral::Signed(CSignedLiteral::Int(1)),
        CScalarType::Bool => CLiteral::Bool(true),
        _ => unreachable!(),
    }
}

fn admitted(scalar: CScalarType, op: Op) -> bool {
    let (registry, source) = fixture(scalar);
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
    let value = match op {
        Op::Unary(operator) => expressions.unary(operator, input.clone()).unwrap(),
        Op::Binary(operator, right) => expressions
            .binary(
                operator,
                input.clone(),
                expressions.literal(literal(right)).unwrap(),
            )
            .unwrap(),
    };
    let statements = CStatements::new(registry.registrations(), function.clone()).unwrap();
    let body = statements
        .block(
            body.scope().clone(),
            vec![
                statements.discard(value).unwrap(),
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
    registry
        .registrations()
        .check_context(std::slice::from_ref(&source))
        .unwrap();
    match project_c_package(registry, vec![source]) {
        Ok(_) => true,
        Err(errors) => {
            assert!(
                errors.iter().any(|e| e.message.contains("profile"))
                    || matches!(op, Op::Unary(CUnaryOperator::Negate))
                        && errors.iter().any(|e| e
                            .message
                            .contains("C signed arithmetic is not proved representable")),
                "{errors:?}"
            );
            false
        }
    }
}

#[test]
fn bitwise_nodes_admit_only_equal_exact_widths_not_other_integer_operations() {
    let scalars = [
        CScalarType::I32,
        CScalarType::I64,
        CScalarType::Int,
        CScalarType::Bool,
    ];
    for left in scalars {
        let exact = matches!(left, CScalarType::I32 | CScalarType::I64);
        for unary in [CUnaryOperator::BitNot, CUnaryOperator::Negate] {
            assert_eq!(
                admitted(left, Op::Unary(unary)),
                exact && unary == CUnaryOperator::BitNot,
                "{left:?} {unary:?}"
            );
        }
        for right in scalars {
            for operator in [
                CBinaryOperator::BitAnd,
                CBinaryOperator::BitOr,
                CBinaryOperator::BitXor,
                CBinaryOperator::Add,
                CBinaryOperator::ShiftLeft,
                CBinaryOperator::ShiftRight,
            ] {
                let bitwise = matches!(
                    operator,
                    CBinaryOperator::BitAnd | CBinaryOperator::BitOr | CBinaryOperator::BitXor
                );
                assert_eq!(
                    admitted(left, Op::Binary(operator, right)),
                    bitwise && (exact || left == CScalarType::Bool) && left == right,
                    "{left:?} {operator:?} {right:?}"
                );
            }
        }
    }
}
