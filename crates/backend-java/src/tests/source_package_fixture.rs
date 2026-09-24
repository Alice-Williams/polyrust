//! Focused construction for explicit package metadata, with no invented source item.
use crate::{ast::*, dialect::JavaDialect};
use portable_codegen::*;
use portable_diagnostics::SourceRef;
use std::{collections::BTreeMap, sync::Arc};

pub fn graph() -> Arc<RustCrateExports> {
    let id = |hash| RustDeclarationId {
        crate_id: 7,
        definition_path_hash: hash,
    };
    let root = Arc::new(RustModuleDocumentation {
        declaration: id(1),
        parent: None,
        location: RustSourceLocation {
            file: "lib.rs".into(),
            line: 1,
            column: 1,
        },
        documentation: vec!["Empty facade root documentation.".into()],
    });
    let child = Arc::new(RustModuleDocumentation {
        declaration: id(2),
        parent: Some(id(1)),
        location: root.location.clone(),
        documentation: vec!["Nested alias module documentation.".into()],
    });
    let alias = |text: &str| RustExportName {
        namespace: RustExportNamespace::Type,
        name: text.into(),
    };
    Arc::new(RustCrateExports {
        root: id(1),
        modules: BTreeMap::from([
            (
                id(1),
                BTreeMap::from([(alias("nested"), RustExportTarget::Module(id(2)))]),
            ),
            (
                id(2),
                BTreeMap::from([(alias("back"), RustExportTarget::Module(id(1)))]),
            ),
        ]),
        module_ancestries: BTreeMap::from([
            (id(1), vec![root.clone()].into()),
            (id(2), vec![root, child].into()),
        ]),
    })
}

fn source() -> SourceRef {
    SourceRef::logical(["java-source-package"])
}
fn name(text: &str) -> JavaIdentifier {
    JavaIdentifier::new(text).unwrap()
}

pub fn empty() -> TargetAstPackage<JavaDialect> {
    empty_with_origin(SynthesisReason::PackageEntryPoint)
}

pub fn empty_with_origin(reason: SynthesisReason) -> TargetAstPackage<JavaDialect> {
    let mut builder = TargetAstBuilder::new(JavaDialect);
    let id = builder.generated_type(GeneratedType {
        name: "Generated".into(),
        kind: JavaDeclarationKind::FinalClass,
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::Synthesized(reason),
        source: source(),
    });
    let module = JavaPackage::RustCrate(7);
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
            declared: vec![GeneratedSymbolId::Type(id)],
            conformances: JavaConformanceInventory::structural().into(),
            package_metadata: Some(JavaSourcePackage::new(graph()).into()),
            dependencies: Default::default(),
            declaration: Box::new(JavaTypeDeclaration {
                declared: Some(id),
                kind: JavaDeclarationKind::FinalClass,
                visibility: JavaVisibility::Public,
                modifiers: vec![],
                name: name("Generated"),
                type_parameters: vec![],
                record_components: vec![],
                heritage: JavaHeritage::None,
                permits: vec![],
                members: vec![JavaMember::Constructor(JavaConstructor {
                    modifiers: vec![JavaModifier::Private],
                    name: name("Generated"),
                    parameters: vec![],
                    body: JavaBlock::new(vec![]),
                })],
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
    builder.build()
}

pub struct File {
    pub path: RelativeOutputPath,
    pub role: SourceRole,
    pub module: JavaPackage,
    pub placement: JavaFilePlacement,
    pub items: Vec<JavaFileItem>,
}

pub fn rebuild(
    package: TargetAstPackage<JavaDialect>,
    mut edit: impl FnMut(&mut File),
) -> TargetAstPackage<JavaDialect> {
    let mut builder = TargetAstBuilder::new(JavaDialect);
    for value in package.generated_types() {
        builder.generated_type(value.clone());
    }
    for value in package.callables() {
        builder.callable(value.clone());
    }
    for value in package.interface_methods() {
        builder.interface_method(value.clone());
    }
    for value in package.values() {
        builder.value(value.clone());
    }
    for original in package.files() {
        let mut file = File {
            path: original.path().clone(),
            role: original.role(),
            module: *original.module(),
            placement: *original.placement(),
            items: original.items().to_vec(),
        };
        edit(&mut file);
        builder.file(TargetFile::new(
            file.path,
            file.role,
            file.module,
            file.placement,
            file.items,
            *original.source_kind(),
            original.source().clone(),
        ));
    }
    for group in package.groups() {
        builder.group(group.clone());
    }
    builder.build()
}

pub fn attach(
    package: TargetAstPackage<JavaDialect>,
    exports: Arc<RustCrateExports>,
) -> TargetAstPackage<JavaDialect> {
    rebuild(package, |file| {
        let JavaFileItem::Type {
            package_metadata, ..
        } = &mut file.items[0]
        else {
            panic!("facade")
        };
        *package_metadata = Some(JavaSourcePackage::new(exports.clone()).into());
    })
}

pub fn exports(package: &TargetAstPackage<JavaDialect>) -> Arc<RustCrateExports> {
    package
        .generated_types()
        .map(|v| &v.origin)
        .chain(package.callables().map(|v| &v.origin))
        .chain(package.values().map(|v| &v.origin))
        .find_map(|origin| match origin {
            GeneratedOrigin::RustSource(origin) => Some(origin.crate_exports.clone()),
            _ => None,
        })
        .unwrap()
}

pub fn certify(package: TargetAstPackage<JavaDialect>) -> RenderReadyPackage<JavaDialect> {
    let checked = verify_unresolved_package(&JavaDialect, package).unwrap();
    let linked = TargetLinker::new(JavaDialect).link_ast(&checked).unwrap();
    certify_resolved_package(&JavaDialect, linked).unwrap()
}
