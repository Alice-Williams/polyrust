//! Separately compiled owner classes; exact bit and call-trace observations.
use super::*;
use crate::tests::source_constants_native::{compile, tool};
use std::{fs, path::PathBuf, process::Command};
#[derive(Clone, Copy, Debug)]
enum Fault {
    None,
    WrongFloor,
    WrongCeil,
    ZeroSign,
    WrongCondition,
    Dropped,
    Duplicated,
}
impl Fault {
    const ALL: [Self; 7] = [
        Self::None,
        Self::WrongFloor,
        Self::WrongCeil,
        Self::ZeroSign,
        Self::WrongCondition,
        Self::Dropped,
        Self::Duplicated,
    ];
    fn wrong_value(self) -> bool {
        matches!(
            self,
            Self::WrongFloor | Self::WrongCeil | Self::ZeroSign | Self::WrongCondition
        )
    }
    fn trace(self) -> &'static str {
        match self {
            Self::Dropped => "ABCAB",
            Self::Duplicated => "ABCABCC",
            _ => "ABCABC",
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
fn exact_double_rounding_selection_and_native_faults() {
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-binary64-rounding");
    let first = owner(101, None);
    let second = owner(102, Some(&first));
    let cases = cases();
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
                        Fault::WrongFloor => {
                            Some(("round0", "\nreturn java.lang.Math.ceil(input);\n"))
                        }
                        Fault::WrongCeil => {
                            Some(("round1", "\nreturn java.lang.Math.floor(input);\n"))
                        }
                        Fault::ZeroSign => Some((
                            "round2",
                            "\nreturn input == 0.0 ? 0.0 : (input < 0.0 ? java.lang.Math.ceil(input) : java.lang.Math.floor(input));\n",
                        )),
                        Fault::WrongCondition => Some((
                            "round2",
                            "\nreturn input < 0.0 ? java.lang.Math.floor(input) : java.lang.Math.ceil(input);\n",
                        )),
                        _ => None,
                    };
                    if let Some((member, replacement)) = replacement {
                        let (start, end) = body(&text, member);
                        text.replace_range(start..end, replacement);
                    }
                    for (member, marker) in [("round0", "A"), ("round1", "B"), ("round2", "C")] {
                        let (start, _) = body(&text, member);
                        text.insert_str(
                            start,
                            &format!("\njava.lang.System.err.print(\"{marker}\");\n"),
                        );
                    }
                } else {
                    let (start, end) = body(&text, "round2");
                    match fault {
                        Fault::Dropped=>text.replace_range(start..end,"\nreturn input < 0.0 ? java.lang.Math.ceil(input) : java.lang.Math.floor(input);\n"),
                        Fault::Duplicated=>text.insert_str(start,"\norg.polyrust.generated.r0000000000000065.Generated.round2(input);\n"),
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
            for (crate_id, twice) in [(101, false), (102, true)] {
                for (index, mode) in Mode::ALL.into_iter().enumerate() {
                    let expected = mode.expected(*bits);
                    let expected = if twice {
                        mode.expected(expected)
                    } else {
                        expected
                    };
                    assertions.push_str(&format!("check(org.polyrust.generated.r{crate_id:016x}.Generated.round{index}(value),0x{expected:016x}L);\n"));
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
        throw new AssertionError("rounding bits");
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
            assert!(trace.contains("rounding bits"), "{trace}");
        } else {
            let expected = fault.trace().repeat(cases.len());
            assert_eq!(trace, expected, "{fault:?}");
            assert_eq!(
                trace == "ABCABC".repeat(cases.len()),
                matches!(fault, Fault::None)
            );
        }
    }
}

fn cases() -> Vec<u64> {
    let mut bits: Vec<_> = super::super::comparisons::cases()
        .into_iter()
        .map(|(bits, _)| bits)
        .collect();
    for exponent in [0_u64, 1, 1022, 1023, 1024, 1074, 1075, 1076, 2046] {
        for fraction in [0, 1, (1 << 51) - 1, 1 << 51, (1 << 52) - 1] {
            for sign in [0, 1 << 63] {
                bits.push(sign | (exponent << 52) | fraction);
            }
        }
    }
    bits
}
