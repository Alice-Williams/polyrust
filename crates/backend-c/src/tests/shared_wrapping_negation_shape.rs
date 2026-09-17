//! Native ABI equivalence alone cannot detect a missing exact-width AST conversion.
use super::*;

fn normalized_return(fixture: &f::Fixture, width: CScalarType) -> bool {
    fixture.files[1].items().iter().all(|item| {
        let CFileItem::Definition(definition) = item else { return false };
        let CDefinitionKind::Function { body, .. } = definition.kind() else { return false };
        let [statement] = body.statements() else { return false };
        let CStatementKind::Return(Some(value)) = statement.kind() else { return false };
        if value.ty().kind() != &CObjectTypeKind::Scalar(width) {
            return false;
        }
        let conditional = match (width, value.kind()) {
            (CScalarType::I32, CValueKind::Convert {
                conversion: CConversion::Numeric(CScalarType::I32), operand,
            }) if operand.ty().kind() == &CObjectTypeKind::Scalar(CScalarType::Int) => operand.as_ref(),
            (CScalarType::I64, CValueKind::Conditional { .. }) => value,
            _ => return false,
        };
        let CValueKind::Conditional { condition, then_value, else_value } = conditional.kind() else {
            return false;
        };
        let CValueKind::Convert {
            conversion: CConversion::Numeric(CScalarType::Bool), operand: comparison,
        } = condition.kind() else { return false };
        let CValueKind::Binary {
            operator: CBinaryOperator::Equal, left, right,
        } = comparison.kind() else { return false };
        let expected_min = match width {
            CScalarType::I32 => CLiteral::Signed(CSignedLiteral::I32(i32::MIN)),
            CScalarType::I64 => CLiteral::Signed(CSignedLiteral::I64(i64::MIN)),
            _ => return false,
        };
        let CValueKind::Unary {
            operator: CUnaryOperator::Negate, operand,
        } = else_value.kind() else { return false };
        matches!(left.kind(), CValueKind::Read(place) if matches!(place.kind(), CPlaceKind::Parameter(_)))
            && left == operand
            && matches!(right.kind(), CValueKind::Literal(value) if *value == expected_min)
            && right == then_value
    })
}

#[test]
fn wrapping_negation_shape_detects_missing_width_normalization() {
    for width in [CScalarType::I32, CScalarType::I64] {
        let valid = fixture_widths(81, Guard::Valid, &[width]);
        assert!(normalized_return(&valid, width));
        for guard in [
            Guard::Missing,
            Guard::Reversed,
            Guard::WrongMinimum,
            Guard::WrongValue,
            Guard::WrongOperation,
        ] {
            assert!(
                !normalized_return(&fixture_widths(81, guard, &[width]), width),
                "{width:?} {guard:?}"
            );
        }
    }
    let mutant = fixture_widths(81, Guard::MissingNormalization, &[CScalarType::I32]);
    // C ABI compatibility accepts Int as the return type. This is intentionally
    // not a language-safety rejection: our lowering shape contract catches it.
    certify_resolved_package(&CDialect, f::linked(&mutant)).unwrap();
    assert!(!normalized_return(&mutant, CScalarType::I32));
}
