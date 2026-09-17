//! Separately compiled owner classes; exact bit and call-trace observations.
use super::*;
use crate::tests::source_constants_native::{compile, tool};
use std::{fs, path::PathBuf, process::Command};
#[derive(Clone, Copy, Debug)]
enum Fault {
    None,
    Swapped,
    MissingZero,
    WrongCondition,
    Dropped,
    Duplicated,
}
impl Fault {
    const ALL: [Self; 6] = [
        Self::None,
        Self::Swapped,
        Self::MissingZero,
        Self::WrongCondition,
        Self::Dropped,
        Self::Duplicated,
    ];
    fn wrong_value(self) -> bool {
        matches!(
            self,
            Self::Swapped | Self::MissingZero | Self::WrongCondition
        )
    }
    fn trace(self) -> &'static str {
        match self {
            Self::Dropped => "ABCDBCD",
            Self::Duplicated => "ABCDAABCD",
            _ => "ABCDABCD",
        }
    }
}

#[cfg(test)]
fn body(text: &str, member: &str) -> (usize, usize) {
    let start = text.find(&format!("{member}(")).unwrap();
    let brace = start + text[start..].find('{').unwrap() + 1;
    let end = brace + text[brace..].find('}').unwrap();
    (brace, end)
}
#[test]
fn exact_double_conditional_selection_and_native_faults() {
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-binary64-conditional");
    let first = owner(98, None);
    let second = owner(99, Some(&first));
    let cases: Vec<_> = super::super::comparisons::cases()
        .into_iter()
        .map(|(bits, _)| bits)
        .collect();
    for fault in Fault::ALL {
        let directory = root.join(format!("{fault:?}"));
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
                    let replacement = match fault {
                        Fault::Swapped => Some(("select0", "\nreturn -input;\n")),
                        Fault::MissingZero => {
                            Some(("select3", "\nreturn input < 0.0 ? -input : input;\n"))
                        }
                        Fault::WrongCondition => {
                            Some(("select2", "\nreturn input >= 0.0 ? input : -input;\n"))
                        }
                        _ => None,
                    };
                    if let Some((member, replacement)) = replacement {
                        let (start, end) = body(&text, member);
                        text.replace_range(start..end, replacement);
                    }
                    for (member, marker) in [
                        ("select0", "A"),
                        ("select1", "B"),
                        ("select2", "C"),
                        ("select3", "D"),
                    ] {
                        let (start, _) = body(&text, member);
                        text.insert_str(
                            start,
                            &format!("\njava.lang.System.err.print(\"{marker}\");\n"),
                        );
                    }
                } else {
                    let (start, end) = body(&text, "select0");
                    match fault {
                        Fault::Dropped=>text.replace_range(start..end,"\nreturn input;\n"),
                        Fault::Duplicated=>text.insert_str(start,"\norg.polyrust.generated.r0000000000000062.Generated.select0(input);\n"),
                        _=>{},
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
                "{{ double value=Double.longBitsToDouble(0x{bits:016x}L);\n"
            ));
            for (crate_id, twice) in [(98, false), (99, true)] {
                for (index, mode) in Selection::ALL.into_iter().enumerate() {
                    let expected = mode.expected(*bits);
                    let expected = if twice {
                        mode.expected(expected)
                    } else {
                        expected
                    };
                    assertions.push_str(&format!("check(org.polyrust.generated.r{crate_id:016x}.Generated.select{index}(value),0x{expected:016x}L);\n"));
                }
            }
            assertions.push_str("}\n");
        }
        let source = directory.join("Consumer.java");
        fs::write(
            &source,
            format!(
                r#"public final class Consumer {{
static void check(double actual, long expected) {{
    boolean nan=(expected & 0x7fffffffffffffffL)>0x7ff0000000000000L;
    if (nan ? !Double.isNaN(actual) : Double.doubleToRawLongBits(actual)!=expected)
        throw new AssertionError("conditional bits");
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
            Some(i32::from(fault.wrong_value())),
            "{fault:?}"
        );
        let trace = String::from_utf8(output.stderr).unwrap();
        if fault.wrong_value() {
            assert!(trace.contains("conditional bits"), "{trace}");
        } else {
            let expected = fault.trace().repeat(cases.len());
            assert_eq!(trace, expected, "{fault:?}");
            assert_eq!(
                trace == "ABCDABCD".repeat(cases.len()),
                matches!(fault, Fault::None)
            );
        }
    }
}
