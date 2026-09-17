use crate::ast::*;
use crate::dialect::{JavaDialect, JavaInvocationKind};
use crate::tests::source_record_fixture::{self as records, Facade, Fixture, id};
use portable_codegen::*;
use std::{collections::BTreeMap, sync::Arc};

pub const HOSTILE: &str =
    r"*/ public static int injected; /* \u002a\u002f @tag <script> café 日本語";

pub fn metadata() -> RustSourceOrigin {
    let mut origin = records::origin();
    let mut root = (*origin.module_ancestors[0]).clone();
    root.documentation = vec!["crate docs first".into(), HOSTILE.into()];
    let root = Arc::new(root);
    let private = Arc::new(RustModuleDocumentation {
        declaration: id(8),
        parent: Some(id(1)),
        location: root.location.clone(),
        documentation: vec!["private module documentation".into()],
    });
    origin.module = id(8);
    origin.module_ancestors = vec![root.clone(), private].into();
    origin.documentation = vec!["record first".into(), "record second".into()];
    origin.crate_exports = Arc::new(RustCrateExports {
        root: id(1),
        modules: BTreeMap::from([(
            id(1),
            BTreeMap::from([
                (
                    RustExportName {
                        namespace: RustExportNamespace::Value,
                        name: "inspect".into(),
                    },
                    RustExportTarget::Declaration(id(5)),
                ),
                (
                    RustExportName {
                        namespace: RustExportNamespace::Value,
                        name: "alias".into(),
                    },
                    RustExportTarget::Declaration(id(5)),
                ),
            ]),
        )]),
        module_ancestries: BTreeMap::from([(id(1), vec![root.clone()].into())]),
    });
    let alias_module = Arc::new(RustModuleDocumentation {
        declaration: id(9),
        parent: Some(id(1)),
        location: root.location.clone(),
        documentation: vec!["alias-only module documentation".into()],
    });
    let exports = Arc::make_mut(&mut origin.crate_exports);
    for alias in ["visible_module", "second_alias"] {
        exports.modules.get_mut(&id(1)).unwrap().insert(
            RustExportName {
                namespace: RustExportNamespace::Type,
                name: alias.into(),
            },
            RustExportTarget::Module(id(9)),
        );
    }
    exports.modules.insert(
        id(9),
        BTreeMap::from([(
            RustExportName {
                namespace: RustExportNamespace::Type,
                name: "back_to_root".into(),
            },
            RustExportTarget::Module(id(1)),
        )]),
    );
    exports
        .module_ancestries
        .insert(id(9), vec![root, alias_module].into());
    origin
}

pub fn fixture(facade: Facade) -> Fixture {
    scalar_fixture(facade, false)
}
pub fn binary64_fixture(facade: Facade) -> Fixture {
    scalar_fixture(facade, true)
}
fn scalar_fixture(facade: Facade, wide: bool) -> Fixture {
    let metadata = metadata();
    let scalar = if wide {
        JavaPrimitive::Double
    } else {
        JavaPrimitive::Int
    };
    let mut fixture = if wide {
        Fixture::with_double_metadata(metadata.clone(), facade)
    } else {
        Fixture::with_metadata(metadata.clone(), facade)
    };
    for (index, component) in fixture.record.record_components.iter_mut().enumerate() {
        let JavaRecordComponentOrigin::RustSource(field) = &mut component.origin else {
            unreachable!()
        };
        Arc::make_mut(&mut field.origin).documentation = vec![
            if index == 0 {
                "value field"
            } else {
                "flag field"
            }
            .into(),
            HOSTILE.into(),
        ];
    }
    let mut origin = metadata;
    origin.declaration = id(5);
    origin.documentation = vec!["inspect first".into(), "inspect second".into()];
    origin.visibility = RustVisibility::Public;
    origin.externally_reachable = true;
    let callable = fixture.builder.callable(GeneratedCallable {
        name: "inspect".into(),
        signature: TargetCallableSignature {
            invocation: JavaInvocationKind::Static,
            receiver: None,
            parameters: vec![
                TargetTypeRef::Primitive(scalar),
                TargetTypeRef::Primitive(JavaPrimitive::Boolean),
            ],
            return_type: TargetTypeRef::Primitive(scalar),
        },
        visibility: JavaVisibility::Public,
        origin: GeneratedOrigin::RustSource(Arc::new(origin)),
        source: records::source(),
    });
    let JavaMember::Method(method) = &mut fixture.facade.members[0] else {
        unreachable!()
    };
    method.declared = JavaMethodDeclaration::Callable(callable);
    fixture
}

pub fn package() -> TargetAstPackage<JavaDialect> {
    fixture(Facade::EntryPoint).finish()
}

pub fn text(package: TargetAstPackage<JavaDialect>) -> String {
    let rendered = records::certify(package);
    assert_eq!(rendered.files().len(), 1);
    let OutputContents::Text(text) = rendered.files()[0].contents() else {
        panic!("Java text")
    };
    text.clone()
}
