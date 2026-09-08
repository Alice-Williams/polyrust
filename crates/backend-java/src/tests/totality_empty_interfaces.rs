//! Unimplemented interfaces are valid portable types and cannot acquire foreign instances.

use super::totality_oracle::CompiledPackage;
use crate::JavaBackend;
use portable_build::{
    I32, Text, interface_method, list_type, option_type, parameter, portable_name, typed_list,
    typed_program,
};

#[test]
fn typed_empty_interfaces_compile_without_exposing_inhabitants() {
    let program = typed_program(portable_name!("empty_interfaces"), |builder| {
        builder.interface(portable_name!("Empty"), typed_list![], |builder, empty| {
            let builder = builder
                .function(
                    portable_name!("empty_values"),
                    typed_list![],
                    list_type(empty.ty()),
                    |body, _| body.list(empty.ty(), typed_list![]),
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("echo"),
                    typed_list![parameter(portable_name!("value"), empty.ty())],
                    empty.ty(),
                    |body, parameters| body.read(parameters.head),
                )
                .builder;
            let builder = builder
                .function(
                    portable_name!("absent"),
                    typed_list![],
                    option_type(empty.ty()),
                    |body, _| body.none(empty.ty()),
                )
                .builder;
            builder.interface(
                portable_name!("Unavailable"),
                typed_list![
                    interface_method(portable_name!("read"), typed_list![], I32::TYPE),
                    interface_method(
                        portable_name!("transform"),
                        typed_list![parameter(portable_name!("input"), list_type(Text::TYPE))],
                        list_type(Text::TYPE)
                    ),
                ],
                |builder, _| builder,
            )
        })
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
    let compiled = CompiledPackage::new(&manifest, "typed-empty-interfaces");
    compiled.consumer(r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        if (!Generated.empty_values().value().isEmpty() || !Generated.absent().ok()) {
            throw new AssertionError("containers of unimplemented interfaces failed");
        }
        for (Class<?> type : new Class<?>[] { Generated.Empty.class, Generated.Unavailable.class }) {
            Class<?>[] permitted = type.getPermittedSubclasses();
            if (!type.isSealed() || permitted.length != 1 || !permitted[0].isEnum()
                || permitted[0].getEnumConstants().length != 0) {
                throw new AssertionError("interface has an inhabitant");
            }
        }
        try {
            Generated.echo(null);
            throw new AssertionError("null crossed portable interface boundary");
        } catch (NullPointerException expected) {
            // Public interface values retain the non-null portable contract.
        }
    }
}


"#);
    compiled.rejects_consumer(
        "Intruder",
        r#"
package org.polyrust.consumer;
public final class Intruder implements org.polyrust.generated.Generated.Empty {}
"#,
        "sealed",
    );
    compiled.rejects_consumer(
        "Forge",
        r#"
package org.polyrust.consumer;
public final class Forge {
    private Forge() {}
    public static Object forge() { return new org.polyrust.generated.Generated.UninhabitedEmpty(); }
}
"#,
        "private access",
    );
    compiled.rejects_generated_member(
        "static Object forge() { return new UninhabitedEmpty(); }",
        "enum classes may not be instantiated",
    );
}

#[test]
fn unimplemented_interfaces_avoid_inherited_and_implicit_enum_methods() {
    let program = typed_program(portable_name!("enum_method_names"), |builder| {
        builder.interface(
            portable_name!("Unavailable"),
            typed_list![
                interface_method(portable_name!("name"), typed_list![], I32::TYPE),
                interface_method(portable_name!("ordinal"), typed_list![], I32::TYPE),
                interface_method(portable_name!("compareTo"), typed_list![], I32::TYPE),
                interface_method(
                    portable_name!("getDeclaringClass"),
                    typed_list![],
                    I32::TYPE
                ),
                interface_method(
                    portable_name!("describeConstable"),
                    typed_list![],
                    I32::TYPE
                ),
                interface_method(portable_name!("values"), typed_list![], I32::TYPE),
                interface_method(
                    portable_name!("valueOf"),
                    typed_list![parameter(portable_name!("input"), Text::TYPE)],
                    I32::TYPE
                ),
            ],
            |builder, _| builder,
        )
    });
    let manifest = JavaBackend
        .generate_typed(&program)
        .expect("Java resource capacity");
    CompiledPackage::new(&manifest, "enum-method-collisions").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) throws ReflectiveOperationException {
        for (String name : new String[] { "name", "ordinal", "compareTo",
                "getDeclaringClass", "describeConstable", "values" }) {
            Generated.Unavailable.class.getDeclaredMethod(name + "_1");
        }
        Generated.Unavailable.class.getDeclaredMethod("valueOf_1", String.class);
    }
}
"#,
    );
}
