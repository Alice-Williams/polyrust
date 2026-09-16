//! Small target-certified fixture, not a substitute for the rustc native oracle.
use portable_backend_java::{
    ast::*,
    dialect::{JavaDependencyApi, JavaDialect},
};
use portable_codegen::*;
use portable_diagnostics::SourceRef;
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn owner(crate_id: u64, documentation: &str, literal: JavaLiteral) -> JavaDependencyApi {
    let root = RustDeclarationId {
        crate_id,
        definition_path_hash: 1,
    };
    let constant = RustDeclarationId {
        crate_id,
        definition_path_hash: 2,
    };
    let location = RustSourceLocation {
        file: "source.rs".into(),
        line: 1,
        column: 1,
    };
    let ancestry: RustModuleAncestry = vec![Arc::new(RustModuleDocumentation {
        declaration: root,
        parent: None,
        location: location.clone(),
        documentation: vec![documentation.into()],
    })]
    .into();
    let exports = Arc::new(RustCrateExports {
        root,
        modules: BTreeMap::from([(
            root,
            BTreeMap::from([(
                RustExportName {
                    namespace: RustExportNamespace::Value,
                    name: "VALUE".into(),
                },
                RustExportTarget::Declaration(constant),
            )]),
        )]),
        module_ancestries: BTreeMap::from([(root, ancestry.clone())]),
    });
    let source = || SourceRef::logical(["java-bundle-fixture"]);
    let name = |value| JavaIdentifier::new(value).unwrap();
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let facade = builder.generated_type(GeneratedType {
        name: "Generated".into(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
        source: source(),
    });
    let primitive = match literal {
        JavaLiteral::Boolean(_) => JavaPrimitive::Boolean,
        JavaLiteral::I32(_) => JavaPrimitive::Int,
        JavaLiteral::I64(_) => JavaPrimitive::Long,
        _ => panic!("scalar fixture"),
    };
    let ty = JavaType::primitive(primitive);
    let value = builder.value(GeneratedValue {
        name: "VALUE".into(),
        ty: TargetTypeRef::Primitive(primitive),
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
            declaration: constant,
            node: RustSourceNode::Declaration,
            module: root,
            location,
            visibility: RustVisibility::Public,
            externally_reachable: true,
            documentation: vec![documentation.into()],
            module_ancestors: ancestry,
            crate_exports: exports,
        })),
        source: source(),
    });
    let declaration = JavaTypeDeclaration {
        declared: Some(facade),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        modifiers: vec![],
        name: name("Generated"),
        type_parameters: vec![],
        record_components: vec![],
        heritage: JavaHeritage::None,
        permits: vec![],
        members: vec![
            JavaMember::Constructor(JavaConstructor {
                modifiers: vec![JavaModifier::Private],
                name: name("Generated"),
                parameters: vec![],
                body: JavaBlock::new(vec![]),
            }),
            JavaMember::Field(JavaField {
                declared: Some(value),
                modifiers: vec![
                    JavaModifier::Public,
                    JavaModifier::Static,
                    JavaModifier::Final,
                ],
                ty: ty.clone(),
                name: name("VALUE"),
                initializer: Some(JavaExpr::literal(ty, literal)),
            }),
        ],
    };
    let module = JavaPackage::RustCrate(crate_id);
    let file = builder.file(TargetFile::new(
        RelativeOutputPath::new(format!(
            "{}Generated.java",
            module.source_directory(JavaFilePlacement::Main)
        ))
        .unwrap(),
        SourceRole::PublicApi,
        module,
        JavaFilePlacement::Main,
        vec![JavaFileItem::Type {
            declared: vec![
                GeneratedSymbolId::Type(facade),
                GeneratedSymbolId::Value(value),
            ],
            conformances: JavaConformanceInventory::structural().into(),
            source_package: None,
            dependencies: Default::default(),
            declaration: Box::new(declaration),
        }],
        JavaSourceFileKind::CompilationUnit,
        source(),
    ));
    builder.group(TargetFileGroup::new(
        FileGroupRole::PublicApi,
        vec![TargetFileMember::Source(file)],
        source(),
    ));
    let verified = verify_unresolved_package(&JavaDialect, builder.build()).unwrap();
    let resolved = TargetLinker::new(JavaDialect).link_ast(&verified).unwrap();
    JavaDependencyApi::from_certificate(certify_resolved_package(&JavaDialect, resolved).unwrap())
        .unwrap()
}
