use std::collections::{BTreeMap, BTreeSet};

use portable_check::v0::CheckedProgram;
use portable_core_ir::{CoreExpectedOutcome, CoreProgram, CoreValue, lower_checked, verify_core};
use portable_eval::Evaluator;
use portable_ir::v0::{Declaration, ExpectedOutcome, NodeId, Value};
use serde_json::{Value as Json, json};

type Fixture = (&'static str, fn() -> CheckedProgram);

#[test]
fn every_historical_program_retains_evaluator_and_test_semantics_in_core_ir() {
    let fixtures: [Fixture; 13] = [
        (
            "models-and-validation",
            polyrust_models_and_validation_example::program,
        ),
        (
            "escape-string-regexp",
            polyrust_escape_string_regexp_example::program,
        ),
        ("has-flag", polyrust_has_flag_example::program),
        ("html-escaper", polyrust_html_escaper_example::program),
        (
            "is-fullwidth-code-point",
            polyrust_is_fullwidth_code_point_example::program,
        ),
        (
            "normalize-newline",
            polyrust_normalize_newline_example::program,
        ),
        ("parse-ms", polyrust_parse_ms_example::program),
        ("slash", polyrust_slash_example::program),
        ("split-on-first", polyrust_split_on_first_example::program),
        (
            "stdlib-is-negative-zero",
            polyrust_stdlib_is_negative_zero_example::program,
        ),
        ("strip-bom", polyrust_strip_bom_example::program),
        ("trim-newlines", polyrust_trim_newlines_example::program),
        (
            "truncate-utf8-bytes",
            polyrust_truncate_utf8_bytes_example::program,
        ),
    ];

    for (name, build) in fixtures {
        let checked = build();
        let results = Evaluator::new(&checked).run_all_tests();
        assert!(!results.is_empty(), "{name} has no semantic vectors");
        assert!(
            results.iter().all(|result| result.passed),
            "{name}: {results:#?}"
        );

        let first = lower_checked(&checked).unwrap_or_else(|errors| panic!("{name}: {errors:#?}"));
        verify_core(&first).unwrap_or_else(|errors| panic!("{name}: {errors:#?}"));
        let dumps = [
            first.canonical_json(),
            lower_checked(&checked).unwrap().canonical_json(),
            lower_checked(&checked).unwrap().canonical_json(),
        ];
        assert_eq!(dumps[0], dumps[1], "{name} lower pass one/two differs");
        assert_eq!(dumps[1], dumps[2], "{name} lower pass two/three differs");

        let evaluated_names = results
            .iter()
            .map(|result| result.name.as_str())
            .collect::<BTreeSet<_>>();
        let core_names = first
            .tests()
            .iter()
            .map(|test| test.header.name.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(evaluated_names, core_names, "{name} test identity changed");

        for source in checked
            .module()
            .declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Declaration::Test(test) => Some(test),
                _ => None,
            })
        {
            let target = first
                .tests()
                .iter()
                .find(|test| test.header.name == source.header.name)
                .expect("test identity set already compared");
            let source_expected = match &source.expected {
                ExpectedOutcome::Value(value) => ("value", normalize_ir(&checked, &value.value)),
                ExpectedOutcome::Error(value) => ("error", normalize_ir(&checked, &value.value)),
            };
            let core_expected = match &target.expected {
                CoreExpectedOutcome::Value(value) => {
                    ("value", normalize_core(&first, &value.value))
                }
                CoreExpectedOutcome::Error(value) => {
                    ("error", normalize_core(&first, &value.value))
                }
            };
            assert_eq!(
                source_expected, core_expected,
                "{name}/{} expected value changed",
                source.header.name
            );
        }
    }
}

fn normalize_ir(program: &CheckedProgram, value: &Value) -> Json {
    match value {
        Value::Unit => json!({"unit": true}),
        Value::Bool(value) => json!({"bool": value}),
        Value::I32(value) => json!({"i32": value}),
        Value::I64(value) => json!({"i64": value.to_string()}),
        Value::F64(value) => json!({"f64_bits": value.0.to_string()}),
        Value::Char(value) => json!({"char": value.to_string()}),
        Value::String(value) => json!({"string": value}),
        Value::Bytes(value) => json!({"bytes": value}),
        Value::List(values) => Json::Array(
            values
                .iter()
                .map(|value| normalize_ir(program, value))
                .collect(),
        ),
        Value::None => json!({"none": true}),
        Value::Some(value) => json!({"some": normalize_ir(program, value)}),
        Value::Ok(value) => json!({"ok": normalize_ir(program, value)}),
        Value::Err(value) => json!({"err": normalize_ir(program, value)}),
        Value::Record {
            declaration,
            fields,
        } => json!({
            "record": declaration_name(program, *declaration),
            "fields": fields.iter().map(|field| (field_name(program, field.field), normalize_ir(program, &field.value))).collect::<BTreeMap<_, _>>(),
        }),
        Value::Enum {
            declaration,
            variant,
            fields,
        } => json!({
            "enum": declaration_name(program, *declaration),
            "variant": variant_name(program, *variant),
            "fields": fields.iter().map(|field| (field_name(program, field.field), normalize_ir(program, &field.value))).collect::<BTreeMap<_, _>>(),
        }),
    }
}

fn normalize_core(program: &CoreProgram, value: &CoreValue) -> Json {
    match value {
        CoreValue::Unit => json!({"unit": true}),
        CoreValue::Bool(value) => json!({"bool": value}),
        CoreValue::I32(value) => json!({"i32": value}),
        CoreValue::I64(value) => json!({"i64": value.to_string()}),
        CoreValue::F64(value) => json!({"f64_bits": value.0.to_string()}),
        CoreValue::Char(value) => json!({"char": value.to_string()}),
        CoreValue::String(value) => json!({"string": value}),
        CoreValue::Bytes(value) => json!({"bytes": value}),
        CoreValue::List(values) => Json::Array(
            values
                .iter()
                .map(|value| normalize_core(program, value))
                .collect(),
        ),
        CoreValue::None => json!({"none": true}),
        CoreValue::Some(value) => json!({"some": normalize_core(program, value)}),
        CoreValue::Ok(value) => json!({"ok": normalize_core(program, value)}),
        CoreValue::Err(value) => json!({"err": normalize_core(program, value)}),
        CoreValue::Record { record, fields } => json!({
            "record": program.record(*record).unwrap().header.name,
            "fields": fields.iter().map(|field| (program.field(field.field).unwrap().header.name.clone(), normalize_core(program, &field.value))).collect::<BTreeMap<_, _>>(),
        }),
        CoreValue::Enum {
            enumeration,
            variant,
            fields,
        } => json!({
            "enum": program.enumeration(*enumeration).unwrap().header.name,
            "variant": program.variant(*variant).unwrap().header.name,
            "fields": fields.iter().map(|field| (program.field(field.field).unwrap().header.name.clone(), normalize_core(program, &field.value))).collect::<BTreeMap<_, _>>(),
        }),
    }
}

fn declaration_name(program: &CheckedProgram, id: NodeId) -> String {
    program
        .module()
        .declarations
        .iter()
        .find(|value| value.header().node.id == id)
        .unwrap()
        .header()
        .name
        .clone()
}

fn field_name(program: &CheckedProgram, id: NodeId) -> String {
    program
        .module()
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Record(record) => record
                .fields
                .iter()
                .find(|field| field.header.node.id == id)
                .map(|field| field.header.name.clone()),
            Declaration::Enum(enumeration) => enumeration.variants.iter().find_map(|variant| {
                variant
                    .fields
                    .iter()
                    .find(|field| field.header.node.id == id)
                    .map(|field| field.header.name.clone())
            }),
            _ => None,
        })
        .unwrap()
}

fn variant_name(program: &CheckedProgram, id: NodeId) -> String {
    program
        .module()
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Enum(enumeration) => enumeration
                .variants
                .iter()
                .find(|variant| variant.header.node.id == id)
                .map(|variant| variant.header.name.clone()),
            _ => None,
        })
        .unwrap()
}
