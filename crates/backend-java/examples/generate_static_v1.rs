use std::path::PathBuf;

use portable_backend_java::JavaBackend;
use portable_build::{I32, StaticV1, portable_name, static_program};
use portable_codegen::OutputContents;

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let output = PathBuf::from(arguments.next().expect("output path"));
    assert!(arguments.next().is_none(), "unexpected argument");

    let program = static_program::<StaticV1>(portable_name!("java_static_v1"), |module| {
        let compute = module.function2(
            portable_name!("compute"),
            (portable_name!("left"), I32::TYPE),
            (portable_name!("right"), I32::TYPE),
            I32::TYPE,
            |body, left, right| {
                let sum_left = body.read(left.clone());
                let sum_right = body.read(right.clone());
                let sum = body.int_add_wrapping(sum_left, sum_right);
                let difference_left = body.read(left);
                let difference_right = body.read(right);
                let difference = body.int_sub_wrapping(difference_left, difference_right);
                body.int_mul_wrapping(sum, difference)
            },
        );

        module.record2(
            portable_name!("Point"),
            (portable_name!("x"), I32::TYPE),
            (portable_name!("y"), I32::TYPE),
            |module, point| {
                module.function2(
                    portable_name!("make_point"),
                    (portable_name!("x"), I32::TYPE),
                    (portable_name!("y"), I32::TYPE),
                    point.ty(),
                    |body, x, y| {
                        let x = body.read(x);
                        let y = body.read(y);
                        body.construct2(point, x, y)
                    },
                );
                module.function0(portable_name!("computed"), I32::TYPE, |body| {
                    let left = body.i32(7);
                    let right = body.i32(2);
                    body.call2(compute, left, right)
                });
            },
        );
    });

    let manifest = JavaBackend.generate_static(&program);
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
