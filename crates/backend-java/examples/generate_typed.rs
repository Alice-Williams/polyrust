use std::path::PathBuf;

use portable_backend_java::JavaBackend;
use portable_build::{I32, field, parameter, portable_name, typed_list, typed_program};
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
                builder
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
