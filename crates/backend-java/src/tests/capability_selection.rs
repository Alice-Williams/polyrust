//! Dynamic Java capability selection and rejection regressions.

use portable_build::{ModuleBuilder, Operation, Parameter, Type, Value, Visibility};
use portable_codegen::{
    Backend, BackendOptions, OutputContents, TypedCompiler, TypedGenerationError,
    TypedPipelineStage,
};
use portable_core_ir::lower_checked;

use super::*;

fn preflight_diagnostics(checked: &portable_check::v0::CheckedProgram) -> Vec<Diagnostic> {
    match crate::JavaBackend::compiler()
        .compile_checked(checked, &BackendOptions::default())
        .expect_err("Java-illegal shape must fail")
    {
        TypedGenerationError::Phase {
            stage: TypedPipelineStage::CapabilityPreflight,
            diagnostics,
        } => diagnostics,
        other => panic!("unexpected generation error: {other:?}"),
    }
}

#[test]
fn target_id_matches_java_backend() {
    assert_eq!(
        JavaCapabilityRegistry::default().target().as_str(),
        "org.polyrust.java"
    );
}

#[test]
fn empty_checked_program_selects_every_collected_feature() {
    let mut module = ModuleBuilder::new("capability_java");
    module.function("identity", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("value", Type::bool()));
        function.returns(Type::bool());
        function.body(|body| {
            let value = body.local("value");
            body.block([], Some(value))
        });
    });
    let checked = module.finish().unwrap();
    let core = lower_checked(&checked).unwrap();
    let selected = preflight_capabilities(&core, &JavaCapabilityRegistry::default()).unwrap();
    assert_eq!(
        selected.len(),
        portable_codegen::collect_core_features(&core).len()
    );
}

#[test]
fn exact_selection_rejects_feature_and_strategy_permutations() {
    let mut module = ModuleBuilder::new("capability_selection");
    module.function("identity", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("value", Type::bool()));
        function.returns(Type::bool());
        function.body(|body| {
            let value = body.local("value");
            body.block([], Some(value))
        });
    });
    let checked = module.finish().unwrap();
    let core = lower_checked(&checked).unwrap();
    let selected = preflight_capabilities(&core, &JavaCapabilityRegistry::default()).unwrap();

    let mut permuted_usage = JavaCapabilitySelection {
        selected: selected.clone(),
    };
    permuted_usage.selected.swap(0, 1);
    assert!(permuted_usage.validate_for(&core).is_err());

    let mut mismatched_strategy = JavaCapabilitySelection { selected };
    let declaration = mismatched_strategy
        .selected
        .iter()
        .position(|value| value.strategy == JavaLoweringStrategy::Declaration)
        .expect("declaration strategy");
    let direct = mismatched_strategy
        .selected
        .iter()
        .position(|value| value.strategy == JavaLoweringStrategy::DirectValue)
        .expect("direct strategy");
    let declaration_strategy = mismatched_strategy.selected[declaration].strategy;
    mismatched_strategy.selected[declaration].strategy =
        mismatched_strategy.selected[direct].strategy;
    mismatched_strategy.selected[direct].strategy = declaration_strategy;
    assert!(mismatched_strategy.validate_for(&core).is_err());
}

#[test]
fn record_restricted_component_allocates_distinct_java_name() {
    let mut module = ModuleBuilder::new("java_record_restricted");
    module.record("Bad", Visibility::Public, vec![], |record| {
        record.field("hashCode", Type::i32(), vec![]);
    });
    let checked = module.finish().unwrap();
    let manifest = crate::JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("portable name has a Java allocation");
    let generated = crate::tests::generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    assert!(generated.contains("record Bad(int hashCode_1)"));
}

#[test]
fn inherited_object_static_method_allocates_distinct_java_name() {
    let mut module = ModuleBuilder::new("java_object_static");
    module.function("hashCode", Visibility::Public, vec![], |function| {
        function.returns(Type::i32());
        function.body(|body| {
            let value = body.literal(Value::i32(7));
            body.block([], Some(value))
        });
    });
    let checked = module.finish().unwrap();
    let manifest = crate::JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("portable name has a Java allocation");
    let generated = crate::tests::generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    assert!(generated.contains("hashCode_1()"));
}

#[test]
fn nonfinal_object_interface_method_allocates_distinct_java_name() {
    let mut module = ModuleBuilder::new("java_object_interface");
    let (interface, method) = module.interface("Bad", Visibility::Public, vec![], |interface| {
        interface.method("toString", vec![], vec![], Some(Type::string()))
    });
    let (record, ()) = module.record("Value", Visibility::Public, vec![], |_| {});
    module.implementation(
        "ValueBad",
        Visibility::Package,
        vec![],
        interface,
        record,
        |implementation| {
            implementation.method("toString", method, vec![], |method| {
                method.returns(Type::string());
                method.body(|body| {
                    let value = body.literal(Value::string("value"));
                    body.block([], Some(value))
                });
            });
        },
    );
    let checked = module.finish().unwrap();
    let manifest = crate::JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("portable name has a Java allocation");
    let generated = crate::tests::generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    assert!(generated.contains("toString_1()"));
}

#[test]
fn all_inherited_object_names_allocate_safely() {
    for name in [
        "getClass",
        "hashCode",
        "clone",
        "toString",
        "notify",
        "notifyAll",
        "wait",
        "finalize",
    ] {
        let mut module = ModuleBuilder::new("object_names");
        module.function(name, Visibility::Public, vec![], |function| {
            function.returns(Type::i32());
            function.body(|body| {
                let value = body.literal(Value::i32(1));
                body.block([], Some(value))
            });
        });
        let checked = module.finish().unwrap();
        let manifest = crate::JavaBackend
            .generate(&checked, &BackendOptions::default())
            .expect("Object method name is mapped");
        let generated = crate::tests::generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Generated.java",
        );
        assert!(generated.contains(&format!("{name}_1()")));
    }
}

#[test]
fn final_object_interface_method_allocates_distinct_java_name() {
    let mut module = ModuleBuilder::new("java_object_final");
    let (interface, method) = module.interface("Bad", Visibility::Public, vec![], |interface| {
        interface.method("getClass", vec![], vec![], Some(Type::bool()))
    });
    let (record, ()) = module.record("Value", Visibility::Public, vec![], |_| {});
    module.implementation(
        "ValueBad",
        Visibility::Package,
        vec![],
        interface,
        record,
        |implementation| {
            implementation.method("getClass", method, vec![], |method| {
                method.returns(Type::bool());
                method.body(|body| {
                    let value = body.literal(Value::bool(true));
                    body.block([], Some(value))
                });
            });
        },
    );
    let checked = module.finish().unwrap();
    let manifest = crate::JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("portable name has a Java allocation");
    let generated = crate::tests::generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    assert!(generated.contains("getClass_1()"));
}

#[test]
fn clone_interface_method_allocates_distinct_java_name() {
    let mut module = ModuleBuilder::new("java_object_clone");
    let (interface, method) = module.interface("Bad", Visibility::Public, vec![], |interface| {
        interface.method("clone", vec![], vec![], Some(Type::i32()))
    });
    let (record, ()) = module.record("Value", Visibility::Public, vec![], |_| {});
    module.implementation(
        "ValueBad",
        Visibility::Package,
        vec![],
        interface,
        record,
        |implementation| {
            implementation.method("clone", method, vec![], |method| {
                method.returns(Type::i32());
                method.body(|body| {
                    let value = body.literal(Value::i32(1));
                    body.block([], Some(value))
                });
            });
        },
    );
    let checked = module.finish().unwrap();
    let manifest = crate::JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("portable name has a Java allocation");
    let generated = crate::tests::generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    assert!(generated.contains("clone_1()"));
}

#[test]
fn record_accessor_implementation_method_allocates_distinct_java_name() {
    let mut module = ModuleBuilder::new("java_accessor_collision");
    let (interface, method) =
        module.interface("Readable", Visibility::Public, vec![], |interface| {
            interface.method("read", vec![], vec![], Some(Type::bool()))
        });
    let (record, ()) = module.record("Value", Visibility::Public, vec![], |record| {
        record.field("read", Type::bool(), vec![]);
    });
    module.implementation(
        "ValueReadable",
        Visibility::Package,
        vec![],
        interface,
        record,
        |implementation| {
            implementation.method("read", method, vec![], |method| {
                method.returns(Type::bool());
                method.body(|body| {
                    let value = body.literal(Value::bool(true));
                    body.block([], Some(value))
                });
            });
        },
    );
    let checked = module.finish().unwrap();
    let manifest = crate::JavaBackend
        .generate(&checked, &BackendOptions::default())
        .expect("portable name has a Java allocation");
    let generated = crate::tests::generated_text(
        &manifest,
        "src/main/java/org/polyrust/generated/Generated.java",
    );
    assert!(generated.contains("read_1()"));
}

#[test]
fn nested_fallible_constant_intrinsic_stops_at_preflight() {
    let mut module = ModuleBuilder::new("java_fallible_constant");
    module.constant(
        "INVALID",
        Visibility::Public,
        vec![],
        Type::bool(),
        |body| {
            let one = body.constant_literal(Value::i32(1));
            let two = body.constant_literal(Value::i32(2));
            let checked =
                body.constant_intrinsic(portable_build::Operation::IntAddChecked, [one, two]);
            let three = body.constant_literal(Value::i32(3));
            body.constant_intrinsic(portable_build::Operation::Equal, [checked, three])
        },
    );
    let checked = module.finish().unwrap();
    let diagnostics = preflight_diagnostics(&checked);
    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .message
            .contains("fallible intrinsic in a static constant initializer")
    );
    assert_eq!(diagnostics[0].target.as_deref(), Some("org.polyrust.java"));
}

#[test]
fn minimal_program_does_not_pull_optional_runtime_helpers() {
    let mut module = ModuleBuilder::new("java_minimal_helpers");
    module.constant("FLAG", Visibility::Public, vec![], Type::bool(), |body| {
        body.constant_literal(Value::bool(true))
    });
    let checked = module.finish().unwrap();
    let manifest = crate::JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let runtime = match manifest
        .file("src/main/java/org/polyrust/generated/Runtime.java")
        .expect("runtime file")
        .contents()
    {
        OutputContents::Text(value) => value,
        OutputContents::Bytes(_) => panic!("Java runtime must be text"),
    };
    let conformance = match manifest
        .file("src/test/java/org/polyrust/generated/ConformanceTest.java")
        .expect("conformance file")
        .contents()
    {
        OutputContents::Text(value) => value,
        OutputContents::Bytes(_) => panic!("Java conformance file must be text"),
    };
    assert!(!runtime.contains("import "));
    for absent in [
        "PolyOption",
        "PolyResult",
        "checkedAdd",
        "requireScalarString",
        "appendList",
        "class Bytes",
    ] {
        assert!(!runtime.contains(absent), "unexpected helper {absent}");
    }
    assert!(!conformance.contains("Runtime."));
}

#[test]
fn string_replace_all_uses_only_its_typed_string_runtime_helper() {
    let mut module = ModuleBuilder::new("java_replace_all_helper");
    module.function("replace_all", Visibility::Public, vec![], |function| {
        function.parameter(Parameter::new("source", Type::string()));
        function.parameter(Parameter::new("needle", Type::string()));
        function.parameter(Parameter::new("replacement", Type::string()));
        function.returns(Type::string());
        function.body(|body| {
            let source = body.local("source");
            let needle = body.local("needle");
            let replacement = body.local("replacement");
            let value = body.intrinsic(Operation::StringReplaceAll, [source, needle, replacement]);
            body.block([], Some(value))
        });
    });
    let checked = module.finish().unwrap();
    let manifest = crate::JavaBackend
        .generate(&checked, &BackendOptions::default())
        .unwrap();
    let generated = match manifest
        .file("src/main/java/org/polyrust/generated/Generated.java")
        .expect("generated file")
        .contents()
    {
        OutputContents::Text(value) => value,
        OutputContents::Bytes(_) => panic!("Java source must be text"),
    };
    let runtime = match manifest
        .file("src/main/java/org/polyrust/generated/Runtime.java")
        .expect("runtime file")
        .contents()
    {
        OutputContents::Text(value) => value,
        OutputContents::Bytes(_) => panic!("Java runtime must be text"),
    };

    assert!(generated.contains("Runtime.stringReplaceAll(source, needle, replacement)"));
    assert!(runtime.contains("static String stringReplaceAll"));
    assert!(runtime.contains("Character.charCount(source.codePointAt(offset))"));
    assert!(!runtime.contains(" stringSliceScalars("));
    assert!(!runtime.contains(" bytesReplaceAll("));
    assert!(!runtime.contains(" checkedAddI32("));
}
