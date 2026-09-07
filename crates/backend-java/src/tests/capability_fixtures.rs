use super::*;

pub(super) fn capability_coverage_fixture() -> CheckedProgram {
    let mut module = ModuleBuilder::new("java_capability_coverage");
    module.alias("Count", Visibility::Public, vec![], Type::i64());
    let truth = module.constant("TRUTH", Visibility::Public, vec![], Type::bool(), |body| {
        body.constant_literal(Value::bool(true))
    });

    let exercise = module.function("exercise", Visibility::Public, vec![], |function| {
        function.returns(Type::unit());
        function.body(|body| {
            let mut statements = Vec::new();

            macro_rules! unary_operation {
                ($operation:expr, $value:expr) => {{
                    let operand = body.literal($value);
                    let expression = body.intrinsic($operation, [operand]);
                    statements.push(body.expression_statement(expression));
                }};
            }
            macro_rules! binary_operation {
                ($operation:expr, $left:expr, $right:expr) => {{
                    let left = body.literal($left);
                    let right = body.literal($right);
                    let expression = body.intrinsic($operation, [left, right]);
                    statements.push(body.expression_statement(expression));
                }};
            }
            macro_rules! ternary_operation {
                ($operation:expr, $first:expr, $second:expr, $third:expr) => {{
                    let first = body.literal($first);
                    let second = body.literal($second);
                    let third = body.literal($third);
                    let expression = body.intrinsic($operation, [first, second, third]);
                    statements.push(body.expression_statement(expression));
                }};
            }

            unary_operation!(Operation::BoolNot, Value::bool(true));
            binary_operation!(Operation::BoolAnd, Value::bool(true), Value::bool(false));
            binary_operation!(Operation::BoolOr, Value::bool(false), Value::bool(true));
            binary_operation!(Operation::Equal, Value::i32(1), Value::i32(1));
            binary_operation!(Operation::NotEqual, Value::i32(1), Value::i32(2));
            binary_operation!(Operation::Less, Value::i32(1), Value::i32(2));
            binary_operation!(Operation::LessEqual, Value::i32(1), Value::i32(2));
            binary_operation!(Operation::Greater, Value::i32(2), Value::i32(1));
            binary_operation!(Operation::GreaterEqual, Value::i32(2), Value::i32(1));
            unary_operation!(Operation::IntNegChecked, Value::i32(2));
            binary_operation!(Operation::IntAddChecked, Value::i32(1), Value::i32(2));
            binary_operation!(Operation::IntSubChecked, Value::i32(2), Value::i32(1));
            binary_operation!(Operation::IntMulChecked, Value::i32(2), Value::i32(3));
            binary_operation!(Operation::IntDivChecked, Value::i32(6), Value::i32(2));
            binary_operation!(Operation::IntRemChecked, Value::i32(7), Value::i32(3));
            unary_operation!(Operation::IntNegWrapping, Value::i32(2));
            binary_operation!(Operation::IntAddWrapping, Value::i32(1), Value::i32(2));
            binary_operation!(Operation::IntSubWrapping, Value::i32(2), Value::i32(1));
            binary_operation!(Operation::IntMulWrapping, Value::i32(2), Value::i32(3));
            unary_operation!(Operation::IntBitNot, Value::i32(2));
            binary_operation!(Operation::IntBitAnd, Value::i32(3), Value::i32(1));
            binary_operation!(Operation::IntBitOr, Value::i32(2), Value::i32(1));
            binary_operation!(Operation::IntBitXor, Value::i32(3), Value::i32(1));
            binary_operation!(Operation::IntShiftLeftChecked, Value::i32(1), Value::i32(2));
            binary_operation!(
                Operation::IntShiftRightChecked,
                Value::i32(4),
                Value::i32(1)
            );
            unary_operation!(Operation::FloatNeg, Value::f64(1.5));
            unary_operation!(Operation::FloatTrunc, Value::f64(1.5));
            unary_operation!(Operation::FloatIsNaN, Value::f64(f64::NAN));
            unary_operation!(Operation::FloatIsNegativeZero, Value::f64(-0.0));
            unary_operation!(Operation::FloatAbs, Value::f64(-1.5));
            binary_operation!(Operation::FloatAdd, Value::f64(1.0), Value::f64(2.0));
            binary_operation!(Operation::FloatSub, Value::f64(2.0), Value::f64(1.0));
            binary_operation!(Operation::FloatMul, Value::f64(2.0), Value::f64(3.0));
            binary_operation!(Operation::FloatDiv, Value::f64(6.0), Value::f64(2.0));
            binary_operation!(Operation::FloatRemTrunc, Value::f64(7.0), Value::f64(3.0));
            binary_operation!(
                Operation::StringConcat,
                Value::string("left"),
                Value::string("right")
            );
            unary_operation!(Operation::StringScalarLength, Value::string("text"));
            unary_operation!(Operation::StringUtf16Length, Value::string("text"));
            binary_operation!(
                Operation::StringIndexOfLiteral,
                Value::string("text"),
                Value::string("ex")
            );
            unary_operation!(Operation::StringIsEmpty, Value::string(""));
            binary_operation!(
                Operation::StringContains,
                Value::string("text"),
                Value::string("ex")
            );
            binary_operation!(
                Operation::StringStartsWith,
                Value::string("text"),
                Value::string("te")
            );
            binary_operation!(
                Operation::StringEndsWith,
                Value::string("text"),
                Value::string("xt")
            );
            binary_operation!(
                Operation::StringStripPrefix,
                Value::string("prefix-value"),
                Value::string("prefix-")
            );
            ternary_operation!(
                Operation::StringSliceScalars,
                Value::string("text"),
                Value::i64(0),
                Value::i64(2)
            );
            ternary_operation!(
                Operation::StringReplaceAll,
                Value::string("text"),
                Value::string("e"),
                Value::string("E")
            );
            ternary_operation!(
                Operation::StringReplaceMany,
                Value::string("text"),
                Value::string("e"),
                Value::string("E")
            );
            binary_operation!(
                Operation::StringTruncateUtf8Bytes,
                Value::string("text"),
                Value::f64(3.0)
            );
            binary_operation!(
                Operation::StringTrimStart,
                Value::string("  text"),
                Value::string(" ")
            );
            binary_operation!(
                Operation::StringTrimEnd,
                Value::string("text  "),
                Value::string(" ")
            );
            binary_operation!(
                Operation::BytesConcat,
                Value::bytes([1, 2]),
                Value::bytes([3, 4])
            );
            ternary_operation!(
                Operation::BytesReplaceAll,
                Value::bytes([1, 2, 1]),
                Value::bytes([1]),
                Value::bytes([9])
            );
            unary_operation!(Operation::BytesLength, Value::bytes([1, 2]));
            unary_operation!(Operation::BytesIsEmpty, Value::bytes([]));
            unary_operation!(Operation::WidenI32ToI64, Value::i32(7));
            unary_operation!(Operation::NarrowI64ToI32Checked, Value::i64(7));
            unary_operation!(Operation::StringToUtf8, Value::string("text"));
            unary_operation!(
                Operation::StringFromUtf8Checked,
                Value::bytes([116, 101, 120, 116])
            );

            for operation in [Operation::ListLength, Operation::ListIsEmpty] {
                let item = body.literal(Value::i32(1));
                let list = body.list(Type::i32(), [item]);
                let expression = body.intrinsic(operation, [list]);
                statements.push(body.expression_statement(expression));
            }
            for operation in [
                Operation::ListGetChecked,
                Operation::ListAppend,
                Operation::ListContains,
                Operation::ListIndexOf,
            ] {
                let item = body.literal(Value::i32(1));
                let list = body.list(Type::i32(), [item]);
                let argument = body.literal(Value::i64(0));
                let argument = if operation == Operation::ListAppend
                    || operation == Operation::ListContains
                    || operation == Operation::ListIndexOf
                {
                    body.literal(Value::i32(1))
                } else {
                    argument
                };
                let expression = body.intrinsic(operation, [list, argument]);
                statements.push(body.expression_statement(expression));
            }
            let left_item = body.literal(Value::i32(1));
            let left_list = body.list(Type::i32(), [left_item]);
            let right_item = body.literal(Value::i32(2));
            let right_list = body.list(Type::i32(), [right_item]);
            let concatenated = body.intrinsic(Operation::ListConcat, [left_list, right_list]);
            statements.push(body.expression_statement(concatenated));

            for operation in [Operation::OptionIsSome, Operation::OptionIsNone] {
                let value = body.literal(Value::i32(1));
                let option = body.some(value);
                let expression = body.intrinsic(operation, [option]);
                statements.push(body.expression_statement(expression));
            }
            let value = body.literal(Value::i32(1));
            let option = body.some(value);
            let fallback = body.literal(Value::i32(2));
            let unwrapped = body.intrinsic(Operation::OptionUnwrapOr, [option, fallback]);
            statements.push(body.expression_statement(unwrapped));

            for operation in [Operation::ResultIsOk, Operation::ResultIsErr] {
                let value = body.literal(Value::i32(1));
                let result = body.ok(value, Type::string());
                let expression = body.intrinsic(operation, [result]);
                statements.push(body.expression_statement(expression));
            }

            let constant = body.constant(truth);
            statements.push(body.expression_statement(constant));

            let bool_value = body.literal(Value::bool(true));
            let bool_not = body.intrinsic(Operation::BoolNot, [bool_value]);
            statements.push(body.expression_statement(bool_not));

            let equal_left = body.literal(Value::i32(1));
            let equal_right = body.literal(Value::i32(1));
            let equal = body.intrinsic(Operation::Equal, [equal_left, equal_right]);
            statements.push(body.expression_statement(equal));

            let less_left = body.literal(Value::i32(1));
            let less_right = body.literal(Value::i32(2));
            let less = body.intrinsic(Operation::Less, [less_left, less_right]);
            statements.push(body.expression_statement(less));

            let checked_left = body.literal(Value::i32(1));
            let checked_right = body.literal(Value::i32(2));
            let checked = body.intrinsic(Operation::IntAddChecked, [checked_left, checked_right]);
            statements.push(body.expression_statement(checked));

            let bit_left = body.literal(Value::i64(1));
            let bit_right = body.literal(Value::i64(2));
            let bitwise = body.intrinsic(Operation::IntBitAnd, [bit_left, bit_right]);
            statements.push(body.expression_statement(bitwise));

            let shift_value = body.literal(Value::i32(1));
            let shift_distance = body.literal(Value::i32(2));
            let shifted = body.intrinsic(
                Operation::IntShiftLeftChecked,
                [shift_value, shift_distance],
            );
            statements.push(body.expression_statement(shifted));

            let float_left = body.literal(Value::f64(1.25));
            let float_right = body.literal(Value::f64(2.5));
            let float_sum = body.intrinsic(Operation::FloatAdd, [float_left, float_right]);
            statements.push(body.expression_statement(float_sum));

            let inspected_float = body.literal(Value::f64(f64::NAN));
            let is_nan = body.intrinsic(Operation::FloatIsNaN, [inspected_float]);
            statements.push(body.expression_statement(is_nan));

            let character = body.literal(Value::char('λ'));
            statements.push(body.expression_statement(character));

            let trim_source = body.literal(Value::string("  value"));
            let trim_chars = body.literal(Value::string(" "));
            let trimmed = body.intrinsic(Operation::StringTrimStart, [trim_source, trim_chars]);
            statements.push(body.expression_statement(trimmed));

            let bytes_left = body.literal(Value::bytes([1, 2]));
            let bytes_right = body.literal(Value::bytes([3, 4]));
            let bytes = body.intrinsic(Operation::BytesConcat, [bytes_left, bytes_right]);
            statements.push(body.expression_statement(bytes));

            let option_value = body.literal(Value::i64(7));
            let absent = body.none(Type::i64());
            statements.push(body.expression_statement(absent));
            let option = body.some(option_value);
            let is_some = body.intrinsic(Operation::OptionIsSome, [option]);
            statements.push(body.expression_statement(is_some));

            let result_value = body.literal(Value::i64(7));
            let error = body.literal(Value::string("failure"));
            let failed = body.err(error, Type::i64());
            statements.push(body.expression_statement(failed));
            let result = body.ok(result_value, Type::string());
            let is_ok = body.intrinsic(Operation::ResultIsOk, [result]);
            statements.push(body.expression_statement(is_ok));

            let narrow = body.literal(Value::i32(7));
            let widened = body.intrinsic(Operation::WidenI32ToI64, [narrow]);
            statements.push(body.expression_statement(widened));

            let utf8_source = body.literal(Value::string("text"));
            let utf8 = body.intrinsic(Operation::StringToUtf8, [utf8_source]);
            statements.push(body.expression_statement(utf8));

            let condition = body.literal(Value::bool(true));
            let then_value = body.literal(Value::unit());
            let then_block = body.block([], Some(then_value));
            let else_value = body.literal(Value::unit());
            let else_block = body.block([], Some(else_value));
            let conditional = body.if_else(condition, then_block, else_block);
            statements.push(body.expression_statement(conditional));

            let list_item = body.literal(Value::i32(1));
            let list = body.list(Type::i32(), [list_item]);
            let loop_item = body.local("item");
            let loop_item_read = body.expression_statement(loop_item);
            let loop_body = body.block([loop_item_read], None);
            statements.push(body.for_each("item", list, loop_body));

            let unit = body.literal(Value::unit());
            body.block(statements, Some(unit))
        });
    });

    module.portable_test(
        "all_operations_execute",
        Visibility::Package,
        vec![],
        Invocation::function(exercise, []),
        Expected::value(TypedValue::new(Type::unit(), Value::unit())),
    );
    module.finish().expect("capability coverage fixture checks")
}

pub(super) fn portable_method_invocation_fixture() -> CheckedProgram {
    let mut module = ModuleBuilder::new("java_portable_method_coverage");
    let (record, field) = module.record("Counter", Visibility::Public, vec![], |record| {
        record.field("value", Type::i32(), vec![])
    });
    let (interface, method) =
        module.interface("Readable", Visibility::Public, vec![], |interface| {
            interface.method("read", vec![], vec![], Some(Type::i32()))
        });
    let (implementation, (implementation_method, ())) = module.implementation(
        "CounterReadable",
        Visibility::Package,
        vec![],
        interface,
        record,
        |implementation| {
            implementation.method("read", method, vec![], |method| {
                method.returns(Type::i32());
                method.body(|body| {
                    let receiver = body.self_value();
                    let value = body.field(receiver, field);
                    body.block([], Some(value))
                });
            })
        },
    );
    module.portable_test(
        "reads_concrete_method",
        Visibility::Package,
        vec![],
        Invocation::method(
            implementation,
            implementation_method,
            TypedValue::new(
                Type::named(record),
                Value::record(record, [(field, Value::i32(9))]),
            ),
            [],
        ),
        Expected::value(TypedValue::new(Type::i32(), Value::i32(9))),
    );
    module
        .finish()
        .expect("portable method coverage fixture checks")
}
