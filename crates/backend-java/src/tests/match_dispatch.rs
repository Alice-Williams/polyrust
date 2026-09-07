//! Both enum representations and wildcard fallback keep executable dispatch honest.
use super::totality_oracle::CompiledPackage;
use portable_build::{ModuleBuilder, Parameter, Type, Value, Visibility};
use portable_codegen::{Backend, BackendOptions};

pub(crate) fn fixture(payload: bool, wildcard: bool) -> portable_check::v0::CheckedProgram {
    let mut module = ModuleBuilder::new("match_dispatch");
    let (choice, (first, second, field)) =
        module.enumeration("Choice", Visibility::Public, vec![], |enumeration| {
            let (first, field) = enumeration.variant("A", vec![], |variant| {
                payload.then(|| variant.field("value", Type::i32(), vec![]))
            });
            let (second, ()) = enumeration.variant("B", vec![], |_| {});
            (first, second, field)
        });
    module.function("matchValue", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("choice", Type::named(choice)));
        function.returns(Type::i32());
        function.body(|body| {
            let bindings = field.map(|field| (field, "payload".to_owned())).into_iter();
            let first_pattern = body.enum_pattern(choice, first, bindings);
            let first_value = if payload {
                body.local("payload")
            } else {
                body.literal(Value::i32(7))
            };
            let first_body = body.block([], Some(first_value));
            let first_arm = body.match_arm(first_pattern, first_body);
            let last_pattern = if wildcard {
                body.wildcard_pattern()
            } else {
                body.enum_pattern(choice, second, [])
            };
            let last_value = body.literal(Value::i32(11));
            let last_body = body.block([], Some(last_value));
            let last_arm = body.match_arm(last_pattern, last_body);
            let value = body.local("choice");
            let matched = body.match_value(value, [first_arm, last_arm]);
            body.block([], Some(matched))
        });
    });
    module.finish().expect("checked enum match fixture")
}

#[test]
fn enum_and_payload_matches_with_and_without_wildcards_compile_and_execute() {
    for (payload, wildcard) in [(false, false), (true, false), (false, true), (true, true)] {
        let checked = fixture(payload, wildcard);
        let manifest = crate::JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let first = if payload {
            "new Generated.ChoiceA(7)"
        } else {
            "Generated.Choice.A"
        };
        let second = if payload {
            "new Generated.ChoiceB()"
        } else {
            "Generated.Choice.B"
        };
        CompiledPackage::new(&manifest, &format!("match-{payload}-{wildcard}")).consumer(&format!(
            "package org.polyrust.consumer;
             import org.polyrust.generated.Generated;
             public final class Consumer {{
                 private Consumer() {{}}
                 public static void main(String[] args) {{
                     if (Generated.matchValue({first}).value() != 7
                         || Generated.matchValue({second}).value() != 11) {{
                         throw new AssertionError(\"enum match dispatch changed\");
                     }}
                 }}
             }}"
        ));
    }
}
