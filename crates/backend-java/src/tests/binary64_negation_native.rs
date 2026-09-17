//! Exact non-NaN sign bits, NaN categories and observable single operand calls.
use super::*;
use crate::tests::source_constants_native::{compile, tool};
use std::{fs, path::PathBuf, process::Command};

#[cfg(test)]
fn body(text: &str, member: &str) -> (usize, usize) {
    let start = text.find(&format!("{member}(")).unwrap();
    let brace = start + text[start..].find('{').unwrap() + 1;
    let end = brace + text[brace..].find('}').unwrap();
    (brace, end)
}
#[test]
fn exact_double_negation_and_native_value_trace_mutants() {
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-binary64-negation");
    let first = owner(96, None);
    let second = owner(97, Some(&first));
    let cases: Vec<_> = super::super::comparisons::cases()
        .into_iter()
        .map(|(bits, _)| bits)
        .collect();
    for fault in 0..5 {
        let directory = root.join(format!("fault-{fault}"));
        let classes = directory.join("classes");
        fs::create_dir_all(&classes).unwrap();
        for (index, api) in [&first, &second].into_iter().enumerate() {
            let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
            for file in output.files() {
                let OutputContents::Text(original) = file.contents() else {
                    panic!("text")
                };
                assert!(original.len() as u64 <= api.source_byte_bound().unwrap());
                let mut text = original.clone();
                if index == 0 {
                    if fault == 1 || fault == 2 {
                        let (start, end) = body(&text, "negate0");
                        text.replace_range(
                            start..end,
                            if fault == 1 {
                                "\nreturn input;\n"
                            } else {
                                "\nreturn 0.0 - input;\n"
                            },
                        );
                    }
                    for (member, marker) in [("negate0", "A"), ("negate1", "B")] {
                        let (start, _) = body(&text, member);
                        text.insert_str(
                            start,
                            &format!("\njava.lang.System.err.print(\"{marker}\");\n"),
                        );
                    }
                } else if fault == 3 || fault == 4 {
                    let (start, end) = body(&text, "negate0");
                    if fault == 3 {
                        text.replace_range(start..end, "\nreturn input;\n");
                    } else {
                        text.insert_str(start, "\norg.polyrust.generated.r0000000000000060.Generated.negate0(input);\n");
                    }
                }
                let path = directory.join(file.path());
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, text).unwrap();
                let result = compile(&path, &classes);
                assert!(
                    result.status.success(),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
        let mut assertions = String::new();
        for bits in &cases {
            assertions.push_str(&format!(
                "{{ double value = Double.longBitsToDouble(0x{bits:016x}L);\n"
            ));
            for (crate_id, index, flip) in
                [(96, 0, true), (96, 1, false), (97, 0, false), (97, 1, true)]
            {
                let expected = bits ^ if flip { 1 << 63 } else { 0 };
                assertions.push_str(&format!("check(org.polyrust.generated.r{crate_id:016x}.Generated.negate{index}(value), 0x{expected:016x}L);\n"));
            }
            assertions.push_str("}\n");
        }
        let source = directory.join("Consumer.java");
        fs::write(
            &source,
            format!(
                r#"public final class Consumer {{
static void check(double actual, long expected) {{
    boolean nan = (expected & 0x7fffffffffffffffL) > 0x7ff0000000000000L;
    if (nan ? !Double.isNaN(actual) : Double.doubleToRawLongBits(actual) != expected)
        throw new AssertionError("negation bits");
}}
public static void main(String[] args) {{ {assertions} }}
}}"#
            ),
        )
        .unwrap();
        let result = compile(&source, &classes);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let output = Command::new(tool("java"))
            .arg("-cp")
            .arg(&classes)
            .arg("Consumer")
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(i32::from(fault == 1 || fault == 2))
        );
        let trace = String::from_utf8(output.stderr).unwrap();
        if fault == 1 || fault == 2 {
            assert!(trace.contains("negation bits"), "{trace}");
        } else {
            let expected = match fault {
                3 => "ABB",
                4 => "ABAAB",
                _ => "ABAB",
            }
            .repeat(cases.len());
            assert_eq!(trace, expected);
            assert_eq!(trace == "ABAB".repeat(cases.len()), fault == 0);
        }
    }
}
