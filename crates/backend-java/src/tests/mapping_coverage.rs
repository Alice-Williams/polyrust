use super::*;
use std::collections::BTreeSet;

#[test]
fn every_registered_java_capability_mapping_is_invoked() {
    crate::capabilities::reset_java_mapping_invocations();

    JavaBackend
        .generate(&fixture(), &BackendOptions::default())
        .expect("complete registration fixture generates");
    JavaBackend
        .generate(&capability_coverage_fixture(), &BackendOptions::default())
        .expect("capability coverage fixture generates");
    let interfaces =
        portable_check::v0::check_program(portable_build::interface_composition_fixture().document)
            .expect("interface composition fixture checks");
    JavaBackend
        .generate(&interfaces, &BackendOptions::default())
        .expect("interface composition fixture generates");
    JavaBackend
        .generate(
            &portable_method_invocation_fixture(),
            &BackendOptions::default(),
        )
        .expect("portable method invocation fixture generates");
    let _ = typed_fixture_manifests();
    let empty_interface = typed_program(portable_name!("empty_interface_coverage"), |builder| {
        builder.interface(portable_name!("Empty"), typed_list![], |builder, _| builder)
    });
    let _ = JavaBackend.generate_typed(&empty_interface);

    let counts = crate::capabilities::java_mapping_operation_counts()
        .into_iter()
        .map(|(name, count)| (name.rsplit("::").next().unwrap(), count))
        .collect::<std::collections::BTreeMap<_, _>>();
    for (capability, expected) in [
        ("Modules", 1),
        ("Constants", 2),
        ("TypeAliases", 1),
        ("Conditionals", 1),
        ("LocalBindings", 2),
        ("PatternMatching", 3),
        ("ResultPropagation", 1),
        ("UnitValues", 2),
        ("BoolValues", 2),
        ("I32Values", 2),
        ("I64Values", 2),
        ("F64Values", 2),
        ("TextValues", 2),
        ("CharValues", 2),
        ("BytesValues", 2),
        ("ListValues", 2),
        ("OptionValues", 3),
        ("ResultValues", 3),
        ("StringConcatenation", 1),
        ("BooleanLogic", 3),
        ("Equality", 2),
        ("Ordering", 4),
        ("CheckedIntegerArithmetic", 6),
        ("WrappingIntegerArithmetic", 4),
        ("IntegerBitwise", 4),
        ("CheckedIntegerShifts", 2),
        ("FloatingPointArithmetic", 6),
        ("FloatingPointInspection", 4),
        ("IntegerConversions", 2),
        ("StringInspection", 7),
        ("StringTransformation", 7),
        ("BytesOperations", 4),
        ("ListOperations", 7),
        ("OptionOperations", 3),
        ("ResultOperations", 2),
        ("Utf8Conversions", 2),
        ("Functions", 4),
        ("Records", 4),
        ("Interfaces", 8),
        ("Enums", 7),
        ("Loops", 2),
        ("PortableTests", 4),
    ] {
        assert_eq!(
            counts.get(capability),
            Some(&expected),
            "not every input variant reached the {capability} mapping"
        );
    }

    let actual = crate::capabilities::java_mapping_invocations()
        .into_iter()
        .map(|name| name.rsplit("::").next().expect("capability type name"))
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from([
        "Functions",
        "Records",
        "BoolValues",
        "I32Values",
        "I64Values",
        "F64Values",
        "TextValues",
        "BooleanLogic",
        "Equality",
        "Ordering",
        "CheckedIntegerArithmetic",
        "WrappingIntegerArithmetic",
        "FloatingPointArithmetic",
        "StringConcatenation",
        "CharValues",
        "BytesValues",
        "ListValues",
        "OptionValues",
        "ResultValues",
        "IntegerBitwise",
        "CheckedIntegerShifts",
        "FloatingPointInspection",
        "StringInspection",
        "StringTransformation",
        "BytesOperations",
        "ListOperations",
        "OptionOperations",
        "ResultOperations",
        "IntegerConversions",
        "Utf8Conversions",
        "Modules",
        "Constants",
        "TypeAliases",
        "Enums",
        "Interfaces",
        "PortableTests",
        "LocalBindings",
        "Conditionals",
        "Loops",
        "PatternMatching",
        "ResultPropagation",
        "UnitValues",
    ]);
    assert_eq!(actual, expected, "uninvoked Java capability mapping");
}
