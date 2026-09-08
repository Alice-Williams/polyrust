//! Force javac's runtime enum mapping, not its same-compilation ordinal shortcut.

use super::{parameter, structural_method, verifier_source};
use crate::ast::{
    JavaBlock, JavaConformanceInventory, JavaDeclarationKind, JavaExpr, JavaFileItem,
    JavaFilePlacement, JavaIdentifier, JavaLiteral, JavaMember, JavaPackage, JavaPattern,
    JavaPrimitive, JavaSourceFileKind, JavaStmt, JavaSwitchArm, JavaType, JavaTypeName,
    JavaVisibility,
};
use crate::dialect::JavaDialect;
use portable_codegen::{GeneratedTypeId, GeneratedValueId, TargetAstBuilder};
use std::path::Path;

const SOURCE: &str = "src/main/java/org/polyrust/generated/OracleEnumConsumer.java";

pub(super) fn add_consumer(
    builder: &mut TargetAstBuilder<JavaDialect>,
    enumeration: GeneratedTypeId,
    variant: GeneratedValueId,
    second: GeneratedValueId,
) {
    let ty = JavaType::Reference(JavaTypeName::Generated(enumeration));
    let int = JavaType::primitive(JavaPrimitive::Int);
    let mut declaration = super::fixture_declaration(vec![structural_method(
        "rank",
        int.clone(),
        vec![parameter(ty.clone(), "value")],
        JavaBlock::new(vec![JavaStmt::Switch {
            value: JavaExpr::local(ty, JavaIdentifier::from_portable("value")),
            arms: vec![
                JavaSwitchArm {
                    pattern: JavaPattern::EnumVariant {
                        enumeration,
                        variant,
                    },
                    body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
                        int.clone(),
                        JavaLiteral::I32(1),
                    )))]),
                },
                JavaSwitchArm {
                    pattern: JavaPattern::EnumVariant {
                        enumeration,
                        variant: second,
                    },
                    body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
                        int.clone(),
                        JavaLiteral::I32(2),
                    )))]),
                },
                JavaSwitchArm {
                    pattern: JavaPattern::Default,
                    body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
                        int,
                        JavaLiteral::I32(0),
                    )))]),
                },
            ],
        }]),
    )]);
    declaration.name = JavaIdentifier::from_portable("OracleEnumConsumer");
    declaration.kind = JavaDeclarationKind::FinalClass;
    declaration.visibility = JavaVisibility::Public;
    assert!(matches!(declaration.members[0], JavaMember::Method(_)));
    let mut items = vec![JavaFileItem::Type {
        conformances: JavaConformanceInventory::structural().into(),
        declared: vec![],
        declaration,
    }];
    for index in 1..=9 {
        let mut occupied = super::fixture_declaration(vec![]);
        occupied.name = JavaIdentifier::new(format!("OracleEnumConsumer${index}")).unwrap();
        items.push(JavaFileItem::Type {
            conformances: JavaConformanceInventory::structural().into(),
            declared: vec![],
            declaration: occupied,
        });
    }
    let file = builder.file(portable_codegen::TargetFile::new(
        portable_codegen::RelativeOutputPath::new(SOURCE).unwrap(),
        portable_codegen::SourceRole::PublicApi,
        JavaPackage::Generated,
        JavaFilePlacement::Main,
        items,
        JavaSourceFileKind::CompilationUnit,
        verifier_source("enum-consumer-file"),
    ));
    builder.group(portable_codegen::TargetFileGroup::new(
        portable_codegen::FileGroupRole::PublicApi,
        vec![portable_codegen::TargetFileMember::Source(file)],
        verifier_source("enum-consumer-group"),
    ));
}

pub(super) fn compile_separately(javac: &Path, root: &Path, classes: &Path) {
    let output = std::process::Command::new(javac)
        .args(["--release", "21", "-Werror", "-Xlint:all", "-cp"])
        .arg(classes)
        .arg("-d")
        .arg(classes)
        .arg(root.join(SOURCE))
        .output()
        .expect("compile enum consumer against classfiles only");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        classes
            .join("org/polyrust/generated/OracleEnumConsumer$10.class")
            .is_file(),
        "native oracle must actually emit the synthetic enum-switch helper",
    );
}
