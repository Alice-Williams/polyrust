//! Every nominal category must use the allocated identity in public factory suffixes.

use super::totality_oracle::CompiledPackage;
use crate::JavaBackend;
use portable_build::{
    I32, option_type, parameter, portable_name, result_type, typed_list, typed_program, variant,
};

macro_rules! nominal_factory_test {
    ($test:ident, $constructor:ident, $members:expr) => {
        #[test]
        fn $test() {
            let program = typed_program(portable_name!("nominal_factories"), |builder| {
                builder.$constructor(portable_name!("exports"), $members, |builder, first| {
                    builder.$constructor(portable_name!("exports_"), $members, |builder, second| {
                        let builder = builder
                            .function(
                                portable_name!("first"),
                                typed_list![],
                                option_type(first.ty()),
                                |body, _| body.none(first.ty()),
                            )
                            .builder;
                        let builder = builder
                            .function(
                                portable_name!("second"),
                                typed_list![],
                                option_type(second.ty()),
                                |body, _| body.none(second.ty()),
                            )
                            .builder;
                        let builder = builder
                            .function(
                                portable_name!("first_result"),
                                typed_list![parameter(
                                    portable_name!("value"),
                                    result_type(first.ty(), I32::TYPE),
                                )],
                                result_type(first.ty(), I32::TYPE),
                                |body, parameters| body.read(parameters.head),
                            )
                            .builder;
                        builder
                            .function(
                                portable_name!("second_result"),
                                typed_list![parameter(
                                    portable_name!("value"),
                                    result_type(second.ty(), I32::TYPE),
                                )],
                                result_type(second.ty(), I32::TYPE),
                                |body, parameters| body.read(parameters.head),
                            )
                            .builder
                    })
                })
            });
            let manifest = JavaBackend.generate_typed(&program);
            CompiledPackage::new(&manifest, stringify!($test)).consumer(
                r#"
package org.polyrust.consumer;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        if (!org.polyrust.generated.Generated.first().ok()
            || !org.polyrust.generated.Generated.second().ok()) {
            throw new AssertionError("nominal option factories merged");
        }
    }
}
"#,
            );
        }
    };
}

nominal_factory_test!(record_factory_names, record, typed_list![]);
nominal_factory_test!(
    enum_factory_names,
    enumeration,
    typed_list![variant(portable_name!("ONE"))]
);
nominal_factory_test!(interface_factory_names, interface, typed_list![]);

#[test]
fn synthetic_interface_name_is_allocated_before_ast_verification() {
    let program = typed_program(portable_name!("synthetic_names"), |builder| {
        builder.record(
            portable_name!("UninhabitedFoo"),
            typed_list![],
            |builder, _| {
                builder.interface(portable_name!("Foo"), typed_list![], |builder, _| builder)
            },
        )
    });
    let manifest = JavaBackend.generate_typed(&program);
    CompiledPackage::new(&manifest, "synthetic-nominal-collision").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        new Generated.UninhabitedFoo();
        Class<?>[] permitted = Generated.Foo.class.getPermittedSubclasses();
        if (permitted.length != 1 || !permitted[0].getSimpleName().equals("UninhabitedFoo_1")
            || permitted[0].getEnumConstants().length != 0) {
            throw new AssertionError("synthetic identity or permits changed");
        }
    }
}


"#,
    );
}

#[test]
fn synthetic_suffixes_do_not_steal_other_synthetic_preferred_names() {
    let program = typed_program(portable_name!("synthetic_suffixes"), |builder| {
        builder.record(
            portable_name!("UninhabitedA"),
            typed_list![],
            |builder, _| {
                builder.interface(portable_name!("A"), typed_list![], |builder, _| {
                    builder.interface(portable_name!("A_1"), typed_list![], |builder, _| builder)
                })
            },
        )
    });
    let manifest = JavaBackend.generate_typed(&program);
    CompiledPackage::new(&manifest, "synthetic-suffix-reservation").consumer(r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        if (!Generated.A.class.getPermittedSubclasses()[0].getSimpleName().equals("UninhabitedA_2")
            || !Generated.A_1.class.getPermittedSubclasses()[0].getSimpleName().equals("UninhabitedA_1")) {
            throw new AssertionError("synthetic preferred names stolen by suffixes");
        }
    }
}
"#);
}
