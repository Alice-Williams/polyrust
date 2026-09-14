//! A large module doc must have one shared payload, not one copy per field.
use portable_backend_c::ast::*;
use portable_codegen::{OutputContents, RenderedPackage};
use std::sync::Arc;

pub(super) fn check(source: &CSourceFile) {
    let mut keys = Vec::new();
    for item in source.items() {
        let CFileItem::Declaration(declaration) = item else {
            continue;
        };
        match declaration.kind() {
            CDeclarationKind::Aggregate { owner, members } => {
                assert_eq!(members.len(), 256);
                keys.push(owner.key());
                keys.extend(members.iter().map(CMemberRef::key));
            }
            CDeclarationKind::FunctionPrototype { function, .. } => keys.push(function.key()),
            _ => panic!("unexpected declaration"),
        }
    }
    assert_eq!(keys.len(), 258);
    let CGeneratedOrigin::RustSource(first) = &keys[0].origin else {
        panic!("source")
    };
    assert_eq!(first.module_ancestors.len(), 1);
    assert_eq!(first.module_ancestors[0].documentation.len(), 1);
    assert_eq!(
        first.module_ancestors[0].documentation[0].len(),
        1024 * 1024
    );
    for key in keys {
        let CGeneratedOrigin::RustSource(origin) = &key.origin else {
            panic!("source")
        };
        assert!(
            Arc::ptr_eq(&first.module_ancestors, &origin.module_ancestors),
            "module ancestry was copied"
        );
        assert!(
            Arc::ptr_eq(&first.module_ancestors[0], &origin.module_ancestors[0]),
            "module text was copied"
        );
        assert!(origin.documentation.is_empty());
    }
}

pub(super) fn rendered(output: &RenderedPackage) {
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("C text")
    };
    assert_eq!(text.matches("/* ").count(), 1);
    let start = text.find("/* ").unwrap() + 3;
    assert!(
        text[start..start + 1024 * 1024]
            .bytes()
            .all(|byte| byte == b'x')
    );
    assert_eq!(&text[start + 1024 * 1024..start + 1024 * 1024 + 3], " */");
}
