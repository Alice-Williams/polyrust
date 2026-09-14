//! Original owner metadata and independent normalized output expectations.
use portable_backend_c::ast::*;
use portable_codegen::{OutputContents, RenderedPackage, RustSourceOrigin};

fn origin(key: &CDeclarationKey) -> &RustSourceOrigin {
    let CGeneratedOrigin::RustSource(origin) = &key.origin else {
        panic!("compiler origin")
    };
    origin
}

pub(super) fn check(source: &CSourceFile) {
    let mut records = Vec::new();
    for item in source.items() {
        let CFileItem::Declaration(declaration) = item else {
            continue;
        };
        match declaration.kind() {
            CDeclarationKind::Aggregate {
                owner: CAggregateRef::Struct(record),
                members,
            } => {
                let metadata = origin(record.key());
                assert_eq!(metadata.module_ancestors.len(), 2);
                assert_eq!(metadata.module_ancestors[1].declaration, metadata.module);
                assert_eq!(
                    metadata.module_ancestors[1].parent,
                    Some(metadata.module_ancestors[0].declaration)
                );
                assert_eq!(metadata.module_ancestors[0].parent, None);
                assert_eq!(
                    metadata.module_ancestors[0].documentation,
                    ["CRATE_FIRST", "", "CRATE_LAST"]
                );
                assert_eq!(members.len(), 1);
                let field = origin(members[0].key());
                assert_eq!(field.module, metadata.module);
                assert_eq!(field.module_ancestors, metadata.module_ancestors);
                assert!(std::sync::Arc::ptr_eq(
                    &field.module_ancestors,
                    &metadata.module_ancestors
                ));
                assert_eq!(members[0].owner(), &CAggregateRef::Struct(record.clone()));
                records.push((metadata, field));
            }
            CDeclarationKind::FunctionPrototype { function, .. } => {
                let metadata = origin(function.key());
                assert_eq!(metadata.module_ancestors.len(), 1);
                assert_eq!(metadata.documentation[0], " SCORE_FIRST");
                assert_eq!(
                    metadata.documentation[1],
                    "SCORE_LINE_ONE\r\nSCORE_LINE_TWO"
                );
                assert_eq!(
                    metadata.documentation[2],
                    "HOSTILE /* inner */ ??/ \\\n#define NOT_CODE λ"
                );
            }
            _ => panic!("unadmitted declaration"),
        }
    }
    assert_eq!(records.len(), 2);
    assert_eq!(
        records[0].0.documentation,
        [" FIRST_RECORD", "INCLUDED_DOCUMENTATION\n"]
    );
    assert_eq!(records[1].0.documentation, [" SECOND_RECORD"]);
    assert_eq!(records[0].1.documentation, [" FIRST_FIELD"]);
    assert_eq!(records[1].1.documentation, [" SECOND_FIELD"]);
    assert_ne!(records[0].1.declaration, records[1].1.declaration);
    assert_ne!(records[0].0.module, records[1].0.module);
    assert!(std::sync::Arc::ptr_eq(
        &records[0].0.module_ancestors[0],
        &records[1].0.module_ancestors[0]
    ));
}

pub(super) fn rendered(output: &RenderedPackage) {
    let OutputContents::Text(text) = output.files()[0].contents() else {
        panic!("C text")
    };
    for label in [
        "CRATE_FIRST",
        "CRATE_LAST",
        "MODULE_FIRST",
        "MODULE_SECOND",
        "FIRST_RECORD",
        "SECOND_RECORD",
        "FIRST_FIELD",
        "SECOND_FIELD",
        "SCORE_FIRST",
        "INCLUDED_DOCUMENTATION",
        "HOSTILE",
    ] {
        assert_eq!(text.matches(label).count(), 1, "missing/duplicated {label}");
    }
    assert!(text.contains("/* CRATE_FIRST */\n/*  */\n/* CRATE_LAST */"));
    assert!(text.contains("/*  FIRST_FIELD */\n    int32_t poly_f0;"));
    assert!(text.contains("/*  SECOND_FIELD */\n    int32_t poly_f0_2;"));
    assert!(text.contains("/* HOSTILE / * inner * /"));
    assert!(text.contains("/*  SCORE_FIRST */\n/* SCORE_LINE_ONE\nSCORE_LINE_TWO */\n"));
    assert!(text.contains("[0x3F][0x3F]/ [0x5C]\n#define NOT_CODE [0xCE][0xBB] */"));
    assert!(!text.contains("ORDINARY_COMMENT_MUST_NOT_APPEAR"));
    let first_record = text.find("FIRST_RECORD").unwrap();
    let first_field = text.find("FIRST_FIELD").unwrap();
    let second_record = text.find("SECOND_RECORD").unwrap();
    let second_field = text.find("SECOND_FIELD").unwrap();
    assert!(
        first_record < first_field && first_field < second_record && second_record < second_field
    );
}
