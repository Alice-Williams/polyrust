use std::path::PathBuf;

use portable_backend_java::JavaBackend;
use portable_build::{I32, Text, field, parameter, portable_name, typed_list, typed_program};
use portable_codegen::OutputContents;

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let output = PathBuf::from(arguments.next().expect("output path"));
    assert!(arguments.next().is_none(), "unexpected argument");

    let program = typed_program(portable_name!("java_inferred"), |builder| {
        let added = builder.function(
            portable_name!("compute"),
            typed_list![
                parameter(portable_name!("left"), I32::TYPE),
                parameter(portable_name!("right"), I32::TYPE),
                parameter(portable_name!("scale"), I32::TYPE),
            ],
            I32::TYPE,
            |body, values| {
                let sum_left = body.read(values.head.clone());
                let sum_right = body.read(values.tail.head.clone());
                let sum = body.int_add_wrapping(sum_left, sum_right);
                let difference_left = body.read(values.head);
                let difference_right = body.read(values.tail.head);
                let difference = body.int_sub_wrapping(difference_left, difference_right);
                let product = body.int_mul_wrapping(sum, difference);
                let scale = body.read(values.tail.tail.head);
                body.int_add_wrapping(product, scale)
            },
        );
        let compute = added.handle;
        added.builder.record(
            portable_name!("Point3"),
            typed_list![
                field(portable_name!("x"), I32::TYPE),
                field(portable_name!("y"), I32::TYPE),
                field(portable_name!("z"), I32::TYPE),
            ],
            |builder, point| {
                let builder = builder
                    .function(
                        portable_name!("make_point"),
                        typed_list![
                            parameter(portable_name!("x"), I32::TYPE),
                            parameter(portable_name!("y"), I32::TYPE),
                            parameter(portable_name!("z"), I32::TYPE),
                        ],
                        point.ty(),
                        |body, values| {
                            let x = body.read(values.head);
                            let y = body.read(values.tail.head);
                            let z = body.read(values.tail.tail.head);
                            body.construct(&point, typed_list![x, y, z])
                        },
                    )
                    .builder;
                let builder = builder
                    .function(
                        portable_name!("computed"),
                        typed_list![],
                        I32::TYPE,
                        |body, _| {
                            let left = body.i32(7);
                            let right = body.i32(2);
                            let scale = body.i32(5);
                            body.call(compute, typed_list![left, right, scale])
                        },
                    )
                    .builder;
                builder
                    .function(
                        portable_name!("extended_features"),
                        typed_list![],
                        portable_build::Bool::TYPE,
                        |body, _| {
                            let value = body.i32(12);
                            let value = body.int_bit_not(value);
                            let mask = body.i32(7);
                            let value = body.int_bit_and(value, mask);
                            let distance = body.i32(1);
                            let shifted = body.int_shift_left_checked(value, distance);
                            let expected = body.i32(6);
                            let integer_ok = body.equal(shifted, expected);

                            let negative = body.f64(-1.75);
                            let absolute = body.float_abs(negative);
                            let nan = body.float_is_nan(absolute);
                            let float_ok = body.bool_not(nan);
                            let result = body.bool_and(integer_ok, float_ok);

                            let source = body.text("poly-rust");
                            let needle = body.text("-");
                            let replacement = body.text(" ");
                            let transformed = body.string_replace_all(source, needle, replacement);
                            let suffix = body.text("rust");
                            let string_ok = body.string_ends_with(transformed, suffix);
                            let result = body.bool_and(result, string_ok);

                            let text = body.text("bytes");
                            let encoded = body.string_to_utf8(text);
                            let suffix = body.bytes(vec![33]);
                            let bytes = body.bytes_concat(encoded, suffix);
                            let length = body.bytes_length(bytes);
                            let expected = body.i64(6);
                            let bytes_ok = body.equal(length, expected);
                            let result = body.bool_and(result, bytes_ok);

                            let one = body.i32(1);
                            let list = body.list(I32::TYPE, typed_list![one]);
                            let two = body.i32(2);
                            let list = body.list_append(list, two);
                            let two = body.i32(2);
                            let list_ok = body.list_contains(list, two);
                            let result = body.bool_and(result, list_ok);

                            let seven = body.i32(7);
                            let some = body.some(seven);
                            let fallback = body.i32(0);
                            let unwrapped = body.option_unwrap_or(some, fallback);
                            let seven = body.i32(7);
                            let option_ok = body.equal(unwrapped, seven);
                            let result = body.bool_and(result, option_ok);

                            let value = body.i32(7);
                            let ok = body.ok(value, Text::TYPE);
                            let result_ok = body.result_is_ok(ok);
                            body.bool_and(result, result_ok)
                        },
                    )
                    .builder
            },
        )
    });

    let manifest = JavaBackend.generate_typed(&program);
    for file in manifest.files() {
        let path = output.join(file.path());
        std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        match file.contents() {
            OutputContents::Text(text) => std::fs::write(path, text),
            OutputContents::Bytes(bytes) => std::fs::write(path, bytes),
        }
        .expect("write output");
    }
}
