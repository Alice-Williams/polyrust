//! Synthetic compiler descriptions; this fixture grants no Rust admission.
use super::{local_fixture::Fixture, model::*};
use portable_backend_java::{ast::*, dialect::*};
use portable_codegen::*;

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

pub struct Owner {
    pub fixture: Fixture,
    pub metadata: JavaCanonicalTypePackage,
    pub namespace: JavaPackage,
    pub path: String,
}

impl Owner {
    pub fn new(maximum: bool) -> Self {
        Self::with_order(maximum, false)
    }
    pub fn with_order(maximum: bool, reverse: bool) -> Self {
        let fixture = Fixture::type_only_with_order(reverse);
        let facts = facts(maximum);
        let profile = JavaCanonicalTypeProfile::ScalarResultV2;
        let metadata = JavaCanonicalTypePackage::new(profile, facts, fixture.types, fixture.kinds);
        let namespace = JavaPackage::CanonicalInstance {
            instance: facts.instance().key(),
            profile,
        };
        let path = format!(
            "{}Generated.java",
            namespace.source_directory(JavaFilePlacement::Main)
        );
        Self {
            fixture,
            metadata,
            namespace,
            path,
        }
    }
    pub fn finish(self) -> TargetAstPackage<JavaDialect> {
        self.with_metadata(None)
    }
    pub fn with_metadata(
        self,
        replacement: Option<Option<JavaPackageMetadata>>,
    ) -> TargetAstPackage<JavaDialect> {
        let Self {
            mut fixture,
            metadata,
            namespace,
            path,
        } = self;
        fixture.facade.members.extend(
            [fixture.interface, fixture.success, fixture.error]
                .into_iter()
                .map(JavaMember::NestedType),
        );
        let mut declared = vec![];
        symbols(&fixture.facade, &mut declared);
        let file = fixture.builder.file(TargetFile::new(
            RelativeOutputPath::new(path).unwrap(),
            SourceRole::PublicApi,
            namespace,
            JavaFilePlacement::Main,
            vec![JavaFileItem::Type {
                declared,
                conformances: JavaConformanceInventory::structural().into(),
                package_metadata: replacement.unwrap_or_else(|| Some(metadata.into())),
                dependencies: Default::default(),
                declaration: fixture.facade.into(),
            }],
            JavaSourceFileKind::CompilationUnit,
            source(),
        ));
        fixture.builder.group(TargetFileGroup::new(
            FileGroupRole::PublicApi,
            vec![TargetFileMember::Source(file)],
            source(),
        ));
        fixture.builder.build()
    }
}
