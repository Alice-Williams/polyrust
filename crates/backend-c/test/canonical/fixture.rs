//! Synthetic target metadata, deliberately not a Rust compiler witness.
use portable_backend_c::ast::*;
use portable_backend_c::dialect::*;
use portable_codegen::*;

pub const PROFILE: CCanonicalTypeProfile = CCanonicalTypeProfile::ScalarResultV2;

pub fn facts(maximum: bool) -> RustCanonicalErrorKindFacts {
    let ids: [RustDeclarationId; 15] = std::array::from_fn(|index| RustDeclarationId {
        crate_id: if maximum { u64::MAX } else { 7 },
        definition_path_hash: if maximum {
            u64::MAX - index as u64
        } else {
            index as u64
        },
    });
    let instance = RustCanonicalInstanceFacts::new(
        RustCanonicalInstanceKey::i32_try_from_int_error_result(ids[1], ids[2]).unwrap(),
        ids[0],
        RustResultVariantFacts {
            variant: ids[3],
            payload: ids[5],
        },
        RustResultVariantFacts {
            variant: ids[4],
            payload: ids[6],
        },
    )
    .unwrap();
    RustCanonicalErrorKindFacts::new(
        instance,
        ids[7],
        ids[8],
        RustIntegerErrorVariants {
            empty: ids[9],
            invalid_digit: ids[10],
            positive_overflow: ids[11],
            negative_overflow: ids[12],
            zero: ids[13],
            not_a_power_of_two: ids[14],
        },
    )
    .unwrap()
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Fault {
    #[default]
    None,
    HeaderName,
    SourceName,
    RecordName,
    TagName,
    ValueName,
    TagType,
    ValueType,
    ReversedMembers,
    ExtraMember,
    ExtraFile,
    ExtraRecord,
    MissingDeclaration,
    DuplicateDeclaration,
    HeaderComment,
    SourceComment,
    DifferentFacts,
    NoProfile,
    ForgedCoreOrigin,
}

pub struct Fixture {
    pub registry: CRegistry,
    pub header: CFileRef,
    pub implementation: CFileRef,
    pub record: CStructRef,
    pub facts: RustCanonicalErrorKindFacts,
    pub files: Vec<CSourceFile>,
}

pub fn fixture(maximum: bool, fault: Fault) -> Fixture {
    let facts = facts(maximum);
    let base = PROFILE.basename(facts.instance().key());
    let key = |role| {
        let mut key = PROFILE
            .declaration_key(facts.instance().key(), role)
            .unwrap();
        if matches!(fault, Fault::ForgedCoreOrigin) {
            let root = facts.instance().core_root();
            key.origin = CGeneratedOrigin::RustSource(std::sync::Arc::new(RustSourceOrigin {
                declaration: facts.instance().key().result_definition(),
                node: RustSourceNode::Declaration,
                module: root,
                location: RustSourceLocation {
                    file: "core/result.rs".into(),
                    line: 1,
                    column: 0,
                },
                visibility: RustVisibility::Public,
                externally_reachable: true,
                documentation: vec![],
                module_ancestors: vec![].into(),
                crate_exports: std::sync::Arc::new(RustCrateExports {
                    root,
                    modules: std::collections::BTreeMap::new(),
                    module_ancestries: std::collections::BTreeMap::new(),
                }),
            }));
        }
        key
    };
    let wrong_key = |role| {
        let mut key = key(role);
        key.name = CIdentifier::new("different").unwrap();
        key
    };
    let mut registry = CRegistry::new();
    let mut file = |extension, role, wrong| {
        registry
            .register_file(CFileKey {
                path: RelativeOutputPath::new(format!(
                    "{}{extension}",
                    if wrong { "wrong" } else { &base }
                ))
                .unwrap(),
                role,
            })
            .unwrap()
    };
    let header = file(
        ".h",
        CFileRole::GeneratedPublicHeader,
        matches!(fault, Fault::HeaderName),
    );
    let implementation = file(
        ".c",
        CFileRole::GeneratedSource,
        matches!(fault, Fault::SourceName),
    );
    let record = registry
        .declare_struct(
            &header,
            if matches!(fault, Fault::RecordName) {
                wrong_key(CCanonicalTypeRole::Result)
            } else {
                key(CCanonicalTypeRole::Result)
            },
        )
        .unwrap();
    let owner = CAggregateRef::Struct(record.clone());
    let tag = registry
        .register_member(
            &owner,
            if matches!(fault, Fault::TagName) {
                wrong_key(CCanonicalTypeRole::SuccessTag)
            } else {
                key(CCanonicalTypeRole::SuccessTag)
            },
            CObjectType::scalar(if matches!(fault, Fault::TagType) {
                CScalarType::I32
            } else {
                CScalarType::Bool
            }),
        )
        .unwrap();
    let value = registry
        .register_member(
            &owner,
            if matches!(fault, Fault::ValueName) {
                wrong_key(CCanonicalTypeRole::Value)
            } else {
                key(CCanonicalTypeRole::Value)
            },
            CObjectType::scalar(if matches!(fault, Fault::ValueType) {
                CScalarType::I64
            } else {
                CScalarType::I32
            }),
        )
        .unwrap();
    let mut members = vec![tag, value];
    if matches!(fault, Fault::ReversedMembers) {
        members.reverse();
    }
    if matches!(fault, Fault::ExtraMember) {
        members.push(
            registry
                .register_member(
                    &owner,
                    wrong_key(CCanonicalTypeRole::Value),
                    CObjectType::scalar(CScalarType::I32),
                )
                .unwrap(),
        );
    }
    registry.define_aggregate(&owner, members).unwrap();
    if matches!(fault, Fault::ExtraFile) {
        registry
            .register_file(CFileKey {
                path: RelativeOutputPath::new("unrelated.c").unwrap(),
                role: CFileRole::GeneratedSource,
            })
            .unwrap();
    }
    if matches!(fault, Fault::ExtraRecord) {
        registry
            .declare_struct(&header, wrong_key(CCanonicalTypeRole::Result))
            .unwrap();
    }
    let d = CDeclarations::new(&registry, header.clone()).unwrap();
    let mut items = vec![CFileItem::Declaration(d.aggregate(owner).unwrap())];
    if matches!(fault, Fault::MissingDeclaration) {
        items.clear();
    }
    if matches!(fault, Fault::DuplicateDeclaration) {
        items.push(items[0].clone());
    }
    if matches!(fault, Fault::HeaderComment) {
        items.push(CFileItem::Comment(CComment::new("extra")));
    }
    let h = d.source_file(items).unwrap();
    let d = CDeclarations::new(&registry, implementation.clone()).unwrap();
    let items = if matches!(fault, Fault::SourceComment) {
        vec![CFileItem::Comment(CComment::new("extra"))]
    } else {
        vec![]
    };
    let s = d.source_file(items).unwrap();
    Fixture {
        registry,
        header,
        implementation,
        record,
        facts,
        files: vec![h, s],
    }
}

pub fn register(value: &mut Fixture, fault: Fault) -> Result<(), CRegistryError> {
    if matches!(fault, Fault::NoProfile) {
        return Ok(());
    }
    value.registry.register_canonical_type_package(
        &value.header,
        &value.implementation,
        PROFILE,
        if matches!(fault, Fault::DifferentFacts) {
            facts(true)
        } else {
            value.facts
        },
        &value.record,
    )
}

pub fn certify(value: Fixture) -> Result<RenderReadyPackage<CDialect>, String> {
    let ast =
        project_c_package(value.registry.freeze(), value.files).map_err(|e| format!("{e:?}"))?;
    let checked = verify_unresolved_package(&CDialect, ast).map_err(|e| format!("{e:?}"))?;
    let linked = TargetLinker::new(CDialect)
        .link_ast(&checked)
        .map_err(|e| format!("{e:?}"))?;
    certify_resolved_package(&CDialect, linked).map_err(|e| format!("{e:?}"))
}
