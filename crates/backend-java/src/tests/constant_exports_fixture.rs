//! Small source-façade builder; aliases never become declarations.
use super::source_dependency_fixture::{id, int, name};
use crate::{ast::*, dialect::*};
use portable_codegen::*;
use portable_diagnostics::SourceRef;
use std::{collections::BTreeMap, sync::Arc};

pub fn binding(text: &str) -> RustExportName {
    RustExportName {
        namespace: RustExportNamespace::Value,
        name: text.into(),
    }
}
pub fn imports(values: impl IntoIterator<Item = JavaDependencyConstant>) -> JavaDependencyBindings {
    values
        .into_iter()
        .fold(JavaDependencyScope::new(), |scope, value| {
            scope.import_constant(value).unwrap().0
        })
        .finish()
}
pub struct Fixture {
    pub graph: RustCrateExports,
    pub dependencies: JavaDependencyBindings,
    pub own: bool,
}
impl Fixture {
    pub fn new(crate_id: u64, values: &[JavaDependencyConstant], own: bool) -> Self {
        let root = id(crate_id, 1);
        let child = id(crate_id, 2);
        let location = RustSourceLocation {
            file: "lib.rs".into(),
            line: 1,
            column: 1,
        };
        let root_doc = Arc::new(RustModuleDocumentation {
            declaration: root,
            parent: None,
            location: location.clone(),
            documentation: vec!["Alias façade — 文.".into()],
        });
        let child_doc = Arc::new(RustModuleDocumentation {
            declaration: child,
            parent: Some(root),
            location,
            documentation: vec!["Nested exports.".into()],
        });
        let module_name = |text: &str| RustExportName {
            namespace: RustExportNamespace::Type,
            name: text.into(),
        };
        let mut graph = RustCrateExports {
            root,
            modules: BTreeMap::from([
                (
                    root,
                    BTreeMap::from([(module_name("nested"), RustExportTarget::Module(child))]),
                ),
                (
                    child,
                    BTreeMap::from([(module_name("back"), RustExportTarget::Module(root))]),
                ),
            ]),
            module_ancestries: BTreeMap::from([
                (root, vec![root_doc.clone()].into()),
                (child, vec![root_doc, child_doc].into()),
            ]),
        };
        for (i, value) in values.iter().enumerate() {
            for module in [root, child] {
                graph.modules.get_mut(&module).unwrap().insert(
                    binding(&format!("renamed{i}")),
                    RustExportTarget::Declaration(value.declaration()),
                );
            }
        }
        if own {
            graph.modules.get_mut(&root).unwrap().insert(
                binding("owned"),
                RustExportTarget::Declaration(id(crate_id, 10)),
            );
        }
        Self {
            graph,
            dependencies: imports(values.iter().cloned()),
            own,
        }
    }
    pub fn finish(self) -> TargetAstPackage<JavaDialect> {
        let graph = Arc::new(self.graph);
        let root = graph.root;
        let source = || SourceRef::logical(["constant-exports"]);
        let mut builder = TargetAstBuilder::new(JavaDialect);
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
        if self.own {
            let ancestry = graph.module_ancestries[&root].clone();
            let value = builder.value(GeneratedValue {
                name: "owned".into(),
                ty: JavaDialect.registered_type(&int()),
                visibility: JavaVisibility::Public,
                origin: GeneratedOrigin::RustSource(Arc::new(RustSourceOrigin {
                    declaration: id(root.crate_id, 10),
                    node: RustSourceNode::Declaration,
                    module: root,
                    location: ancestry[0].location.clone(),
                    visibility: RustVisibility::Public,
                    externally_reachable: true,
                    documentation: vec!["Owned constant.".into()],
                    module_ancestors: ancestry,
                    crate_exports: graph.clone(),
                })),
                source: source(),
            });
            declared.push(GeneratedSymbolId::Value(value));
            members.push(JavaMember::Field(JavaField {
                declared: Some(value),
                modifiers: vec![
                    JavaModifier::Public,
                    JavaModifier::Static,
                    JavaModifier::Final,
                ],
                ty: int(),
                name: name("owned"),
                initializer: Some(JavaExpr::literal(int(), JavaLiteral::I32(42))),
            }));
        }
        let module = JavaPackage::RustCrate(root.crate_id);
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
                package_metadata: Some(JavaSourcePackage::new(graph).into()),
                dependencies: self.dependencies,
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
        builder.build()
    }
}
