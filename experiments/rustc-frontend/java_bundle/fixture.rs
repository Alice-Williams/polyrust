//! Small target-certified fixture, not a substitute for the rustc native oracle.
use portable_backend_java::{
    ast::*,
    dialect::{JavaDependencyApi, JavaDialect, JavaInvocationKind},
};
use portable_codegen::*;
use portable_diagnostics::SourceRef;
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn owner(crate_id: u64, documentation: &str) -> JavaDependencyApi {
    let root = RustDeclarationId {
        crate_id,
        definition_path_hash: 1,
    };
    let function = RustDeclarationId {
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
                    name: "score".into(),
                },
                RustExportTarget::Declaration(function),
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
    let ty = JavaType::primitive(JavaPrimitive::Int);
    let callable = builder.callable(GeneratedCallable {
        name: "score".into(),
        signature: TargetCallableSignature {
            invocation: JavaInvocationKind::Static,
            receiver: None,
            parameters: vec![],
            return_type: TargetTypeRef::Primitive(JavaPrimitive::Int),
        },
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
            declaration: function,
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
            JavaMember::Method(JavaMethod {
                declared: JavaMethodDeclaration::Callable(callable),
                annotations: vec![],
                modifiers: vec![JavaModifier::Public, JavaModifier::Static],
                type_parameters: vec![],
                return_type: ty.clone(),
                name: name("score"),
                parameters: vec![],
                body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(
                    JavaExpr::literal(ty, JavaLiteral::I32(42)),
                ))])),
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
                GeneratedSymbolId::Callable(callable),
            ],
            conformances: JavaConformanceInventory::structural().into(),
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
