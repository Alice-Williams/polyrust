//! Typed interface identities survive record accessor and generic-erasure collisions.

use super::totality_oracle::CompiledPackage;
use crate::JavaBackend;
use portable_build::{
    Bool, I32, Text, field, interface_method, list_type, method_binding, parameter, portable_name,
    typed_list, typed_program,
};

#[test]
fn typed_interface_methods_do_not_merge_with_record_accessors_or_each_other() {
    let program = typed_program(portable_name!("interface_names"), |builder| {
        builder.record(
            portable_name!("Datum"),
            typed_list![field(portable_name!("read"), I32::TYPE)],
            |builder, record| {
                builder.interface(
                    portable_name!("First"),
                    typed_list![interface_method(
                        portable_name!("read"),
                        typed_list![],
                        I32::TYPE
                    )],
                    |builder, first| {
                        builder.interface(
                            portable_name!("Second"),
                            typed_list![interface_method(
                                portable_name!("read"),
                                typed_list![],
                                Bool::TYPE
                            )],
                            |builder, second| {
                                let binding = method_binding(
                                    &record,
                                    &first.methods().head,
                                    portable_name!("first_read"),
                                    |body, _, _| body.i32(13),
                                );
                                builder.implementation(
                                    portable_name!("FirstForDatum"),
                                    &first,
                                    &record,
                                    typed_list![binding],
                                    |builder, first_impl| {
                                        let binding = method_binding(
                                            &record,
                                            &second.methods().head,
                                            portable_name!("second_read"),
                                            |body, _, _| body.bool(true),
                                        );
                                        builder.implementation(
                                            portable_name!("SecondForDatum"),
                                            &second,
                                            &record,
                                            typed_list![binding],
                                            |builder, second_impl| {
                                                let builder = builder
                                                    .function(
                                                        portable_name!("concrete_first"),
                                                        typed_list![parameter(
                                                            portable_name!("value"),
                                                            record.ty()
                                                        )],
                                                        I32::TYPE,
                                                        |body, parameters| {
                                                            let value = body.read(parameters.head);
                                                            body.concrete_method(
                                                                &first_impl,
                                                                first_impl.methods().head,
                                                                value,
                                                                typed_list![],
                                                            )
                                                        },
                                                    )
                                                    .builder;
                                                builder
                                                    .function(
                                                        portable_name!("dynamic_second"),
                                                        typed_list![parameter(
                                                            portable_name!("value"),
                                                            record.ty()
                                                        )],
                                                        Bool::TYPE,
                                                        |body, parameters| {
                                                            let value = body.read(parameters.head);
                                                            let value = body.interface_value(
                                                                &second_impl,
                                                                value,
                                                            );
                                                            body.interface_method(
                                                                &second,
                                                                &second.methods().head,
                                                                value,
                                                                typed_list![],
                                                            )
                                                        },
                                                    )
                                                    .builder
                                            },
                                        )
                                    },
                                )
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
    CompiledPackage::new(&manifest, "typed-interface-names").consumer(r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        Generated.Datum value = new Generated.Datum(11);
        Generated.First first = value;
        Generated.Second second = value;
        if (value.read() != 11 || first.read_1().value() != 13 || !second.read_2().value()
            || Generated.concrete_first(value).value() != 13 || !Generated.dynamic_second(value).value()) {
            throw new AssertionError("typed interface identity changed");
        }
    }
}
"#);
}

#[test]
fn typed_generic_erasure_preserves_different_parameter_and_result_types() {
    let program = typed_program(portable_name!("erased_names"), |builder| {
        builder.record(portable_name!("Datum"), typed_list![], |builder, record| {
            builder.interface(
                portable_name!("Numbers"),
                typed_list![interface_method(
                    portable_name!("inspect"),
                    typed_list![parameter(portable_name!("items"), list_type(I32::TYPE))],
                    I32::TYPE
                )],
                |builder, numbers| {
                    builder.interface(
                        portable_name!("Words"),
                        typed_list![interface_method(
                            portable_name!("inspect"),
                            typed_list![parameter(portable_name!("items"), list_type(Text::TYPE))],
                            Bool::TYPE
                        )],
                        |builder, words| {
                            let binding = method_binding(
                                &record,
                                &numbers.methods().head,
                                portable_name!("inspect_numbers"),
                                |body, _, parameters| {
                                    let items = body.read(parameters.head);
                                    let zero = body.i64(0);
                                    body.list_get_checked(items, zero)
                                },
                            );
                            builder.implementation(
                                portable_name!("NumbersForDatum"),
                                &numbers,
                                &record,
                                typed_list![binding],
                                |builder, _| {
                                    let binding = method_binding(
                                        &record,
                                        &words.methods().head,
                                        portable_name!("inspect_words"),
                                        |body, _, parameters| {
                                            let items = body.read(parameters.head);
                                            let expected = body.text("yes");
                                            body.list_contains(items, expected)
                                        },
                                    );
                                    builder.implementation(
                                        portable_name!("WordsForDatum"),
                                        &words,
                                        &record,
                                        typed_list![binding],
                                        |builder, _| builder,
                                    )
                                },
                            )
                        },
                    )
                },
            )
        })
    });
    let manifest = JavaBackend
        .generate_typed(&program)
        .expect("Java resource capacity");
    CompiledPackage::new(&manifest, "typed-erased-signatures").consumer(
        r#"
package org.polyrust.consumer;
import java.util.List;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        Generated.Datum value = new Generated.Datum();
        Generated.Numbers numbers = value;
        Generated.Words words = value;
        if (numbers.inspect(List.of(17)).value() != 17 || !words.inspect_1(List.of("yes")).value()
            || value.inspect(List.of(23)).value() != 23 || value.inspect_1(List.of("no")).value()) {
            throw new AssertionError("erased methods were merged");
        }
    }
}
"#,
    );
}
