use super::*;

pub(super) fn typed_fixture_manifests() -> [OutputManifest; 3] {
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
        let builder = added.builder.record(
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
        );
        builder.enumeration(
            portable_name!("TrafficLight"),
            typed_list![
                variant(portable_name!("RED")),
                variant(portable_name!("AMBER")),
                variant(portable_name!("GREEN")),
            ],
            |builder, traffic_light| {
                let builder = builder
                    .function(
                        portable_name!("stop_light"),
                        typed_list![],
                        traffic_light.ty(),
                        |body, _| body.enum_variant(&traffic_light, traffic_light.variants().head),
                    )
                    .builder;
                let builder = builder
                    .function(
                        portable_name!("stop_light_is_red"),
                        typed_list![parameter(portable_name!("value"), traffic_light.ty(),)],
                        portable_build::Bool::TYPE,
                        |body, values| {
                            let left = body.read(values.head);
                            let right =
                                body.enum_variant(&traffic_light, traffic_light.variants().head);
                            body.equal(left, right)
                        },
                    )
                    .builder;
                builder
                    .function(
                        portable_name!("traffic_light_priority"),
                        typed_list![parameter(portable_name!("value"), traffic_light.ty())],
                        I32::TYPE,
                        |body, values| {
                            let value = body.read(values.head);
                            let red = body.i32(3);
                            let amber = body.i32(2);
                            let green = body.i32(1);
                            body.enum_match(
                                &traffic_light,
                                value,
                                typed_list![
                                    enum_arm(traffic_light.variants().head, red),
                                    enum_arm(traffic_light.variants().tail.head, amber),
                                    enum_arm(traffic_light.variants().tail.tail.head, green,),
                                ],
                            )
                        },
                    )
                    .builder
            },
        )
    });
    [
        JavaBackend
            .generate_typed(&program)
            .expect("Java resource capacity"),
        JavaBackend
            .generate_typed(&program)
            .expect("Java resource capacity"),
        JavaBackend
            .generate_typed(&program)
            .expect("Java resource capacity"),
    ]
}
