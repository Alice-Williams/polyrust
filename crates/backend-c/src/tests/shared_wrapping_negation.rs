//! Numeric certificates, not profile admission, justify guarded signed negation.
use super::{
    CDependencyApi, CDependencyFunction, CDialect, dependency_fixture as f, project_c_package,
};
use crate::ast::*;
use portable_codegen::*;

#[derive(Clone, Copy, Debug)]
pub(super) enum Guard {
    Valid,
    Missing,
    Reversed,
    WrongMinimum,
    WrongValue,
    SafeLiteral,
    WrongOperation,
    MissingNormalization,
}

pub(super) fn fixture(id: u64, guard: Guard) -> f::Fixture {
    fixture_widths(id, guard, &[CScalarType::I32, CScalarType::I64])
}
fn fixture_widths(id: u64, guard: Guard, widths: &[CScalarType]) -> f::Fixture {
    let mut fixture = f::fixture(id, widths, &[], &vec![None; widths.len()]);
    let source = &fixture.files[1];
    let expressions = CExpressions::new(fixture.registry.registrations());
    let declarations =
        CDeclarations::new(fixture.registry.registrations(), source.identity().clone()).unwrap();
    let items = source
        .items()
        .iter()
        .map(|item| {
            let CFileItem::Definition(definition) = item else {
                panic!("function")
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
            let scalar = match parameters[0].ty().kind() {
                CObjectTypeKind::Scalar(ty) => *ty,
                _ => panic!("scalar"),
            };
            let literal = |value: i64| {
                expressions
                    .literal(CLiteral::Signed(match scalar {
                        CScalarType::I32 => CSignedLiteral::I32(value.try_into().unwrap()),
                        CScalarType::I64 => CSignedLiteral::I64(value),
                        _ => panic!("width"),
                    }))
                    .unwrap()
            };
            let min = if scalar == CScalarType::I32 {
                i64::from(i32::MIN)
            } else {
                i64::MIN
            };
            let input = expressions
                .read(expressions.parameter(parameters[0].clone()).unwrap())
                .unwrap();
            let operand = if matches!(guard, Guard::SafeLiteral) {
                literal(1)
            } else {
                input.clone()
            };
            let negated = expressions
                .unary(
                    if matches!(guard, Guard::WrongOperation) {
                        CUnaryOperator::BitNot
                    } else {
                        CUnaryOperator::Negate
                    },
                    operand,
                )
                .unwrap();
            let value = if matches!(guard, Guard::Missing | Guard::SafeLiteral) {
                negated
            } else {
                let compare = if matches!(guard, Guard::WrongValue) {
                    literal(0)
                } else {
                    input.clone()
                };
                let limit = if matches!(guard, Guard::WrongMinimum) {
                    min + 1
                } else {
                    min
                };
                let operator = if matches!(guard, Guard::Reversed) {
                    CBinaryOperator::NotEqual
                } else {
                    CBinaryOperator::Equal
                };
                let condition = expressions
                    .binary(operator, compare, literal(limit))
                    .unwrap();
                let condition = expressions
                    .numeric_conversion(CScalarType::Bool, condition)
                    .unwrap();
                expressions
                    .conditional(condition, literal(min), negated)
                    .unwrap()
            };
            let value =
                if scalar == CScalarType::I32 && !matches!(guard, Guard::MissingNormalization) {
                    expressions
                        .numeric_conversion(CScalarType::I32, value)
                        .unwrap()
                } else {
                    value
                };
            let statements =
                CStatements::new(fixture.registry.registrations(), function.clone()).unwrap();
            let mut sequence = Vec::new();
            if matches!(guard, Guard::SafeLiteral) {
                sequence.push(statements.discard(input).unwrap());
            }
            sequence.push(statements.return_statement(Some(value)).unwrap());
            let body = statements.block(body.scope().clone(), sequence).unwrap();
            CFileItem::Definition(
                declarations
                    .function_definition(function.clone(), *linkage, parameters.clone(), body)
                    .unwrap(),
            )
        })
        .collect();
    fixture.files[1] = declarations.source_file(items).unwrap();
    fixture
}

pub(super) fn api(id: u64) -> CDependencyApi {
    let fixture = fixture(id, Guard::Valid);
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, f::linked(&fixture)).unwrap(),
    )
    .unwrap()
}

#[test]
fn wrapping_negation_guard_is_proved_for_each_width_and_bad_guards_reject() {
    for scalar in [CScalarType::I32, CScalarType::I64] {
        for guard in [
            Guard::Valid,
            Guard::Missing,
            Guard::Reversed,
            Guard::WrongMinimum,
            Guard::WrongValue,
            Guard::SafeLiteral,
        ] {
            let fixture = fixture_widths(81, guard, &[scalar]);
            fixture
                .registry
                .registrations()
                .check_context(&fixture.files)
                .unwrap();
            let result = project_c_package(fixture.registry, fixture.files);
            if matches!(guard, Guard::Valid | Guard::SafeLiteral) {
                let checked = verify_unresolved_package(&CDialect, result.unwrap()).unwrap();
                let linked = TargetLinker::new(CDialect).link_ast(&checked).unwrap();
                certify_resolved_package(&CDialect, linked).unwrap();
            } else {
                let errors = result.expect_err("unsafe negate must reject");
                assert!(
                    errors.iter().any(|error| error
                        .message
                        .contains("C signed arithmetic is not proved representable")),
                    "{scalar:?} {guard:?}: {errors:?}"
                );
            }
        }
    }
}

#[test]
fn wrapping_negation_bodies_remain_closed_original_authority_dependencies() {
    let producer = api(81);
    let imports: Vec<CDependencyFunction> = producer.functions().cloned().collect();
    assert_eq!(imports.len(), 2);
    let consumer = f::fixture(
        82,
        &[CScalarType::I32, CScalarType::I64],
        &imports,
        &[Some(0), Some(1)],
    );
    let certificate = certify_resolved_package(&CDialect, f::linked(&consumer)).unwrap();
    let api = CDependencyApi::from_certificate(certificate).unwrap();
    assert!(
        api.functions()
            .all(|function| function.stack_bound_bytes() > imports[0].stack_bound_bytes())
    );
    assert_eq!(api.functions().count(), 2);
}

#[path = "shared_wrapping_negation_shape.rs"]
mod shape;

#[path = "shared_wrapping_negation_native.rs"]
mod native;
