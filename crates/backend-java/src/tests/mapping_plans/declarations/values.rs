use super::*;
fn reject_wrong_category<M>(mapping: M, input: M::Input)
where
    M: JavaCapabilityMapping<Context = (), Output = JavaValueNode> + 'static,
    c::JavaCapabilitySet:
        Supports<M::Capability, Dialect = JavaDialect, Mapping = c::support::CheckedJavaMapping<M>>,
{
    let (plan, output) = checked(mapping, input);
    let wrong = match output {
        JavaValueNode::Type(_) => JavaValueNode::Expression(Box::new(i32_literal(1))),
        JavaValueNode::Expression(_) => {
            JavaValueNode::Type(JavaType::primitive(JavaPrimitive::Int))
        }
    };
    assert!(!plan.verify_output(&wrong));
}
#[test]
fn all_value_capabilities_reject_type_expression_swaps() {
    macro_rules! scalar {
        ($mapping:expr, $input:path, $value:expr) => {{
            use $input as I;
            reject_wrong_category($mapping, I::Type);
            reject_wrong_category($mapping, I::Value($value));
        }};
    }
    scalar!(c::JavaBoolValues, c::bool_values::JavaBoolValuesInput, true);
    scalar!(c::JavaI32Values, c::i32_values::JavaI32ValuesInput, 1);
    scalar!(c::JavaI64Values, c::i64_values::JavaI64ValuesInput, 1);
    scalar!(
        c::JavaF64Values,
        c::f64_values::JavaF64ValuesInput,
        1.0_f64.to_bits()
    );
    scalar!(
        c::JavaTextValues,
        c::text_values::JavaTextValuesInput,
        "text".to_owned()
    );
    scalar!(c::JavaCharValues, c::char_values::JavaCharValuesInput, 'a');
    reject_wrong_category(c::JavaUnitValues, c::unit_values::JavaUnitValuesInput::Type);
    reject_wrong_category(
        c::JavaUnitValues,
        c::unit_values::JavaUnitValuesInput::Value,
    );
    let int = JavaType::primitive(JavaPrimitive::Int);
    reject_wrong_category(c::JavaBytesValues, c::bytes_values::JavaBytesInput::Type);
    reject_wrong_category(
        c::JavaBytesValues,
        c::bytes_values::JavaBytesInput::Value {
            values: vec![1],
            result: JavaType::known(JavaKnownType::RuntimeBytes),
        },
    );
    reject_wrong_category(
        c::JavaListValues,
        c::list_values::JavaListInput::Type {
            element: int.clone(),
        },
    );
    reject_wrong_category(
        c::JavaListValues,
        c::list_values::JavaListInput::Value {
            elements: vec![i32_literal(1)],
            result: JavaType::generic(JavaKnownType::List, vec![int.clone().boxed()]),
        },
    );
    reject_wrong_category(
        c::JavaOptionValues,
        c::option_values::JavaOptionInput::Type { inner: int.clone() },
    );
    let option = JavaType::generic(JavaKnownType::RuntimeOption, vec![int.clone().boxed()]);
    reject_wrong_category(
        c::JavaOptionValues,
        c::option_values::JavaOptionInput::None {
            result: option.clone(),
        },
    );
    reject_wrong_category(
        c::JavaOptionValues,
        c::option_values::JavaOptionInput::Some {
            value: Box::new(i32_literal(1)),
            result: option,
        },
    );
    reject_wrong_category(
        c::JavaResultValues,
        c::result_values::JavaResultInput::Type {
            ok: int.clone(),
            error: int.clone(),
        },
    );
    let result = JavaType::generic(
        JavaKnownType::RuntimeValueResult,
        vec![int.clone().boxed(), int.boxed()],
    );
    reject_wrong_category(
        c::JavaResultValues,
        c::result_values::JavaResultInput::Ok {
            value: i32_literal(1),
            result: result.clone(),
        },
    );
    reject_wrong_category(
        c::JavaResultValues,
        c::result_values::JavaResultInput::Err {
            value: i32_literal(1),
            result,
        },
    );
}
