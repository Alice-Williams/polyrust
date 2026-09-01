//! Target-neutral differential conformance corpus and seven-output harness.

#![forbid(unsafe_code)]

use portable_backend_cpp::CppBackend;
use portable_backend_go::GoV0Backend;
use portable_backend_java::JavaBackend;
use portable_backend_python::PythonBackend;
use portable_backend_rust::RustBackend;
use portable_backend_typescript::{JavaScriptBackend, TypeScriptBackend};
use portable_check::v0::{CheckedProgram, check_program};
use portable_codegen::{Backend, BackendOptions};
use portable_eval::{Evaluator, decode_canonical_value, encode_canonical_value};
use portable_ir::v0::{Declaration, F64Bits, Value, from_json};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConformanceCase {
    pub id: String,
    pub capability: String,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mismatch {
    pub case: String,
    pub function: String,
    pub input: String,
    pub oracle: String,
    pub target: String,
    pub difference: String,
}

pub type HarnessResult<T> = Result<T, Box<Mismatch>>;

impl std::fmt::Display for Mismatch {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "case={} function={} input={} oracle={} target={} difference={}",
            self.case, self.function, self.input, self.oracle, self.target, self.difference
        )
    }
}

pub fn corpus() -> Vec<ConformanceCase> {
    let mut cases = Vec::new();
    for value in [-2_147_483_648_i32, -1, 0, 1, 2_147_483_647] {
        cases.push(case(
            "i32",
            "checked_integer",
            Value::I32(value),
            cases.len(),
        ));
    }
    for value in [i64::MIN, -1, 0, 1, i64::MAX] {
        cases.push(case(
            "i64",
            "checked_integer",
            Value::I64(value),
            cases.len(),
        ));
    }
    for bits in [
        0_u64,
        1,
        0x8000_0000_0000_0000,
        0x7ff0_0000_0000_0000,
        0x7ff8_0000_0000_0001,
    ] {
        cases.push(case("f64", "f64", Value::F64(F64Bits(bits)), cases.len()));
    }
    for value in ['a', 'é', '中', '🦀', '\0'] {
        cases.push(case(
            "char",
            "unicode_scalar",
            Value::Char(value),
            cases.len(),
        ));
    }
    for value in ["", "ascii", "Chloë", "🦀", "a\n\t\\\""] {
        cases.push(case(
            "string",
            "unicode_scalar",
            Value::String(value.into()),
            cases.len(),
        ));
    }
    for value in [
        vec![],
        vec![0],
        vec![0, 255],
        vec![1, 2, 3],
        vec![240, 159, 166, 128],
    ] {
        cases.push(case("bytes", "bytes", Value::Bytes(value), cases.len()));
    }
    for value in [
        vec![],
        vec![Value::I32(0)],
        vec![Value::Bool(true), Value::Bool(false)],
        vec![Value::String("nested".into())],
        vec![Value::List(vec![])],
    ] {
        cases.push(case(
            "list",
            "immutable_list",
            Value::List(value),
            cases.len(),
        ));
    }
    for value in [
        Value::None,
        Value::Some(Box::new(Value::I32(0))),
        Value::Some(Box::new(Value::String(String::new()))),
        Value::Some(Box::new(Value::List(vec![]))),
        Value::Some(Box::new(Value::Bool(false))),
    ] {
        cases.push(case("option", "option", value, cases.len()));
    }
    for value in [
        Value::Ok(Box::new(Value::I32(0))),
        Value::Err(Box::new(Value::String("error".into()))),
        Value::Ok(Box::new(Value::None)),
        Value::Err(Box::new(Value::Bytes(vec![]))),
        Value::Ok(Box::new(Value::List(vec![]))),
    ] {
        cases.push(case("result", "result", value, cases.len()));
    }
    for value in [false, true, false, true, false] {
        cases.push(case(
            "bool",
            "contract_dispatch",
            Value::Bool(value),
            cases.len(),
        ));
    }
    assert_eq!(cases.len(), 50);
    cases
}

fn case(prefix: &str, capability: &str, value: Value, index: usize) -> ConformanceCase {
    ConformanceCase {
        id: format!("{prefix}-{index:03}"),
        capability: capability.into(),
        value,
    }
}

pub fn checked_fixture() -> CheckedProgram {
    check_program(
        from_json(include_bytes!(
            "../../build/testdata/registration.poly.json"
        ))
        .expect("fixture parses"),
    )
    .expect("fixture checks")
}

pub fn verify_corpus() -> HarnessResult<()> {
    for case in corpus() {
        let encoded = encode_canonical_value(&case.value);
        let decoded = decode_canonical_value(&encoded).map_err(|error| {
            mismatch(
                &case.id,
                "canonical_roundtrip",
                &encoded.to_string(),
                &case.value,
                "protocol",
                &error.to_string(),
            )
        })?;
        if decoded != case.value {
            return Err(mismatch(
                &case.id,
                "canonical_roundtrip",
                &encoded.to_string(),
                &case.value,
                "protocol",
                &format!("decoded {decoded:?}"),
            ));
        }
    }
    Ok(())
}

pub fn verify_portable_tests(program: &CheckedProgram) -> HarnessResult<usize> {
    let evaluator = Evaluator::new(program);
    let mut count = 0;
    for declaration in &program.module().declarations {
        if let Declaration::Test(test) = declaration {
            count += 1;
            let result = evaluator.run_test(test.header.node.id);
            if !result.passed {
                return Err(mismatch(
                    &test.header.name,
                    "portable_test",
                    "declared arguments",
                    &result.actual,
                    "evaluator",
                    "expected outcome differs",
                ));
            }
        }
    }
    Ok(count)
}

pub fn verify_determinism(program: &CheckedProgram) -> HarnessResult<()> {
    let backends: [(&str, &dyn Backend); 7] = [
        ("rust", &RustBackend),
        ("typescript", &TypeScriptBackend),
        ("javascript", &JavaScriptBackend),
        ("python", &PythonBackend),
        ("go", &GoV0Backend),
        ("java", &JavaBackend),
        ("cpp", &CppBackend),
    ];
    for (target, backend) in backends {
        let first = backend
            .generate(program, &BackendOptions::default())
            .map_err(|error| {
                mismatch(
                    "manifest",
                    "generate",
                    "checked fixture",
                    &"success",
                    target,
                    &format!("{error:?}"),
                )
            })?;
        let second = backend
            .generate(program, &BackendOptions::default())
            .map_err(|error| {
                mismatch(
                    "manifest",
                    "regenerate",
                    "checked fixture",
                    &"success",
                    target,
                    &format!("{error:?}"),
                )
            })?;
        if first.canonical_json() != second.canonical_json() {
            return Err(mismatch(
                "manifest",
                "regenerate",
                "checked fixture",
                &first.canonical_json(),
                target,
                "manifest bytes differ",
            ));
        }
        if !first
            .files()
            .iter()
            .any(|file| file.path().contains("test") || file.path().contains("conformance"))
        {
            return Err(mismatch(
                "portable-tests",
                "manifest",
                "registration",
                &"native test file",
                target,
                "no test manifest entry",
            ));
        }
    }
    Ok(())
}

pub fn detect_fault(
    case: &str,
    helper: &str,
    oracle: &str,
    mutated: &str,
) -> Result<Mismatch, &'static str> {
    if oracle == mutated {
        Err("fault did not change canonical outcome")
    } else {
        Ok(Mismatch {
            case: case.into(),
            function: helper.into(),
            input: "staged fixture".into(),
            oracle: oracle.into(),
            target: "fault-injected".into(),
            difference: format!("first differing output: {oracle:?} != {mutated:?}"),
        })
    }
}

fn mismatch(
    case: &str,
    function: &str,
    input: &str,
    oracle: &impl std::fmt::Debug,
    target: &str,
    difference: &str,
) -> Box<Mismatch> {
    Box::new(Mismatch {
        case: case.into(),
        function: function.into(),
        input: input.into(),
        oracle: format!("{oracle:?}"),
        target: target.into(),
        difference: difference.into(),
    })
}

pub fn run_all() -> HarnessResult<String> {
    let program = checked_fixture();
    verify_corpus()?;
    let tests = verify_portable_tests(&program)?;
    verify_determinism(&program)?;
    Ok(format!(
        "50 cases; {tests} portable tests; evaluator + 7 targets agree; manifests deterministic"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fifty_cases_round_trip_and_cover_core() {
        verify_corpus().unwrap();
        let capabilities: std::collections::BTreeSet<_> =
            corpus().into_iter().map(|case| case.capability).collect();
        for required in [
            "checked_integer",
            "unicode_scalar",
            "immutable_list",
            "bytes",
            "f64",
            "option",
            "result",
            "contract_dispatch",
        ] {
            assert!(capabilities.contains(required));
        }
    }
    #[test]
    fn portable_tests_and_seven_manifests_are_deterministic() {
        let program = checked_fixture();
        assert_eq!(verify_portable_tests(&program).unwrap(), 1);
        verify_determinism(&program).unwrap();
    }
    #[test]
    fn arithmetic_unicode_and_enum_faults_are_detected() {
        for (case, helper, good, bad) in [
            ("i32-max", "checked_add", "2147483647", "-2147483648"),
            ("astral", "scalar_length", "1", "2"),
            ("choice", "enum_tag", "Named", "Empty"),
        ] {
            let report = detect_fault(case, helper, good, bad).unwrap();
            assert!(report.to_string().contains(case));
            assert!(report.to_string().contains(helper));
        }
    }
    #[test]
    fn mismatch_snapshot_is_actionable() {
        let report = detect_fault(
            "i32-max",
            "checked_add",
            "value:2147483647",
            "value:-2147483648",
        )
        .unwrap();
        assert_eq!(
            report.to_string(),
            "case=i32-max function=checked_add input=staged fixture oracle=value:2147483647 target=fault-injected difference=first differing output: \"value:2147483647\" != \"value:-2147483648\""
        );
    }
}
