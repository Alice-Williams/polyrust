#![forbid(unsafe_code)]

//! End-to-end Rust-hosted authoring example for a portable validation model.
//!
//! The functions in this crate are ordinary **Rust host code** which construct
//! a checked **PolyRust portable program**. Backends then turn that program into
//! seven independent forms of **generated target code**.

use std::sync::Arc;

use portable_backend_cpp::CppBackend;
use portable_backend_go::GoV0Backend;
use portable_backend_java::JavaBackend;
use portable_backend_python::PythonBackend;
use portable_backend_rust::RustBackend;
use portable_backend_typescript::{JavaScriptBackend, TypeScriptBackend};
use portable_build::{
    Expected, Invocation, ModuleBuilder, Operation, Parameter, Type, TypedValue, Value, Visibility,
};
use portable_check::v0::CheckedProgram;
use portable_codegen::{Backend, BackendOptions, OutputManifest};

/// Builds the target-independent validation module and checks it.
pub fn program() -> CheckedProgram {
    let mut module = ModuleBuilder::new("models_and_validation");
    let adult_age = module.constant(
        "ADULT_AGE",
        Visibility::Public,
        vec!["Default adult registration age.".into()],
        Type::i64(),
        |body| body.constant_literal(Value::i64(18)),
    );
    let (user, (user_name, user_age)) = module.record(
        "User",
        Visibility::Public,
        vec!["A registration candidate.".into()],
        |record| {
            (
                record.field("name", Type::string(), vec![]),
                record.field("age", Type::i64(), vec![]),
            )
        },
    );
    let (age_validator, minimum) = module.record(
        "AgeValidator",
        Visibility::Public,
        vec!["A configurable minimum-age validator.".into()],
        |record| record.field("minimum", Type::i64(), vec![]),
    );
    let (validator, accepts) = module.contract(
        "Validator",
        Visibility::Public,
        vec!["Restricted validation interface.".into()],
        |contract| {
            contract.method(
                "accepts",
                vec![],
                vec![Parameter::new("user", Type::named(user))],
                Some(Type::bool()),
            )
        },
    );
    let (age_validator_impl, (accepts_impl, ())) = module.implementation(
        "AgeValidatorImpl",
        Visibility::Public,
        vec!["Validator implementation backed by AgeValidator.".into()],
        validator,
        age_validator,
        |implementation| {
            implementation.method("accepts", accepts, vec![], |method| {
                method.parameter(Parameter::new("user", Type::named(user)));
                method.returns(Type::bool());
                method.body(|body| {
                    let candidate = body.local("user");
                    let age = body.field(candidate, user_age);
                    let receiver = body.self_value();
                    let threshold = body.field(receiver, minimum);
                    let accepted = body.intrinsic(Operation::GreaterEqual, [age, threshold]);
                    body.block([], Some(accepted))
                });
            })
        },
    );
    let is_adult = module.function(
        "is_adult",
        Visibility::Public,
        vec!["Uses the public default-age constant.".into()],
        |function| {
            function.parameter(Parameter::new("user", Type::named(user)));
            function.returns(Type::bool());
            function.body(|body| {
                let candidate = body.local("user");
                let age = body.field(candidate, user_age);
                let threshold = body.constant(adult_age);
                let accepted = body.intrinsic(Operation::GreaterEqual, [age, threshold]);
                body.block([], Some(accepted))
            });
        },
    );
    let concrete = module.function(
        "can_register_concrete",
        Visibility::Public,
        vec!["Calls a known implementation directly.".into()],
        |function| {
            function.parameter(Parameter::new("validator", Type::named(age_validator)));
            function.parameter(Parameter::new("user", Type::named(user)));
            function.returns(Type::bool());
            function.body(|body| {
                let receiver = body.local("validator");
                let argument = body.local("user");
                let accepted =
                    body.concrete_method(receiver, age_validator_impl, accepts_impl, [argument]);
                body.block([], Some(accepted))
            });
        },
    );
    let abstract_dispatch = module.function(
        "can_register",
        Visibility::Public,
        vec!["Calls through the portable Validator contract.".into()],
        |function| {
            function.parameter(Parameter::new("validator", Type::contract(validator)));
            function.parameter(Parameter::new("user", Type::named(user)));
            function.returns(Type::bool());
            function.body(|body| {
                let receiver = body.local("validator");
                let argument = body.local("user");
                let accepted = body.contract_method(receiver, validator, accepts, [argument]);
                body.block([], Some(accepted))
            });
        },
    );

    let user_value = |name: &str, age: i64| {
        TypedValue::new(
            Type::named(user),
            Value::record(
                user,
                [
                    (user_name, Value::string(name)),
                    (user_age, Value::i64(age)),
                ],
            ),
        )
    };
    let validator_value = |age: i64| {
        TypedValue::new(
            Type::named(age_validator),
            Value::record(age_validator, [(minimum, Value::i64(age))]),
        )
    };
    let expectation =
        |accepted| Expected::value(TypedValue::new(Type::bool(), Value::bool(accepted)));

    for (name, age, accepted) in [
        ("adult_boundary_rejects_17", 17, false),
        ("adult_boundary_accepts_18", 18, true),
        ("adult_accepts_20", 20, true),
        ("adult_rejects_zero", 0, false),
    ] {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(is_adult, [user_value("Candidate", age)]),
            expectation(accepted),
        );
    }
    for (name, age, accepted) in [
        ("concrete_rejects_17", 17, false),
        ("concrete_accepts_18", 18, true),
        ("concrete_accepts_21", 21, true),
    ] {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                concrete,
                [validator_value(18), user_value("Candidate", age)],
            ),
            expectation(accepted),
        );
    }
    for (name, age, accepted) in [
        ("contract_rejects_20", 20, false),
        ("contract_accepts_21", 21, true),
        ("contract_accepts_50", 50, true),
    ] {
        module.portable_test(
            name,
            Visibility::Package,
            vec![],
            Invocation::function(
                abstract_dispatch,
                [validator_value(21), user_value("Candidate", age)],
            ),
            expectation(accepted),
        );
    }

    module.finish().unwrap_or_else(|diagnostics| {
        panic!("models-and-validation did not check: {diagnostics:#?}")
    })
}

/// Generates all required target manifests from the same checked program.
pub fn manifests() -> Vec<(&'static str, OutputManifest)> {
    let program = program();
    let backends: [(&str, Arc<dyn Backend>); 7] = [
        ("rust", Arc::new(RustBackend)),
        ("typescript", Arc::new(TypeScriptBackend)),
        ("javascript", Arc::new(JavaScriptBackend)),
        ("python", Arc::new(PythonBackend)),
        ("go", Arc::new(GoV0Backend)),
        ("java", Arc::new(JavaBackend)),
        ("cpp", Arc::new(CppBackend)),
    ];
    backends
        .into_iter()
        .map(|(directory, backend)| {
            let manifest = backend
                .generate(&program, &BackendOptions::default())
                .unwrap_or_else(|error| panic!("{directory} generation failed: {error:?}"));
            (directory, manifest)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use portable_eval::Evaluator;

    use super::*;

    #[test]
    fn ten_portable_tests_pass_in_the_reference_evaluator() {
        let program = program();
        let results = Evaluator::new(&program).run_all_tests();
        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|result| result.passed), "{results:#?}");
    }

    #[test]
    fn all_seven_manifests_are_nonempty_and_repeatable() {
        let first = manifests();
        let second = manifests();
        assert_eq!(first, second);
        assert_eq!(first.len(), 7);
        assert!(
            first
                .iter()
                .all(|(_, manifest)| !manifest.files().is_empty())
        );
    }
}
