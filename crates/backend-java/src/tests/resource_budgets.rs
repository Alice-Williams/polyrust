//! Conservative reservations are separate from exact JVM encoding checks.

use super::tests::{declaration, item, method};
use super::{budget, check};
use crate::ast::{
    JavaBlock, JavaExpr, JavaIdentifier, JavaLiteral, JavaMember, JavaPrimitive, JavaStmt, JavaType,
};

#[test]
fn oversized_method_and_aggregate_class_are_rejected_as_resources() {
    for count in [10, 20_000] {
        let JavaMember::Method(mut value) =
            method(JavaType::primitive(JavaPrimitive::Int), 0, false)
        else {
            unreachable!()
        };
        value.body = Some(JavaBlock::new(
            (0..count)
                .map(|_| JavaStmt::If {
                    condition: JavaExpr::literal(
                        JavaType::primitive(JavaPrimitive::Boolean),
                        JavaLiteral::Boolean(true),
                    ),
                    then_block: JavaBlock::new(vec![]),
                    else_block: None,
                })
                .collect(),
        ));
        let file = item(declaration(vec![JavaMember::Method(value)]));
        let result = check(vec![("Fixture.java", vec![&file])]);
        assert_eq!(result.is_ok(), count == 10);
        if let Err(errors) = result {
            assert!(errors.iter().any(|error| {
                error
                    .message
                    .contains("conservative method code bytes budget")
            }));
        }
    }
    for count in [10, 65_535] {
        let members = (0..count)
            .map(|index| {
                let JavaMember::Method(mut value) =
                    method(JavaType::primitive(JavaPrimitive::Int), 0, false)
                else {
                    unreachable!()
                };
                value.name = JavaIdentifier::new(format!("method{index}")).unwrap();
                JavaMember::Method(value)
            })
            .collect();
        let file = item(declaration(members));
        let result = check(vec![("Fixture.java", vec![&file])]);
        assert_eq!(result.is_ok(), count == 10);
        if let Err(errors) = result {
            assert!(
                errors
                    .iter()
                    .any(|error| error.message.contains("conservative method budget"))
            );
            assert!(
                errors
                    .iter()
                    .any(|error| error.message.contains("conservative constant-pool budget"))
            );
        }
    }
}

#[test]
fn nested_class_reservations_remain_independent() {
    let mut outer = declaration(vec![JavaMember::NestedType(declaration(vec![]))]);
    outer.name = JavaIdentifier::new("Outer").unwrap();
    let reports = budget::report(&outer, &outer.members.iter().collect::<Vec<_>>(), 2);
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].name, "Outer$Fixture");
    assert_eq!(reports[1].name, "Outer");
    assert!(reports.iter().all(|class| class.pool < 1024));
    assert_eq!(reports[0].fields, 1); // Captured enclosing instance.
    assert_eq!(reports[1].fields, 0);
}

#[test]
fn typed_large_byte_array_returns_resource_error_before_rendering() {
    use portable_build::{Bytes, portable_name, typed_list, typed_program};
    for count in [100, 20_000] {
        let program = typed_program(portable_name!("array_capacity"), |builder| {
            builder
                .function(
                    portable_name!("bytes"),
                    typed_list![],
                    Bytes::TYPE,
                    |body, _| body.bytes(vec![0; count]),
                )
                .builder
        });
        let result = crate::JavaBackend.generate_typed(&program);
        assert_eq!(result.is_ok(), count == 100);
        if let Err(error) = result {
            assert!(error.diagnostics().iter().any(|error| {
                error
                    .message
                    .contains("conservative method code bytes budget")
            }));
        }
    }
}
