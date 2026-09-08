//! Known constructor metadata is owned by value mappings, not an operand hole.
use super::*;
use crate::{ast::*, capabilities as c};
use c::support::JavaValueNode;
fn verify<M>(mapping: M, input: M::Input)
where
    M: JavaCapabilityMapping<Context = (), Output = JavaValueNode> + 'static,
    M::Input: Clone,
    c::JavaCapabilitySet:
        Supports<M::Capability, Dialect = JavaDialect, Mapping = c::support::CheckedJavaMapping<M>>,
{
    for wrong_owner in [false, true] {
        let (plan, mut output) = checked(mapping, input.clone());
        let JavaValueNode::Expression(value) = &mut output else {
            panic!()
        };
        let JavaExprKind::New {
            constructor:
                JavaConstructorRef::Known {
                    owner, parameters, ..
                },
            ..
        } = &mut value.kind
        else {
            panic!()
        };
        if wrong_owner {
            *owner = JavaType::known(JavaKnownType::String);
        } else {
            parameters.push(JavaType::primitive(JavaPrimitive::Long));
        }
        assert!(!plan.verify_output(&output));
    }
}
#[test]
fn scalar_and_unit_constructor_owner_and_signature_are_exact() {
    verify(
        c::JavaCharValues,
        c::char_values::JavaCharValuesInput::Value('x'),
    );
    verify(
        c::JavaUnitValues,
        c::unit_values::JavaUnitValuesInput::Value,
    );
}
