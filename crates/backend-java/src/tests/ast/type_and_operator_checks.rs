use super::{
    BTreeSet, JAVA_KEYWORDS, JavaArrayOwnership, JavaBinaryOperator, JavaIdentifier, JavaKnownType,
    JavaLiteral, JavaNullPurpose, JavaPrimitive, JavaType, JavaTypeName, JavaTypeUse,
    JavaUnaryOperator, JavaWildcardBound, binary_signature_matches, literal_matches_type,
    unary_signature_matches,
};

#[test]
fn identifiers_and_type_positions_fail_closed() {
    assert!(JavaIdentifier::new("valid_name").is_ok());
    assert!(JavaIdentifier::new("class").is_err());
    assert!(JavaIdentifier::new("false").is_err());
    assert!(JavaIdentifier::new("_").is_err());
    assert!(JavaIdentifier::new("9invalid").is_err());
    assert_eq!(JavaIdentifier::from_portable("false").as_str(), "false_");
    assert_eq!(JavaIdentifier::from_portable("_").as_str(), "__");
    assert_eq!(
        JavaIdentifier::from_portable("__polyrust_callResult_0").as_str(),
        "__polyrust_callResult_0_user"
    );
    assert_eq!(
        JavaIdentifier::from_portable("match-value").as_str(),
        "match_value"
    );
    assert_eq!(
        JAVA_KEYWORDS.iter().copied().collect::<BTreeSet<_>>().len(),
        JAVA_KEYWORDS.len()
    );
    for keyword in JAVA_KEYWORDS {
        assert!(
            JavaIdentifier::new(*keyword).is_err(),
            "protected Java spelling was accepted: {keyword}"
        );
    }

    assert!(
        !JavaType::primitive(JavaPrimitive::Void)
            .verify(JavaTypeUse::Value)
            .is_empty()
    );
    assert!(
        !JavaType::primitive(JavaPrimitive::Int)
            .verify(JavaTypeUse::GenericArgument)
            .is_empty()
    );
    assert!(
        !JavaType::Wildcard { bound: None }
            .verify(JavaTypeUse::Value)
            .is_empty()
    );
    assert!(
        JavaType::Wildcard { bound: None }
            .verify(JavaTypeUse::GenericArgument)
            .is_empty()
    );
    assert!(
        !JavaType::Wildcard {
            bound: Some((
                JavaWildcardBound::Extends,
                Box::new(JavaType::primitive(JavaPrimitive::Int)),
            )),
        }
        .verify(JavaTypeUse::GenericArgument)
        .is_empty()
    );
    assert!(
        !JavaType::generic(
            JavaKnownType::List,
            vec![JavaType::Array {
                component: Box::new(JavaType::primitive(JavaPrimitive::Byte)),
                ownership: JavaArrayOwnership::InternalMutable,
            }],
        )
        .verify(JavaTypeUse::Field)
        .is_empty()
    );
    assert!(
        !JavaType::Generic {
            raw: JavaTypeName::Known(JavaKnownType::List),
            arguments: vec![],
        }
        .verify(JavaTypeUse::Value)
        .is_empty()
    );
}

#[test]
fn literal_and_operator_signatures_reject_false_type_claims() {
    let boolean = JavaType::primitive(JavaPrimitive::Boolean);
    let int = JavaType::primitive(JavaPrimitive::Int);
    let long = JavaType::primitive(JavaPrimitive::Long);
    let string = JavaType::known(JavaKnownType::String);

    assert!(literal_matches_type(&JavaLiteral::Boolean(true), &boolean));
    assert!(!literal_matches_type(&JavaLiteral::Boolean(true), &int));
    assert!(literal_matches_type(
        &JavaLiteral::Utf16Units(vec![0xd800]),
        &string
    ));
    assert!(!literal_matches_type(
        &JavaLiteral::InternalNull(JavaNullPurpose::AbsentTaggedPayload),
        &long
    ));
    assert!(unary_signature_matches(
        JavaUnaryOperator::Not,
        &boolean,
        &boolean
    ));
    assert!(!unary_signature_matches(
        JavaUnaryOperator::Not,
        &int,
        &boolean
    ));
    assert!(binary_signature_matches(
        JavaBinaryOperator::Add,
        &int,
        &int,
        &int
    ));
    assert!(!binary_signature_matches(
        JavaBinaryOperator::Add,
        &int,
        &long,
        &long
    ));
    assert!(!binary_signature_matches(
        JavaBinaryOperator::Add,
        &string,
        &string,
        &string
    ));
    assert!(!binary_signature_matches(
        JavaBinaryOperator::ShiftLeft,
        &long,
        &long,
        &long
    ));
}
