//! Java source names must also be unique after JVM nesting uses '$'.

use super::{fixture_declaration, verify_fixture};
use crate::ast::{JavaIdentifier, JavaMember, JavaModifier, JavaVisibility};
use crate::dialect::JavaDialect;

#[test]
fn nested_and_dollar_spelled_types_cannot_share_a_binary_name() {
    for collision in [false, true] {
        let mut nested = fixture_declaration(vec![]);
        nested.name = JavaIdentifier::new("Inner").unwrap();
        nested.visibility = JavaVisibility::Public;
        nested.modifiers = vec![JavaModifier::Static];
        let outer = fixture_declaration(vec![JavaMember::NestedType(nested)]);
        let mut flat = fixture_declaration(vec![]);
        flat.name = JavaIdentifier::new(if collision {
            "Fixture$Inner"
        } else {
            "Fixture$Other"
        })
        .unwrap();
        let result = verify_fixture(
            portable_codegen::TargetAstBuilder::new(JavaDialect),
            vec![(vec![], outer), (vec![], flat)],
        );
        assert_eq!(result.is_ok(), !collision, "{result:?}");
    }
}
