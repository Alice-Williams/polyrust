//! Typed names are portable identities, not Java member spellings.

use super::totality_oracle::CompiledPackage;
use crate::JavaBackend;
use portable_build::variant;
use portable_build::{I32, field, parameter, portable_name, typed_list, typed_program};

#[test]
fn accessor_suffixes_respect_requested_interface_method_names() {
    let program = typed_program(portable_name!("accessor_suffixes"), |builder| {
        builder.record(
            portable_name!("R"),
            typed_list![field(portable_name!("Runtime"), I32::TYPE)],
            |builder, _| {
                builder.interface(
                    portable_name!("Readable"),
                    typed_list![portable_build::interface_method(
                        portable_name!("Runtime_1"),
                        typed_list![],
                        I32::TYPE
                    )],
                    |builder, _| builder,
                )
            },
        )
    });
    let manifest = JavaBackend
        .generate_typed(&program)
        .expect("Java resource capacity");
    CompiledPackage::new(&manifest, "accessor-suffix-reservation").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) throws ReflectiveOperationException {
        if (new Generated.R(7).Runtime_2() != 7) throw new AssertionError();
        Generated.Readable.class.getDeclaredMethod("Runtime_1");
    }
}
"#,
    );
}

#[test]
fn known_static_call_qualifiers_survive_field_and_parameter_names() {
    use portable_build::F64;
    let program = typed_program(portable_name!("known_qualifiers"), |builder| {
        builder.interface(portable_name!("Empty"), typed_list![], |builder, empty| {
            builder.record(
                portable_name!("Holder"),
                typed_list![field(portable_name!("Objects"), empty.ty())],
                |builder, _| {
                    builder
                        .function(
                            portable_name!("truncate"),
                            typed_list![parameter(portable_name!("Math"), F64::TYPE)],
                            F64::TYPE,
                            |body, parameters| {
                                let value = body.read(parameters.head);
                                body.float_trunc(value)
                            },
                        )
                        .builder
                },
            )
        })
    });
    let manifest = JavaBackend
        .generate_typed(&program)
        .expect("Java resource capacity");
    CompiledPackage::new(&manifest, "known-static-qualifier-collisions").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        if (Generated.truncate(7.9).value() != 7.0) throw new AssertionError();
        try {
            new Generated.Holder(null);
            throw new AssertionError("null crossed interface boundary");
        } catch (NullPointerException expected) {
            // Objects.requireNonNull must still name the standard-library class.
        }
    }
}
"#,
    );
}

#[test]
fn expression_qualifiers_cannot_be_shadowed_by_portable_value_names() {
    let program = typed_program(portable_name!("qualifier_names"), |builder| {
        builder.record(portable_name!("org"), typed_list![], |builder, _| {
            builder.record(portable_name!("java"), typed_list![], |builder, _| {
                builder.record(
                    portable_name!("Box"),
                    typed_list![
                        field(portable_name!("Objects"), portable_build::Text::TYPE),
                        field(portable_name!("Runtime"), I32::TYPE),
                    ],
                    |builder, _| {
                        builder
                            .function(
                                portable_name!("echo"),
                                typed_list![parameter(portable_name!("Runtime"), I32::TYPE),],
                                I32::TYPE,
                                |body, parameters| body.read(parameters.head),
                            )
                            .builder
                    },
                )
            })
        })
    });
    let manifest = JavaBackend
        .generate_typed(&program)
        .expect("Java resource capacity");
    CompiledPackage::new(&manifest, "expression-qualifier-collisions").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        if (Generated.echo(7).value() != 7) throw new AssertionError();
    }
}
"#,
    );
}

#[test]
fn annotation_names_are_reserved_from_the_typed_annotation_catalogue() {
    let program = typed_program(portable_name!("annotation_names"), |builder| {
        builder.record(portable_name!("Override"), typed_list![], |builder, _| {
            builder.record(
                portable_name!("SafeVarargs"),
                typed_list![],
                |builder, _| builder,
            )
        })
    });
    let manifest = JavaBackend
        .generate_typed(&program)
        .expect("Java resource capacity");
    CompiledPackage::new(&manifest, "annotation-name-collisions").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        new Generated.Override_1();
        new Generated.SafeVarargs_1();
    }
}
"#,
    );
}

#[test]
fn typed_object_names_preserve_function_field_and_projection_identity() {
    let program = typed_program(portable_name!("object_names"), |builder| {
        let builder = builder
            .function(
                portable_name!("hashCode"),
                typed_list![],
                I32::TYPE,
                |body, _| body.i32(7),
            )
            .builder;
        builder.record(
            portable_name!("Named"),
            typed_list![
                field(portable_name!("hashCode"), I32::TYPE),
                field(portable_name!("hashCode_1"), I32::TYPE),
            ],
            |builder, record| {
                builder
                    .function(
                        portable_name!("project"),
                        typed_list![parameter(portable_name!("value"), record.ty())],
                        I32::TYPE,
                        |body, parameters| {
                            let value = body.read(parameters.head);
                            body.field(value, record.fields().head)
                        },
                    )
                    .builder
            },
        )
    });
    let manifest = JavaBackend
        .generate_typed(&program)
        .expect("Java resource capacity");
    assert_eq!(
        manifest.canonical_json(),
        JavaBackend
            .generate_typed(&program)
            .expect("Java resource capacity")
            .canonical_json()
    );
    CompiledPackage::new(&manifest, "typed-object-names").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        Generated.Named value = new Generated.Named(11, 13);
        if (Generated.hashCode_1().value() != 7 || value.hashCode_2() != 11
            || value.hashCode_1() != 13 || Generated.project(value).value() != 11) {
            throw new AssertionError("allocated names changed identity");
        }
    }
}

"#,
    );
}

#[test]
fn typed_generated_names_and_contextual_keywords_keep_distinct_identities() {
    let program = typed_program(portable_name!("all_name_spaces"), |builder| {
        let builder = builder
            .function(
                portable_name!("keywords"),
                typed_list![
                    parameter(portable_name!("exports"), I32::TYPE),
                    parameter(portable_name!("exports_"), I32::TYPE),
                ],
                I32::TYPE,
                |body, parameters| {
                    let left = body.read(parameters.head);
                    let right = body.read(parameters.tail.head);
                    body.int_sub_wrapping(left, right)
                },
            )
            .builder;
        builder.record(
            portable_name!("Generated"),
            typed_list![field(portable_name!("value"), I32::TYPE)],
            |builder, record| {
                let builder = builder
                    .function(
                        portable_name!("make"),
                        typed_list![],
                        record.ty(),
                        |body, _| {
                            let value = body.i32(31);
                            body.construct(&record, typed_list![value])
                        },
                    )
                    .builder;
                builder.enumeration(
                    portable_name!("First"),
                    typed_list![variant(portable_name!("SAME"))],
                    |builder, first| {
                        let builder = builder
                            .function(
                                portable_name!("first"),
                                typed_list![],
                                first.ty(),
                                |body, _| body.enum_variant(&first, first.variants().head),
                            )
                            .builder;
                        builder.enumeration(
                            portable_name!("Second"),
                            typed_list![variant(portable_name!("SAME"))],
                            |builder, second| {
                                builder
                                    .function(
                                        portable_name!("second"),
                                        typed_list![],
                                        second.ty(),
                                        |body, _| {
                                            body.enum_variant(&second, second.variants().head)
                                        },
                                    )
                                    .builder
                            },
                        )
                    },
                )
            },
        )
    });
    let manifest = JavaBackend
        .generate_typed(&program)
        .expect("Java resource capacity");
    CompiledPackage::new(&manifest, "typed-all-name-spaces").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        Generated.Generated_1 value = Generated.make().value();
        if (value.value() != 31 || Generated.keywords(19, 4).value() != 15
            || Generated.first().value() != Generated.First.SAME
            || Generated.second().value() != Generated.Second.SAME_1) {
            throw new AssertionError("normalized target names merged");
        }
    }
}
"#,
    );
}
