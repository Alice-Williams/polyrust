//! Native Java21 producer/consumer proof independent of the generated operation.
use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::tests::source_constants_native::tool;
fn compile(source: &Path, classes: &Path) {
    let output = Command::new(tool("javac"))
        .args([
            "--release",
            "21",
            "-encoding",
            "UTF-8",
            "-Xlint:all",
            "-Werror",
            "-implicit:none",
            "-sourcepath",
            "",
            "-cp",
        ])
        .arg(classes)
        .arg("-d")
        .arg(classes)
        .arg(source)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
fn cases() -> String {
    let mut values = vec![
        i64::MIN,
        i64::MIN + 1,
        i64::MAX,
        -1,
        0,
        1,
        i64::from(i32::MIN),
        i64::from(i32::MAX),
    ];
    for bit in 0..63 {
        let n = 1_i64 << bit;
        values.extend([n - 1, n, n + 1, -n - 1, -n, -n + 1]);
    }
    let mut seed = 0x83e51a93f6294db7_u64;
    for _ in 0..4096 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        values.push(seed as i64);
    }
    values
        .into_iter()
        .map(|wide| {
            let small = wide as i32;
            let a = if small == i32::MIN {
                i64::from(small)
            } else {
                -i64::from(small)
            };
            let b = if wide == i64::MIN {
                i128::from(wide)
            } else {
                -i128::from(wide)
            };
            format!("{small} {a} {wide} {b}\n")
        })
        .collect()
}
#[test]
fn wrapping_negation_java_native_boundaries_and_wrong_operation_control() {
    let root =
        PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-wrapping-negation");
    fs::create_dir_all(&root).unwrap();
    for mutant in [false, true] {
        let directory = root.join(if mutant { "wrong-operation" } else { "valid" });
        let classes = directory.join("classes");
        fs::create_dir_all(&classes).unwrap();
        let operator = if mutant {
            JavaUnaryOperator::BitNot
        } else {
            JavaUnaryOperator::Negate
        };
        for owner in chain_with_operator(operator) {
            let output =
                render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
            assert_eq!(output.files().len(), 1);
            for file in output.files() {
                let OutputContents::Text(text) = file.contents() else {
                    panic!("Java source")
                };
                let path = directory.join(file.path());
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, text).unwrap();
                compile(&path, &classes);
            }
        }
        let source = directory.join("Consumer.java");
        fs::write(&source, r#"import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
public final class Consumer {
    public static void main(String[] args) throws Exception {
        int rows = 0;
        try (BufferedReader input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8))) {
            String line;
            while ((line = input.readLine()) != null) {
                String[] parts = line.split(" ");
                int a = Integer.parseInt(parts[0]), expectedA = Integer.parseInt(parts[1]);
                long b = Long.parseLong(parts[2]), expectedB = Long.parseLong(parts[3]);
                if (org.polyrust.generated.r0000000000000051.Generated.negate32(a) != expectedA ||
                    org.polyrust.generated.r0000000000000051.Generated.negate64(b) != expectedB ||
                    org.polyrust.generated.r0000000000000052.Generated.negate32(a) != a ||
                    org.polyrust.generated.r0000000000000052.Generated.negate64(b) != b)
                    throw new AssertionError("wrapping value");
                ++rows;
            }
        }
        if (rows != 4482) throw new AssertionError("row count");
    }
}
"#).unwrap();
        compile(&source, &classes);
        fs::write(directory.join("cases.txt"), cases()).unwrap();
        let output = Command::new(tool("java"))
            .arg("-cp")
            .arg(&classes)
            .arg("Consumer")
            .stdin(fs::File::open(directory.join("cases.txt")).unwrap())
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(if mutant { 1 } else { 0 }),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if mutant {
            assert!(String::from_utf8_lossy(&output.stderr).contains("wrapping value"));
        } else {
            assert!(output.stderr.is_empty());
        }
    }
}

#[test]
fn wrapping_negation_negative_literals_and_nested_negation_compile_and_execute() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap())
        .join("java-wrapping-negation-literals");
    let classes = root.join("classes");
    fs::create_dir_all(&classes).unwrap();
    let mut declarations = Vec::new();
    let mut assertions = String::new();
    for wide in [false, true] {
        let (primitive, minimum, maximum, suffix) = if wide {
            (JavaPrimitive::Long, i64::MIN, i64::MAX, "L")
        } else {
            (
                JavaPrimitive::Int,
                i64::from(i32::MIN),
                i64::from(i32::MAX),
                "",
            )
        };
        for number in [minimum, minimum + 1, -1, 0, 1, maximum] {
            for nested in [false, true] {
                let ty = JavaType::primitive(primitive);
                let literal = if wide {
                    JavaLiteral::I64(number)
                } else {
                    JavaLiteral::I32(number.try_into().unwrap())
                };
                let mut value = negate(JavaExpr::literal(ty.clone(), literal));
                if nested {
                    value = negate(value);
                }
                let index = declarations.len();
                let name = format!("literal{index}");
                declarations.push(f::Function {
                    hash: 100 + index as u64,
                    public: true,
                    name: f::name(&name),
                    parameters: vec![],
                    result: ty,
                    body: JavaBlock::new(vec![JavaStmt::Return(Some(value))]),
                });
                let expected = if nested || number == minimum {
                    i128::from(number)
                } else {
                    -i128::from(number)
                };
                assertions.push_str(&format!(
                    "if (org.polyrust.generated.r0000000000000051.Generated.{name}() != {expected}{suffix}) throw new AssertionError(\"{name}\");\n"
                ));
            }
        }
    }
    let api =
        JavaDependencyApi::from_certificate(f::certify(f::package(81, declarations))).unwrap();
    let output = render_certified_package(&JavaStructuralRenderer, api.package()).unwrap();
    for file in output.files() {
        let OutputContents::Text(text) = file.contents() else {
            panic!("Java source")
        };
        assert!(!text.contains("--"), "negation must not become decrement");
        assert!(text.len() as u64 <= api.source_byte_bound().unwrap());
        let path = root.join(file.path());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        compile(&path, &classes);
    }
    let source = root.join("LiteralConsumer.java");
    fs::write(&source, format!(
        "public final class LiteralConsumer {{ public static void main(String[] args) {{ {assertions} }} }}"
    )).unwrap();
    compile(&source, &classes);
    let output = Command::new(tool("java"))
        .arg("-cp")
        .arg(classes)
        .arg("LiteralConsumer")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
