//! Qualified references retain checked identities through Java lexical shadowing.

use super::totality_oracle::CompiledPackage;
use crate::JavaBackend;
use portable_build::{ModuleBuilder, Operation, Parameter, Type, Value, Visibility};
use portable_codegen::{Backend, BackendOptions};

#[test]
fn outer_constants_and_functions_are_not_rebound_to_parameters_or_record_members() {
    let mut module = ModuleBuilder::new("name_identities");
    let constant = module.constant("C", Visibility::Public, vec![], Type::i32(), |body| {
        body.constant_literal(Value::i32(7))
    });
    let function = module.function("foo", Visibility::Public, vec![], |function| {
        function.returns(Type::i32());
        function.body(|body| {
            let value = body.literal(Value::i32(11));
            body.block([], Some(value))
        });
    });
    module.function("captured", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("C", Type::i32()));
        function.returns(Type::i32());
        function.body(|body| {
            let value = body.constant(constant);
            body.block([], Some(value))
        });
    });
    let (record, ()) = module.record("R", Visibility::Public, vec![], |record| {
        record.field("C", Type::i32(), vec![]);
        record.field("foo", Type::i32(), vec![]);
    });
    let (interface, method) =
        module.interface("Readable", Visibility::Public, vec![], |interface| {
            interface.method("read", vec![], vec![], Some(Type::i32()))
        });
    module.implementation(
        "RReadable",
        Visibility::Package,
        vec![],
        interface,
        record,
        |implementation| {
            implementation.method("read", method, vec![], |method| {
                method.returns(Type::i32());
                method.body(|body| {
                    let left = body.constant(constant);
                    let right = body.call(function, []);
                    let sum = body.intrinsic(Operation::IntAddWrapping, [left, right]);
                    body.block([], Some(sum))
                });
            });
        },
    );
    let checked = module.finish().unwrap();
    let manifest = JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    CompiledPackage::new(&manifest, "outer-member-identities").consumer(
        r#"
package org.polyrust.consumer;
import org.polyrust.generated.Generated;
public final class Consumer {
    private Consumer() {}
    public static void main(String[] args) {
        Generated.R value = new Generated.R(100, 200);
        Generated.Readable readable = value;
        if (Generated.captured(99).value() != 7 || value.read().value() != 18
            || readable.read().value() != 18 || value.C() != 100 || value.foo() != 200) {
            throw new AssertionError("Java lexical lookup changed the referenced Core identity");
        }
    }
}
"#,
    );
}
