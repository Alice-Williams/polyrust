//! Qualifier admission applies to flow bindings and structural nominals too.

use super::{
    JavaBlock, JavaDialect, JavaExpr, JavaExprKind, JavaIdentifier, JavaKnownType, JavaLiteral,
    JavaMember, JavaPrecedence, JavaPrimitive, JavaStmt, JavaType, JavaUnaryOperator,
    JavaVisibility, fixture_declaration, instanceof, parameter, structural_method, verify_fixture,
};

#[test]
fn structural_top_level_names_are_unique_across_the_package() {
    for separate_files in [false, true] {
        for duplicate in [false, true] {
            let mut builder = portable_codegen::TargetAstBuilder::new(JavaDialect);
            let mut declarations = Vec::new();
            for index in 0..2 {
                let mut declaration = fixture_declaration(vec![]);
                declaration.name = JavaIdentifier::from_portable(if index == 0 || duplicate {
                    "First"
                } else {
                    "Second"
                });
                declarations.push(super::JavaFileItem::Type {
                    declared: vec![],
                    conformances: crate::ast::JavaConformanceInventory::structural().into(),
                    declaration,
                });
            }
            let units = if separate_files {
                declarations.into_iter().map(|item| vec![item]).collect()
            } else {
                vec![declarations]
            };
            let mut files = Vec::new();
            for (index, items) in units.into_iter().enumerate() {
                files.push(portable_codegen::TargetFileMember::Source(
                    builder.file(portable_codegen::TargetFile::new(
                        portable_codegen::RelativeOutputPath::new(format!(
                            "src/main/java/org/polyrust/generated/Unit{index}.java",
                        ))
                        .unwrap(),
                        portable_codegen::SourceRole::PublicApi,
                        super::JavaPackage::Generated,
                        super::JavaFilePlacement::Main,
                        items,
                        super::JavaSourceFileKind::CompilationUnit,
                        super::verifier_source("package-unique"),
                    )),
                ));
            }
            builder.group(portable_codegen::TargetFileGroup::new(
                portable_codegen::FileGroupRole::PublicApi,
                files,
                super::verifier_source("package-unique"),
            ));
            let result = portable_codegen::verify_target_ast(&builder.build());
            assert_eq!(
                result.is_ok(),
                !duplicate,
                "separate={separate_files}: {result:?}"
            );
        }
    }
}

#[test]
fn positive_and_negative_guard_patterns_reject_qualifier_shadowing() {
    for negative in [false, true] {
        for name in ["matched", "org", "java", "Objects", "Fixture"] {
            let string = JavaType::known(JavaKnownType::String);
            let object = JavaType::known(JavaKnownType::Object);
            let boolean = JavaType::primitive(JavaPrimitive::Boolean);
            let pattern = instanceof(
                JavaExpr::local(object.clone(), JavaIdentifier::from_portable("input")),
                string.clone(),
                name,
            );
            let use_binding = JavaStmt::Local {
                finality: super::JavaLocalFinality::Final,
                ty: string.clone(),
                name: JavaIdentifier::from_portable("retained"),
                value: Some(JavaExpr::local(string, JavaIdentifier::from_portable(name))),
            };
            let mut statements = if negative {
                vec![
                    JavaStmt::If {
                        condition: JavaExpr {
                            ty: boolean.clone(),
                            precedence: JavaPrecedence::Unary,
                            kind: JavaExprKind::Unary {
                                operator: JavaUnaryOperator::Not,
                                operand: Box::new(pattern),
                            },
                        },
                        then_block: JavaBlock::new(vec![JavaStmt::Return(Some(
                            JavaExpr::literal(boolean.clone(), JavaLiteral::Boolean(false)),
                        ))]),
                        else_block: None,
                    },
                    use_binding,
                ]
            } else {
                vec![JavaStmt::If {
                    condition: pattern,
                    then_block: JavaBlock::new(vec![use_binding]),
                    else_block: None,
                }]
            };
            statements.push(JavaStmt::Return(Some(JavaExpr::literal(
                boolean.clone(),
                JavaLiteral::Boolean(true),
            ))));
            let declaration = fixture_declaration(vec![structural_method(
                "matches",
                boolean,
                vec![parameter(object, "input")],
                JavaBlock::new(statements),
            )]);
            let result = verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], declaration)],
            );
            assert_eq!(
                result.is_ok(),
                name == "matched",
                "{name}, negative={negative}: {result:?}"
            );
        }
    }
}

#[test]
fn structural_nominals_cannot_shadow_runtime_roots_annotations_or_known_types() {
    for nested in [false, true] {
        for name in [
            "Ordinary",
            "org",
            "java",
            "Override",
            "SafeVarargs",
            "Objects",
            "Integer",
            "PolyResult",
            "Generated",
            "Runtime",
            "GeneratedTest",
            "ConformanceTest",
            "InvalidTypes",
        ] {
            let mut declaration = fixture_declaration(vec![]);
            declaration.name = JavaIdentifier::from_portable(name);
            if nested {
                declaration.visibility = JavaVisibility::Public;
                declaration = fixture_declaration(vec![JavaMember::NestedType(declaration)]);
            }
            let result = verify_fixture(
                portable_codegen::TargetAstBuilder::new(JavaDialect),
                vec![(vec![], declaration)],
            );
            assert_eq!(
                result.is_ok(),
                name == "Ordinary",
                "{name}, nested={nested}: {result:?}"
            );
        }
    }
}
