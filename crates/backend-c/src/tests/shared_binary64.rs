//! Finite literal and transport admission through actual public certificates.
use super::{CDependencyApi, CDialect, dependency_fixture as f};
use crate::{ast::*, dialect::CStructuralRenderer};
use portable_codegen::*;

pub(super) fn values() -> Vec<u64> {
    let mut bits = vec![
        0,
        1,
        0x000f_ffff_ffff_ffff,
        0x0010_0000_0000_0000,
        0x3fef_ffff_ffff_ffff,
        0x3ff0_0000_0000_0000,
        0x3ff0_0000_0000_0001,
        0x7fef_ffff_ffff_ffff,
    ];
    let mut seed = 0x1462_9538_784a_6b29_u64;
    for _ in 0..128 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let magnitude = seed & 0x7fff_ffff_ffff_ffff;
        let finite = if magnitude >> 52 == 0x7ff {
            magnitude ^ (1 << 52)
        } else {
            magnitude
        };
        bits.push(finite);
    }
    let negative: Vec<_> = bits.iter().map(|bits| bits | (1 << 63)).collect();
    bits.extend(negative);
    bits
}

pub(super) fn fixture(bits: &[u64]) -> f::Fixture {
    let mut fixture = f::fixture(
        91,
        &vec![CScalarType::F64; bits.len() + 1],
        &[],
        &vec![None; bits.len() + 1],
    );
    let declarations = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.files[1].identity().clone(),
    )
    .unwrap();
    let expressions = CExpressions::new(fixture.registry.registrations());
    let items = fixture.files[1]
        .items()
        .iter()
        .enumerate()
        .map(|(index, item)| {
            if index == bits.len() {
                return item.clone();
            }
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
            let value = expressions
                .literal(CLiteral::F64(
                    portable_binary64::FiniteBinary64::from_bits(bits[index]).unwrap(),
                ))
                .unwrap();
            let body = statements
                .block(
                    body.scope().clone(),
                    vec![
                        statements
                            .discard(
                                expressions
                                    .read(expressions.parameter(parameters[0].clone()).unwrap())
                                    .unwrap(),
                            )
                            .unwrap(),
                        statements.return_statement(Some(value)).unwrap(),
                    ],
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
    fixture
}

pub(super) fn api(fixture: &f::Fixture) -> CDependencyApi {
    CDependencyApi::from_certificate(
        certify_resolved_package(&CDialect, f::linked(fixture)).unwrap(),
    )
    .unwrap()
}

#[test]
fn finite_values_and_original_owner_transport_are_certified() {
    let bits = values();
    let first = api(&fixture(&bits));
    let imports: Vec<_> = first.functions().cloned().collect();
    let second = api(&f::fixture(
        92,
        &vec![CScalarType::F64; imports.len()],
        &imports,
        &(0..imports.len()).map(Some).collect::<Vec<_>>(),
    ));
    for owner in [&first, &second] {
        let rendered = render_certified_package(&CStructuralRenderer, owner.package()).unwrap();
        assert_eq!(rendered.files().len(), 2);
        let text: String = rendered
            .files()
            .iter()
            .map(|file| {
                let OutputContents::Text(text) = file.contents() else {
                    panic!("source")
                };
                text.as_str()
            })
            .collect();
        assert!(text.contains("double "));
        assert!(text.contains("#include <float.h>"));
        for property in [
            "FLT_RADIX",
            "DBL_MANT_DIG",
            "DBL_MIN_EXP",
            "DBL_MAX_EXP",
            "DBL_HAS_SUBNORM",
            "FLT_EVAL_METHOD",
        ] {
            assert!(text.contains(property));
        }
        assert!(!text.contains("runtime.") && !text.contains("memcpy") && !text.contains("goto "));
    }
}

#[test]
fn binary64_platform_checks_cannot_be_removed_or_changed() {
    let fixture = fixture(&[1]);
    let files = super::platform::install_package(&fixture.registry, fixture.files).unwrap();
    let declarations = CDeclarations::new(
        fixture.registry.registrations(),
        files[0].identity().clone(),
    )
    .unwrap();
    let checks: Vec<_> = files[0]
        .items()
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if let CFileItem::StaticAssert(assertion) = item {
                (super::platform::classify(assertion).unwrap().object
                    == super::platform::Object::F64)
                    .then_some(index)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(checks.len(), 8);
    for index in checks {
        let CFileItem::StaticAssert(assertion) = &files[0].items()[index] else {
            panic!("assertion")
        };
        let CValueKind::Binary { left, right, .. } = assertion.condition().kind() else {
            panic!("equality")
        };
        let bad_literal = match right.kind() {
            CValueKind::Literal(CLiteral::Signed(CSignedLiteral::Int(value))) => {
                CLiteral::Signed(CSignedLiteral::Int(value + 1))
            }
            CValueKind::Literal(CLiteral::Unsigned(CUnsignedLiteral::Size(value))) => {
                CLiteral::Unsigned(CUnsignedLiteral::Size(value + 1))
            }
            _ => panic!("closed platform literal"),
        };
        let expressions = CExpressions::new(fixture.registry.registrations());
        let condition = expressions
            .binary(
                CBinaryOperator::Equal,
                left.as_ref().clone(),
                expressions.literal(bad_literal).unwrap(),
            )
            .unwrap();
        let mut wrong = files.clone();
        let mut items = wrong[0].items().to_vec();
        items[index] = CFileItem::StaticAssert(
            declarations
                .static_assert(condition, assertion.diagnostic().clone())
                .unwrap(),
        );
        wrong[0] = declarations.source_file(items).unwrap();
        assert!(
            super::platform::verify_package(&wrong)
                .unwrap_err()
                .contains("incorrect")
        );
        assert!(super::project_c_package(fixture.registry.clone(), wrong).is_err());

        let mut changed = files.clone();
        let mut items = changed[0].items().to_vec();
        items.remove(index);
        changed[0] = declarations.source_file(items).unwrap();
        assert!(super::project_c_package(fixture.registry.clone(), changed).is_err());
    }
}

#[path = "shared_binary64_native.rs"]
mod native;

#[test]
fn floating_arithmetic_cannot_enter_the_closed_c_profile() {
    let source = fixture(&[1]);
    let declarations = CDeclarations::new(
        source.registry.registrations(),
        source.files[1].identity().clone(),
    )
    .unwrap();
    let expressions = CExpressions::new(source.registry.registrations());
    let literal = expressions
        .literal(CLiteral::F64(
            portable_binary64::FiniteBinary64::from_bits(1).unwrap(),
        ))
        .unwrap();
    assert!(
        expressions
            .unary(CUnaryOperator::LogicalNot, literal.clone())
            .is_err()
    );
    assert!(
        expressions
            .unary(CUnaryOperator::BitNot, literal.clone())
            .is_err()
    );
    let values = [
        expressions
            .binary(CBinaryOperator::Add, literal.clone(), literal.clone())
            .unwrap(),
        expressions
            .binary(CBinaryOperator::Multiply, literal.clone(), literal.clone())
            .unwrap(),
        expressions
            .numeric_conversion(CScalarType::I32, literal.clone())
            .unwrap(),
        expressions
            .numeric_conversion(CScalarType::Bool, literal.clone())
            .unwrap(),
        expressions
            .binary(
                CBinaryOperator::Equal,
                literal.clone(),
                expressions
                    .literal(CLiteral::Signed(CSignedLiteral::I64(0)))
                    .unwrap(),
            )
            .unwrap(),
    ];
    for value in values {
        let mut files = source.files.clone();
        let mut items = files[1].items().to_vec();
        let CFileItem::Definition(definition) = &items[0] else {
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
        let body = statements
            .block(
                body.scope().clone(),
                vec![
                    statements.discard(value).unwrap(),
                    statements.return_statement(Some(literal.clone())).unwrap(),
                ],
            )
            .unwrap();
        items[0] = CFileItem::Definition(
            declarations
                .function_definition(function.clone(), *linkage, parameters.clone(), body)
                .unwrap(),
        );
        files[1] = declarations.source_file(items).unwrap();
        assert!(super::project_c_package(source.registry.clone(), files).is_err());
    }
}

#[path = "shared_binary64_comparisons.rs"]
mod comparisons;

#[path = "shared_binary64_records.rs"]
mod records;

#[path = "shared_binary64_trace.rs"]
mod trace;

#[path = "shared_binary64_negation.rs"]
mod negation;
