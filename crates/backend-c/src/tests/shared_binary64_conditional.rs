//! Floating conditional is distinct from the guarded signed-integer operation.
use super::*;
use crate::dialect::CDependencyFunction;

#[derive(Clone, Copy, Debug)]
pub(super) enum Selection {
    First,
    Second,
    Negative,
    Absolute,
}
impl Selection {
    pub(super) const ALL: [Self; 4] = [Self::First, Self::Second, Self::Negative, Self::Absolute];
    pub(super) fn expected(self, bits: u64) -> u64 {
        const SIGN: u64 = 1 << 63;
        let magnitude = bits & !SIGN;
        match self {
            Self::First => bits,
            Self::Second => bits ^ SIGN,
            Self::Negative
                if bits & SIGN != 0 && magnitude != 0 && magnitude <= 0x7ff0_0000_0000_0000 =>
            {
                bits
            }
            Self::Negative => bits ^ SIGN,
            Self::Absolute => magnitude,
        }
    }
}

pub(super) fn select(e: &CExpressions<'_>, mode: Selection, operand: CValue) -> CValue {
    let zero = e
        .literal(CLiteral::F64(
            portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
        ))
        .unwrap();
    let negative = e.unary(CUnaryOperator::Negate, operand.clone()).unwrap();
    let condition = |op| {
        e.numeric_conversion(
            CScalarType::Bool,
            e.binary(op, operand.clone(), zero.clone()).unwrap(),
        )
        .unwrap()
    };
    match mode {
        Selection::First | Selection::Second => e
            .conditional(
                e.literal(CLiteral::Bool(matches!(mode, Selection::First)))
                    .unwrap(),
                operand,
                negative,
            )
            .unwrap(),
        Selection::Negative => e
            .conditional(condition(CBinaryOperator::Less), operand, negative)
            .unwrap(),
        Selection::Absolute => {
            let magnitude = e
                .conditional(condition(CBinaryOperator::Less), negative, operand.clone())
                .unwrap();
            e.conditional(condition(CBinaryOperator::Equal), zero, magnitude)
                .unwrap()
        }
    }
}

pub(super) fn owner(crate_id: u64, dependencies: &[CDependencyFunction]) -> CDependencyApi {
    let mut fixture = if dependencies.is_empty() {
        f::fixture(crate_id, &[CScalarType::F64; 4], &[], &[None; 4])
    } else {
        assert_eq!(dependencies.len(), 4);
        f::materialized_double_calls(crate_id, dependencies)
    };
    let expressions = CExpressions::new(fixture.registry.registrations());
    let declarations = CDeclarations::new(
        fixture.registry.registrations(),
        fixture.files[1].identity().clone(),
    )
    .unwrap();
    let items = fixture.files[1]
        .items()
        .iter()
        .enumerate()
        .map(|(index, item)| {
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
            let mut prefix = body.statements().to_vec();
            let returned = prefix.pop().unwrap();
            let CStatementKind::Return(Some(operand)) = returned.kind() else {
                panic!("fixture returns its input or materialized call");
            };
            let value = select(&expressions, Selection::ALL[index], operand.clone());
            assert_eq!(
                value.ty().kind(),
                &CObjectTypeKind::Scalar(CScalarType::F64)
            );
            let statements =
                CStatements::new(fixture.registry.registrations(), function.clone()).unwrap();
            prefix.push(statements.return_statement(Some(value)).unwrap());
            let body = statements.block(body.scope().clone(), prefix).unwrap();
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

#[test]
fn primitive_and_imported_double_conditional_have_exact_certificates() {
    let first = owner(98, &[]);
    let imports: Vec<_> = first.functions().cloned().collect();
    let second = owner(99, &imports);
    for api in [&first, &second] {
        assert_eq!(api.functions().count(), 4);
        let output = render_certified_package(&CStructuralRenderer, api.package()).unwrap();
        assert_eq!(output.files().len(), 2);
        for file in output.files() {
            let OutputContents::Text(text) = file.contents() else {
                panic!("text")
            };
            assert!(
                !text.contains("runtime") && !text.contains("memcpy") && !text.contains("0.0 -")
            );
        }
    }
}

#[path = "shared_binary64_conditional_native.rs"]
mod native;

#[test]
fn mismatched_condition_and_branches_reject_at_checked_construction() {
    let fixture = f::fixture(98, &[CScalarType::F64], &[], &[None]);
    let e = CExpressions::new(fixture.registry.registrations());
    let zero = e
        .literal(CLiteral::F64(
            portable_binary64::FiniteBinary64::from_bits(0).unwrap(),
        ))
        .unwrap();
    let boolean = e.literal(CLiteral::Bool(true)).unwrap();
    let integer = e.literal(CLiteral::Signed(CSignedLiteral::I64(0))).unwrap();
    assert!(
        e.conditional(integer.clone(), zero.clone(), zero.clone())
            .is_err()
    );
    assert!(
        e.conditional(boolean.clone(), integer, zero.clone())
            .is_err()
    );
    assert!(
        e.conditional(
            boolean,
            zero.clone(),
            e.literal(CLiteral::Bool(false)).unwrap()
        )
        .is_err()
    );
}
