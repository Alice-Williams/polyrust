//! Raw-bit consumer checks, strict Java21 compilation and semantic mutants.
use super::*;
use crate::tests::source_constants_native::{compile, tool};
use std::{fs, path::PathBuf, process::Command};

#[test]
fn finite_bits_native_and_sign_exponent_subnormal_mutants() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-binary64");
    for mutation in 0..4 {
        let expected = if mutation == 0 {
            values()
        } else {
            values()[..8].to_vec()
        };
        let mut actual = expected.clone();
        match mutation {
            1 => actual[0] ^= 1 << 63,
            2 => actual[3] ^= 1 << 52,
            3 => actual[1] = 2,
            _ => {}
        }
        let directory = root.join(format!("case-{mutation}"));
        let classes = directory.join("classes");
        fs::create_dir_all(&classes).unwrap();
        for owner in chain(&actual)
            .into_iter()
            .chain([comparisons::comparisons(), records::record()])
        {
            let output =
                render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
            for file in output.files() {
                let OutputContents::Text(text) = file.contents() else {
                    panic!("source")
                };
                assert!(text.len() as u64 <= owner.source_byte_bound().unwrap());
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
        for owner in [91, 92] {
            let prefix = format!("org.polyrust.generated.r{owner:016x}.Generated");
            for (index, bits) in expected.iter().enumerate() {
                assertions.push_str(&format!("if (Double.doubleToRawLongBits({prefix}.literal{index}()) != 0x{bits:016x}L) throw new AssertionError(\"finite bits\");\n"));
                assertions.push_str(&format!("if (Double.doubleToRawLongBits({prefix}.identity(Double.longBitsToDouble(0x{bits:016x}L))) != 0x{bits:016x}L) throw new AssertionError(\"transport\");\n"));
            }
            assertions.push_str(&format!("if (!Double.isNaN({prefix}.identity(Double.NaN)) || {prefix}.identity(Double.POSITIVE_INFINITY) != Double.POSITIVE_INFINITY || {prefix}.identity(Double.NEGATIVE_INFINITY) != Double.NEGATIVE_INFINITY) throw new AssertionError(\"nonfinite transport\");\n"));
        }
        for bits in &expected {
            assertions.push_str(&format!("if (Double.doubleToRawLongBits(org.polyrust.generated.r0000000000000007.Generated.inspect(Double.longBitsToDouble(0x{bits:016x}L), true)) != 0x{bits:016x}L) throw new AssertionError(\"record transport\");\n"));
        }
        for (raw, expectations) in comparisons::cases() {
            let expected = expectations.map(|value| value.to_string()).join(", ");
            assertions.push_str(&format!(
                "check(Double.longBitsToDouble(0x{raw:016x}L), {expected});\n"
            ));
        }
        let checks: String = (0..6).map(|index| format!(
            "if (org.polyrust.generated.r000000000000005d.Generated.compare{index}(value) != e{index}) throw new AssertionError(\"comparison\");"
        )).collect();
        let parameters = (0..6)
            .map(|index| format!("boolean e{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let helper = format!("static void check(double value, {parameters}) {{ {checks} }}");
        let source = directory.join("Consumer.java");
        fs::write(&source, format!("public final class Consumer {{ {helper} public static void main(String[] args) {{ {assertions} }} }}")).unwrap();
        let result = compile(&source, &classes);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let output = Command::new(tool("java"))
            .arg("-cp")
            .arg(classes)
            .arg("Consumer")
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(i32::from(mutation != 0)),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if mutation != 0 {
            assert!(String::from_utf8_lossy(&output.stderr).contains("finite bits"));
        } else {
            assert!(output.stderr.is_empty());
        }
    }
}
