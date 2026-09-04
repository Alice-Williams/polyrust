//! Java 21 generation through verified CoreIR, a typed Java AST, the shared
//! symbol linker, opaque syntax certification, and total structural rendering.

#![forbid(unsafe_code)]

pub mod ast;
mod capability;
pub mod dialect;
pub mod feature;
mod lower;
mod render;
mod runtime;

use std::collections::BTreeMap;

use portable_build::{Feature, Requirements, Supports, SupportsAll, TypedProgram};
use portable_check::v0::{Capability, CheckedProgram};
use portable_codegen::{
    Backend, BackendDescriptor, BackendError, BackendOptions, BackendVersion, CanonicalCoreAdapter,
    CapabilitySupport, CertifiedStructuralRendererAdapter, IrVersionRange, OptionsSchema,
    OutputManifest, TargetId, TargetLinker, TypedCompiler, TypedCompilerAdapter,
    TypedLanguagePlugin,
};
use portable_core_ir::CoreProgram;
use portable_ir::v0::IrVersion;

use crate::{
    capability::JavaCapabilityRegistry,
    dialect::{JavaDialect, JavaHelperCapability, JavaRuntimeHelper},
    feature::{JavaFeatureSet, java_features},
    lower::JavaLowerer,
    render::JavaRenderer,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct JavaBackend;

impl JavaBackend {
    fn descriptor_value() -> BackendDescriptor {
        BackendDescriptor {
            target: TargetId::parse("org.polyrust.java").expect("static target ID is valid"),
            display_name: "Java".to_owned(),
            backend_version: BackendVersion::new(0, 2, 0),
            supported_ir: IrVersionRange::exact(IrVersion::CURRENT),
        }
    }

    fn compiler() -> TypedCompilerAdapter<CanonicalCoreAdapter, JavaPlugin> {
        TypedCompilerAdapter::new(CanonicalCoreAdapter, JavaPlugin::new())
    }

    /// Generates Java from a valid-by-construction typed portable program.
    ///
    /// The `SupportsAll<R>` bound proves at the call site that Java implements
    /// every feature inferred from this particular program. A failure below
    /// this boundary is a PolyRust implementation defect, not a user diagnostic.
    ///
    /// An ordinary dynamically checked program cannot call this API:
    ///
    /// ```compile_fail
    /// use portable_backend_java::JavaBackend;
    /// use portable_check::v0::CheckedProgram;
    /// fn rejected(program: &CheckedProgram) {
    ///     let _ = JavaBackend.generate_typed(program);
    /// }
    /// ```
    ///
    /// A generic requirement tree without a Java support proof also fails:
    ///
    /// ```compile_fail
    /// use portable_backend_java::JavaBackend;
    /// use portable_build::{Requirements, TypedProgram};
    /// fn rejected<R: Requirements>(program: &TypedProgram<R>) {
    ///     let _ = JavaBackend.generate_typed(program);
    /// }
    /// ```
    pub fn generate_typed<R>(&self, program: &TypedProgram<R>) -> OutputManifest
    where
        R: Requirements,
        JavaPlugin: SupportsAll<R>,
    {
        self.generate(program.checked_program(), &BackendOptions::default())
            .unwrap_or_else(|error| panic!("TypedProgram Java invariant failure: {error:#?}"))
    }
}

impl Backend for JavaBackend {
    fn descriptor(&self) -> BackendDescriptor {
        Self::descriptor_value()
    }

    fn support(&self, capability: Capability) -> CapabilitySupport {
        match capability {
            Capability::CheckedIntegerArithmetic => helper_support(
                JavaRuntimeHelper::CheckedIntegers,
                JavaHelperCapability::CheckedArithmetic,
            ),
            Capability::UnicodeScalar => helper_support(
                JavaRuntimeHelper::Unicode,
                JavaHelperCapability::UnicodeScalars,
            ),
            Capability::ImmutableList => helper_support(
                JavaRuntimeHelper::ImmutableLists,
                JavaHelperCapability::ImmutableLists,
            ),
            Capability::Bytes => helper_support(
                JavaRuntimeHelper::Bytes,
                JavaHelperCapability::ImmutableBytes,
            ),
            Capability::InterfaceDispatch | Capability::FirstClassInterfaceValues => {
                helper_support(
                    JavaRuntimeHelper::Interfaces,
                    JavaHelperCapability::InterfaceDispatch,
                )
            }
            Capability::F64 => helper_support(
                JavaRuntimeHelper::FloatBits,
                JavaHelperCapability::ExactFloatBits,
            ),
            Capability::Option | Capability::Result => helper_support(
                JavaRuntimeHelper::TaggedValues,
                JavaHelperCapability::TaggedValues,
            ),
            Capability::WrappingIntegerArithmetic | Capability::BoundedIteration => {
                CapabilitySupport::Native
            }
        }
    }

    fn options_schema(&self) -> OptionsSchema {
        BTreeMap::new()
    }

    fn generate(
        &self,
        program: &CheckedProgram,
        options: &BackendOptions,
    ) -> Result<OutputManifest, BackendError> {
        Self::compiler()
            .compile_checked(program, options)
            .map_err(|error| BackendError::Generation {
                message: format!("typed Java generation failed: {error:#?}"),
            })
    }
}

fn helper_support(
    helper: JavaRuntimeHelper,
    capability: JavaHelperCapability,
) -> CapabilitySupport {
    CapabilitySupport::Helper {
        helper: format!("{}:{}", helper.name(), capability.name()),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct JavaPlugin {
    features: JavaFeatureSet,
}

impl JavaPlugin {
    fn new() -> Self {
        Self {
            features: java_features(),
        }
    }
}

impl Default for JavaPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl<F> Supports<F> for JavaPlugin
where
    F: Feature,
    JavaFeatureSet: Supports<F, Dialect = JavaDialect>,
{
    type Dialect = JavaDialect;
    type Mapping = <JavaFeatureSet as Supports<F>>::Mapping;

    fn mapping(&self) -> &Self::Mapping {
        self.features.mapping()
    }
}

impl TypedLanguagePlugin<CoreProgram> for JavaPlugin {
    type Dialect = JavaDialect;
    type CapabilityRegistry = JavaCapabilityRegistry;
    type Lowerer = JavaLowerer;
    type Resolver = TargetLinker<JavaDialect>;
    type Renderer = CertifiedStructuralRendererAdapter<JavaDialect, JavaRenderer>;

    fn descriptor(&self) -> BackendDescriptor {
        JavaBackend::descriptor_value()
    }
    fn options_schema(&self) -> OptionsSchema {
        BTreeMap::new()
    }
    fn dialect(&self) -> Self::Dialect {
        JavaDialect
    }
    fn capability_registry(&self) -> Self::CapabilityRegistry {
        JavaCapabilityRegistry::new(self.features)
    }
    fn lowerer(&self) -> Self::Lowerer {
        JavaLowerer::new(self.features)
    }
    fn resolver(&self) -> Self::Resolver {
        TargetLinker::new(JavaDialect)
    }
    fn renderer(&self) -> Self::Renderer {
        CertifiedStructuralRendererAdapter::new(JavaRenderer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use portable_build::{
        I32, ModuleBuilder, Operation, Parameter, Type, Value, Visibility, field, parameter,
        portable_name, typed_list, typed_program,
    };
    use portable_codegen::{OutputContents, TypedGenerationError, TypedPipelineStage};
    use std::collections::BTreeSet;

    fn fixture() -> CheckedProgram {
        portable_check::v0::check_program(
            portable_ir::v0::from_json(include_bytes!(
                "../../build/testdata/registration.poly.json"
            ))
            .expect("fixture parses"),
        )
        .expect("fixture checks")
    }

    fn typed_fixture_manifests() -> [OutputManifest; 3] {
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
            added.builder.record(
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
            )
        });
        [
            JavaBackend.generate_typed(&program),
            JavaBackend.generate_typed(&program),
            JavaBackend.generate_typed(&program),
        ]
    }

    fn generated_text<'a>(manifest: &'a OutputManifest, path: &str) -> &'a str {
        match manifest.file(path).expect("generated file").contents() {
            OutputContents::Text(value) => value,
            OutputContents::Bytes(_) => panic!("Java source must be text"),
        }
    }

    #[test]
    fn inferred_generation_is_total_deterministic_and_structural() {
        let [first, second, third] = typed_fixture_manifests();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(second.canonical_json(), third.canonical_json());

        let generated = generated_text(
            &first,
            "src/main/java/org/polyrust/generated/Generated.java",
        );
        assert!(generated.contains("public static record Point3(int x, int y, int z)"));
        assert!(generated.contains("compute(final int left, final int right, final int scale)"));
        assert!(generated.contains("final int __polyrust_intrinsicOperand_0 = (left + right);"));
        assert!(generated.contains("final int __polyrust_intrinsicOperand_1 = (left - right);"));
        assert!(generated.contains(
            "final int __polyrust_intrinsicOperand_2 = (__polyrust_intrinsicOperand_0 * __polyrust_intrinsicOperand_1);"
        ));
        assert!(generated.contains("Runtime.ok((__polyrust_intrinsicOperand_2 + scale))"));
        assert!(generated.contains("Runtime.ok(new Point3(x, y, z))"));
    }

    #[test]
    fn generated_manifest_is_typed_deterministic_and_dependency_free() {
        let checked = fixture();
        let first = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let third = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(second.canonical_json(), third.canonical_json());
        assert!(first.dependencies().is_empty());
        assert_eq!(first.files().len(), 6);
    }

    #[test]
    fn generated_source_contains_direct_methods_and_no_legacy_interpreter() {
        let manifest = JavaBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let generated = generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Generated.java",
        );
        let runtime = generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Runtime.java",
        );
        assert!(generated.contains("record Label"));
        assert!(generated.contains("interface Renderable"));
        for forbidden in [
            "serde_json",
            "jsonArray",
            "invokeTest",
            "invokeMethod",
            "readConstant",
            "POLYRUST-BEGIN",
            "POLYRUST-END",
            "Runtime.capture",
            "Runtime.unwrap",
            "class Halt",
            "extends RuntimeException",
        ] {
            assert!(!generated.contains(forbidden), "found {forbidden}");
            assert!(!runtime.contains(forbidden), "found {forbidden}");
        }
    }

    #[test]
    fn conformance_program_executes_the_exact_portable_test_inventory() {
        let checked = fixture();
        assert_eq!(
            checked
                .module()
                .declarations
                .iter()
                .filter(|declaration| matches!(declaration, portable_ir::v0::Declaration::Test(_)))
                .count(),
            1
        );
        let manifest = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let conformance = generated_text(
            &manifest,
            "src/test/java/org/polyrust/generated/ConformanceTest.java",
        );
        assert!(conformance.contains("call_render_returns_text"));
        assert!(conformance.contains("int completed = 0;"));
        assert!(conformance.contains("completed = (completed + 1);"));
        assert!(conformance.contains("completed == 1"));
        assert!(conformance.contains("Runtime.deepEqual"));
        assert!(
            !conformance.contains("public static void main(final String[] arguments) {\n    }")
        );
    }

    #[test]
    fn imports_are_derived_from_typed_references() {
        let manifest = JavaBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let generated = generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Generated.java",
        );
        assert!(!generated.contains("import java.math.BigInteger;"));
        assert!(!generated.contains("import java.nio.ByteBuffer;"));
        assert_eq!(generated.matches("import java.util.List;").count(), 0);
        assert_eq!(generated.matches("import java.util.Objects;").count(), 1);
        assert!(generated.contains("Runtime.requireScalarString(text)"));
    }

    #[test]
    fn runtime_imports_are_exact_and_physically_deduplicated() {
        let manifest = JavaBackend
            .generate(&fixture(), &BackendOptions::default())
            .unwrap();
        let runtime = generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Runtime.java",
        );
        let imports = runtime
            .lines()
            .filter_map(|line| line.strip_prefix("import ")?.strip_suffix(';'))
            .collect::<Vec<_>>();
        let unique = imports.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(imports.len(), unique.len(), "duplicate import: {imports:?}");
        assert_eq!(
            unique,
            BTreeSet::from(["java.util.List", "java.util.Objects"])
        );
    }

    #[test]
    fn canonical_interface_and_composition_corpus_is_flat_and_deterministic() {
        let checked = portable_check::v0::check_program(
            portable_build::interface_composition_fixture().document,
        )
        .expect("canonical interface corpus checks");
        let first = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let second = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        let third = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .unwrap();
        assert_eq!(first.canonical_json(), second.canonical_json());
        assert_eq!(second.canonical_json(), third.canonical_json());

        let generated = generated_text(
            &first,
            "src/main/java/org/polyrust/generated/Generated.java",
        );
        assert!(generated.contains("interface Labelled"));
        assert!(generated.contains("interface Measured"));
        assert!(generated.contains(
            "record Label(String text) implements org.polyrust.generated.Runtime.SemanticValue, Labelled, Measured"
        ));
        assert!(generated.contains("record Service(Labelled renderer)"));
        assert!(generated.contains("Objects.requireNonNull(renderer)"));
        assert!(!generated.contains("interface Labelled extends"));
        assert!(!generated.contains("interface Measured extends"));
    }

    #[test]
    fn overlapping_erased_interface_methods_stop_at_capability_preflight() {
        let mut module = ModuleBuilder::new("java_overlap");
        let (first, first_method) =
            module.interface("First", Visibility::Public, vec![], |interface| {
                interface.method("render", vec![], vec![], Some(Type::string()))
            });
        let (second, second_method) =
            module.interface("Second", Visibility::Public, vec![], |interface| {
                interface.method("render", vec![], vec![], Some(Type::string()))
            });
        let (record, ()) = module.record("Value", Visibility::Public, vec![], |_| {});
        module.implementation(
            "ValueFirst",
            Visibility::Package,
            vec![],
            first,
            record,
            |implementation| {
                implementation.method("render", first_method, vec![], |method| {
                    method.returns(Type::string());
                    method.body(|body| {
                        let value = body.literal(Value::string("first"));
                        body.block([], Some(value))
                    });
                });
            },
        );
        module.implementation(
            "ValueSecond",
            Visibility::Package,
            vec![],
            second,
            record,
            |implementation| {
                implementation.method("render", second_method, vec![], |method| {
                    method.returns(Type::string());
                    method.body(|body| {
                        let value = body.literal(Value::string("second"));
                        body.block([], Some(value))
                    });
                });
            },
        );
        let checked = module.finish().expect("portable overlap is valid");
        let error = JavaBackend::compiler()
            .compile_checked(&checked, &BackendOptions::default())
            .unwrap_err();
        match error {
            TypedGenerationError::Phase {
                stage: TypedPipelineStage::CapabilityPreflight,
                diagnostics,
            } => {
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].target.as_deref(), Some("org.polyrust.java"));
                assert!(
                    diagnostics[0]
                        .message
                        .contains("Java-erased method render() collides")
                );
            }
            other => panic!("unexpected generation error: {other:?}"),
        }
    }

    #[test]
    fn generated_expressions_are_evaluated_once_in_source_order() {
        let mut module = ModuleBuilder::new("java_evaluation_order");
        let (boxed, field) = module.record("Boxed", Visibility::Public, vec![], |record| {
            record.field("value", Type::i32(), vec![])
        });
        let later = module.function("later", Visibility::Package, vec![], |function| {
            function.returns(Type::i32());
            function.body(|body| {
                let value = body.literal(Value::i32(2));
                body.block([], Some(value))
            });
        });
        let combine = module.function("combine", Visibility::Package, vec![], |function| {
            function.parameter(Parameter::new("left", Type::named(boxed)));
            function.parameter(Parameter::new("right", Type::i32()));
            function.returns(Type::i32());
            function.body(|body| {
                let right = body.local("right");
                body.block([], Some(right))
            });
        });
        module.function("entry", Visibility::Public, vec![], |function| {
            function.returns(Type::i32());
            function.body(|body| {
                let one = body.literal(Value::i32(1));
                let allocated = body.record(boxed, [(field, one)]);
                let later_value = body.call(later, []);
                let result = body.call(combine, [allocated, later_value]);
                body.block([], Some(result))
            });
        });
        module.function("unwrap_once", Visibility::Public, vec![], |function| {
            function.returns(Type::string());
            function.body(|body| {
                let a = body.literal(Value::string("a"));
                let b = body.literal(Value::string("b"));
                let concatenated = body.intrinsic(Operation::StringConcat, [a, b]);
                let present = body.some(concatenated);
                let fallback = body.literal(Value::string("fallback"));
                let result = body.intrinsic(Operation::OptionUnwrapOr, [present, fallback]);
                body.block([], Some(result))
            });
        });

        let checked = module.finish().expect("evaluation-order fixture checks");
        let manifest = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .expect("evaluation-order fixture generates");
        let generated = generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Generated.java",
        );

        let entry = generated
            .split(" entry()")
            .nth(1)
            .expect("entry method is generated")
            .split("\n    }")
            .next()
            .expect("entry method body");
        let allocation = entry.find("new Boxed(").expect("record allocation");
        let later_call = entry.find("later()").expect("later operand call");
        let combine_call = entry.find("combine(").expect("outer call");
        assert!(
            allocation < later_call,
            "allocation must precede later operand"
        );
        assert!(
            later_call < combine_call,
            "operands must precede outer call"
        );
        assert_eq!(entry.matches("new Boxed(").count(), 1);

        let unwrap = generated
            .split(" unwrap_once()")
            .nth(1)
            .expect("unwrap_once method is generated")
            .split("\n    }")
            .next()
            .expect("unwrap_once method body");
        assert_eq!(
            unwrap.matches("\"a\" + \"b\"").count(),
            1,
            "the nontrivial left operand must be evaluated exactly once"
        );
    }

    #[test]
    fn portable_evaluate_lowers_to_a_valid_java_local() {
        let mut module = ModuleBuilder::new("java_evaluate");
        module.function("visit", Visibility::Public, vec![], |function| {
            function.parameter(Parameter::new("value", Type::i64()));
            function.returns(Type::unit());
            function.body(|body| {
                let value = body.local("value");
                let evaluate = body.expression_statement(value);
                let unit = body.literal(Value::unit());
                body.block([evaluate], Some(unit))
            });
        });

        let checked = module.finish().expect("Evaluate fixture checks");
        let manifest = JavaBackend
            .generate(&checked, &BackendOptions::default())
            .expect("valid Evaluate fixture generates");
        let generated = generated_text(
            &manifest,
            "src/main/java/org/polyrust/generated/Generated.java",
        );
        assert_eq!(
            generated.matches("final long __polyrust_evaluate_").count(),
            1,
            "portable Evaluate is materialized as a legal Java local initializer"
        );
        assert!(!generated.lines().any(|line| line.trim() == "value;"));
    }
}
