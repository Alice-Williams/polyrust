//! Typed local materialization with value-preserving dropped/reordered calls.
use super::*;
use crate::tests::source_constants_native::{compile, tool};
use std::{fs, path::PathBuf, process::Command};

fn probe(producer: &JavaDependencyApi, mutation: usize) -> JavaDependencyApi {
    let first = producer.functions().next().unwrap().clone();
    let (scope, imported) = JavaDependencyScope::new().import(first).unwrap();
    let input = JavaExpr::local(double(), f::name("input"));
    let call = |value| JavaExpr {
        ty: double(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Dependency(imported.clone()),
            receiver: None,
            arguments: vec![value],
        },
    };
    let local = |name, value| JavaStmt::Local {
        finality: JavaLocalFinality::Final,
        ty: double(),
        name: f::name(name),
        value: Some(value),
    };
    let left = local(
        "left",
        if mutation == 1 {
            input.clone()
        } else {
            call(input)
        },
    );
    let zero = JavaExpr::literal(
        double(),
        JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(0).unwrap()),
    );
    let right = local("right", call(zero));
    let mut statements = if mutation == 2 {
        vec![right, left]
    } else {
        vec![left, right]
    };
    statements.push(JavaStmt::Return(Some(JavaExpr::local(
        double(),
        f::name("left"),
    ))));
    let declaration = f::Function {
        hash: 10,
        public: true,
        name: f::name("probe"),
        parameters: vec![JavaParameter {
            ty: double(),
            name: f::name("input"),
            final_parameter: true,
        }],
        result: double(),
        body: JavaBlock::new(statements),
    };
    JavaDependencyApi::from_certificate(f::certify(f::package_with_dependencies(
        95,
        vec![declaration],
        scope.finish(),
    )))
    .unwrap()
}
#[test]
fn double_operand_calls_are_once_and_ordered_with_value_preserving_controls() {
    let root = PathBuf::from(std::env::var_os("TEST_TMPDIR").unwrap()).join("java-double-trace");
    let producer =
        JavaDependencyApi::from_certificate(f::certify(f::package(94, functions(&[])))).unwrap();
    for mutation in 0..3 {
        let directory = root.join(mutation.to_string());
        let classes = directory.join("classes");
        fs::create_dir_all(&classes).unwrap();
        let consumer = probe(&producer, mutation);
        for owner in [&producer, &consumer] {
            let output =
                render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
            for file in output.files() {
                let OutputContents::Text(text) = file.contents() else {
                    panic!("source")
                };
                let mut text = text.clone();
                if text.contains("double identity(") {
                    let start = text.find("double identity(").unwrap();
                    let brace = start + text[start..].find('{').unwrap() + 1;
                    text.insert_str(
                        brace,
                        r#"
System.err.print(Long.toHexString(Double.doubleToRawLongBits(input)) + ",");
"#,
                    );
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
        let source = directory.join("Consumer.java");
        fs::write(&source, "public final class Consumer { public static void main(String[] args) { if (org.polyrust.generated.r000000000000005f.Generated.probe(1.5) != 1.5) throw new AssertionError(\"value\"); } }").unwrap();
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
        assert!(
            output.status.success(),
            "every mutant must preserve the value"
        );
        let trace = String::from_utf8(output.stderr).unwrap();
        assert_eq!(
            trace,
            ["3ff8000000000000,0,", "0,", "0,3ff8000000000000,"][mutation]
        );
        assert_eq!(trace == "3ff8000000000000,0,", mutation == 0);
    }
}
