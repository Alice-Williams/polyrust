//! Certificate-only alias fixture; compiler integration supplies independent Rust proof.
use portable_backend_java::{ast::*, dialect::*};
use portable_codegen::*;
use portable_diagnostics::SourceRef;
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn owner(
    crate_id: u64,
    values: &[JavaDependencyConstant],
    read: bool,
) -> JavaDependencyApi {
    let id = |hash| RustDeclarationId {
        crate_id,
        definition_path_hash: hash,
    };
    let name = |text| JavaIdentifier::new(text).unwrap();
    let export = |text: String| RustExportName {
        namespace: RustExportNamespace::Value,
        name: text,
    };
    let location = RustSourceLocation {
        file: "alias.rs".into(),
        line: 1,
        column: 1,
    };
    let root_doc = Arc::new(RustModuleDocumentation {
        declaration: id(1),
        parent: None,
        location: location.clone(),
        documentation: vec!["Alias façade 🦀.".into()],
    });
    let child_doc = Arc::new(RustModuleDocumentation {
        declaration: id(2),
        parent: Some(id(1)),
        location: location.clone(),
        documentation: vec!["Nested aliases.".into()],
    });
    let ancestry: RustModuleAncestry = vec![root_doc.clone()].into();
    let module_name = |text: &str| RustExportName {
        namespace: RustExportNamespace::Type,
        name: text.into(),
    };
    let mut graph = RustCrateExports {
        root: id(1),
        modules: BTreeMap::from([
            (
                id(1),
                BTreeMap::from([(module_name("nested"), RustExportTarget::Module(id(2)))]),
            ),
            (
                id(2),
                BTreeMap::from([(module_name("back"), RustExportTarget::Module(id(1)))]),
            ),
        ]),
        module_ancestries: BTreeMap::from([
            (id(1), ancestry.clone()),
            (id(2), vec![root_doc, child_doc].into()),
        ]),
    };
    let mut scope = JavaDependencyScope::new();
    let mut imported = vec![];
    for (index, value) in values.iter().enumerate() {
        for module in [id(1), id(2)] {
            graph.modules.get_mut(&module).unwrap().insert(
                export(format!("alias{index}")),
                RustExportTarget::Declaration(value.declaration()),
            );
        }
        let (next, value) = scope.import_constant(value.clone()).unwrap();
        scope = next;
        imported.push(value);
    }
    if read {
        graph
            .modules
            .get_mut(&id(1))
            .unwrap()
            .insert(export("read".into()), RustExportTarget::Declaration(id(10)));
    }
    let graph = Arc::new(graph);
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let source = || SourceRef::logical(["java-alias-bundle"]);
    let facade = builder.generated_type(GeneratedType {
        name: "Generated".into(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
        source: source(),
    });
    let mut declared = vec![GeneratedSymbolId::Type(facade)];
    let mut members = vec![JavaMember::Constructor(JavaConstructor {
        modifiers: vec![JavaModifier::Private],
        name: name("Generated"),
        parameters: vec![],
        body: JavaBlock::new(vec![]),
    })];
    if read {
        let value = imported[0].clone();
        let JavaType::Primitive(primitive) = value.ty() else {
            panic!("scalar fixture")
        };
        let signature = TargetCallableSignature {
            invocation: JavaInvocationKind::Static,
            receiver: None,
            parameters: vec![],
            return_type: TargetTypeRef::Primitive(*primitive),
        };
        let callable = builder.callable(GeneratedCallable {
            name: "read".into(),
            signature,
            visibility: JavaVisibility::Public,
            origin: GeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                declaration: id(10),
                node: RustSourceNode::Declaration,
                module: id(1),
                location,
                visibility: RustVisibility::Public,
                externally_reachable: true,
                documentation: vec!["Actual reader.".into()],
                module_ancestors: ancestry,
                crate_exports: graph.clone(),
            })),
            source: source(),
        });
        declared.push(GeneratedSymbolId::Callable(callable));
        members.push(JavaMember::Method(JavaMethod {
            declared: JavaMethodDeclaration::Callable(callable),
            annotations: vec![],
            modifiers: vec![JavaModifier::Public, JavaModifier::Static],
            type_parameters: vec![],
            return_type: value.ty().clone(),
            name: name("read"),
            parameters: vec![],
            body: Some(JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
                ty: value.ty().clone(),
                precedence: JavaPrecedence::Primary,
                kind: JavaExprKind::Value(JavaValueRef::Dependency(value)),
            }))])),
        }));
    }
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
            declared,
            conformances: JavaConformanceInventory::structural().into(),
            source_package: Some(JavaSourcePackage::new(graph)),
            dependencies: scope.finish(),
            declaration: Box::new(JavaTypeDeclaration {
                declared: Some(facade),
                kind: JavaDeclarationKind::FinalClass,
                visibility: JavaVisibility::Public,
                modifiers: vec![],
                name: name("Generated"),
                type_parameters: vec![],
                record_components: vec![],
                heritage: JavaHeritage::None,
                permits: vec![],
                members,
            }),
        }],
        JavaSourceFileKind::CompilationUnit,
        source(),
    ));
    builder.group(TargetFileGroup::new(
        FileGroupRole::PublicApi,
        vec![TargetFileMember::Source(file)],
        source(),
    ));
    let checked = verify_unresolved_package(&JavaDialect, builder.build()).unwrap();
    let linked = TargetLinker::new(JavaDialect).link_ast(&checked).unwrap();
    JavaDependencyApi::from_certificate(certify_resolved_package(&JavaDialect, linked).unwrap())
        .unwrap()
}
