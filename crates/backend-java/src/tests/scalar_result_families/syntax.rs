use super::{expressions::*, fixture::*};
use crate::ast::*;
use crate::dialect::JavaDialect;
use crate::tests::source_record_fixture::{boolean, int, name};
use portable_codegen::*;

#[test]
fn scalar_result_generated_upcast_rejects_checked_core_interface_origin() {
    use portable_build::{ModuleBuilder, Visibility};
    use portable_core_ir::CoreDeclaration;
    let mut module = ModuleBuilder::new("core_origin_boundary");
    module.interface("Empty", Visibility::Public, vec![], |_| ());
    let checked = module.finish().unwrap();
    let core = portable_core_ir::lower_checked(&checked).unwrap();
    let interface = core
        .module()
        .declarations
        .iter()
        .find_map(|value| match value {
            CoreDeclaration::Interface(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    // A real checked Core identity does not authorize a synthesized-adapter edge.
    let fixture = Fixture::with_interface_origin(GeneratedOrigin::CoreDeclaration(
        CoreDeclaration::Interface(interface),
    ));
    let errors = verify_unresolved_package(&JavaDialect, fixture.finish()).unwrap_err();
    assert!(
        errors.iter().any(|error| error
            .message
            .contains("interface coercion disagrees with conformance generated adapter")),
        "{errors:?}"
    );
}

#[test]
fn scalar_result_generated_upcast_cannot_claim_non_adapter_origin() {
    let fixture = Fixture::with_interface_reason(SynthesisReason::TestHarness);
    let errors = verify_unresolved_package(&JavaDialect, fixture.finish()).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("interface coercion disagrees")),
        "{errors:?}"
    );
}

#[test]
fn scalar_result_family_syntax_rejects_changed_inventory_storage_and_constructors() {
    for mutation in 0..6 {
        let mut fixture = Fixture::new();
        match mutation {
            0 => {
                fixture.family.interface.permits.pop();
            }
            1 => fixture.family.error.heritage = JavaHeritage::None,
            2 => {
                fixture.family.success.record_components[0].origin =
                    JavaRecordComponentOrigin::Synthesized(payload(fixture.other.types.success))
            }
            3 => {
                fixture.family.success.record_components[0].ty =
                    JavaType::primitive(JavaPrimitive::Long)
            }
            4 => fixture
                .family
                .success
                .members
                .push(JavaMember::Field(JavaField {
                    declared: None,
                    modifiers: vec![JavaModifier::Public],
                    ty: int(),
                    name: name("mutable"),
                    initializer: None,
                })),
            5 => {
                let JavaMember::Constructor(constructor) = &mut fixture.family.success.members[0]
                else {
                    unreachable!()
                };
                constructor.parameters[0].ty = boolean();
            }
            _ => unreachable!(),
        }
        assert!(
            verify_unresolved_package(&JavaDialect, fixture.finish()).is_err(),
            "mutation {mutation}"
        );
    }
}
