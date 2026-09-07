use super::*;

#[test]
fn structural_operator_spelling_is_closed() {
    assert_eq!(unary_operator(JavaUnaryOperator::Not), "!");
    assert_eq!(binary_operator(JavaBinaryOperator::ShiftRight), ">>");
}

#[test]
fn switch_patterns_render_complete_case_labels() {
    let names = std::collections::BTreeMap::new();
    assert_eq!(
        render_switch_pattern(&JavaPattern::Literal(JavaLiteral::I32(7)), &names)
            .expect("literal pattern renders"),
        "case 7"
    );
    assert_eq!(
        render_switch_pattern(&JavaPattern::Default, &names).expect("default pattern renders"),
        "default"
    );
}
