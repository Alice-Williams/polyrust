// Real shared linking of file-only references and independent mutation proofs.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Mode {
    Normal,
    NoDirective,
    Duplicate,
    SelfReference,
    WrongOwner,
    MissingPath,
    Cycle,
    AtLimit,
    OverLimit,
    SplitOverLimit,
    PermittedCycle,
    SelectiveCycles,
    AllCycles,
}

pub(super) fn requirements(
    mode: CatalogueMode,
    file: &TargetFile<TestDialect>,
) -> Vec<TargetFileRequirement<TestDialect>> {
    let CatalogueMode::RequiredFiles(mode) = mode else {
        return vec![];
    };
    if matches!(mode, Mode::SelectiveCycles | Mode::AllCycles) {
        let target = match file.path().as_str() {
            "0.test" => "1.test",
            "1.test" => "0.test",
            "2.test" => "3.test",
            "3.test" => "2.test",
            _ => return vec![],
        };
        return vec![TargetFileRequirement::new(
            Module::Generated,
            RelativeOutputPath::new(target).unwrap(),
        )];
    }
    let (left, right) = if file.path().as_str().starts_with("src/") {
        ("src/a.test", "src/b.test")
    } else {
        ("left.test", "right.test")
    };
    if file.path().as_str() == right {
        if mode == Mode::SplitOverLimit {
            return vec![
                TargetFileRequirement::new(
                    Module::Generated,
                    RelativeOutputPath::new(left).unwrap()
                );
                50_001
            ];
        }
        return if matches!(mode, Mode::Cycle | Mode::PermittedCycle) {
            vec![TargetFileRequirement::new(
                Module::Generated,
                RelativeOutputPath::new(left).unwrap(),
            )]
        } else {
            vec![]
        };
    }
    if file.path().as_str() != left {
        return vec![];
    }
    let module = if mode == Mode::WrongOwner {
        Module::Runtime
    } else {
        Module::Generated
    };
    let path = match mode {
        Mode::SelfReference => left,
        Mode::MissingPath => "missing.test",
        _ => right,
    };
    let requirement = TargetFileRequirement::new(module, RelativeOutputPath::new(path).unwrap());
    vec![
        requirement;
        match mode {
            Mode::Duplicate => 3,
            Mode::AtLimit => 100_000,
            Mode::OverLimit => 100_001,
            Mode::SplitOverLimit => 50_000,
            _ => 1,
        }
    ]
}

fn empty(mode: Mode, left: SourceRole, right: SourceRole) -> TargetAstPackage<TestDialect> {
    let mut builder = TargetAstBuilder::new(TestDialect(CatalogueMode::RequiredFiles(mode)));
    let mut groups = BTreeMap::<FileGroupRole, Vec<TargetFileMember>>::new();
    for (path, role) in [("left.test", left), ("right.test", right)] {
        let file = builder.file(TargetFile::new(
            RelativeOutputPath::new(path).unwrap(),
            role,
            Module::Generated,
            Placement::Implementation,
            vec![],
            Template::Source,
            source(path),
        ));
        groups
            .entry(group_role(role))
            .or_default()
            .push(TargetFileMember::Source(file));
    }
    for (role, members) in groups {
        builder.group(TargetFileGroup::new(role, members, source("group")));
    }
    builder.build()
}

fn group_role(role: SourceRole) -> FileGroupRole {
    match role {
        SourceRole::PublicApi => FileGroupRole::PublicApi,
        SourceRole::Implementation => FileGroupRole::Implementation,
        SourceRole::Runtime => FileGroupRole::Runtime,
        SourceRole::NativeTest => FileGroupRole::NativeTests,
        SourceRole::Conformance => FileGroupRole::Conformance,
        SourceRole::NegativeTest => FileGroupRole::NegativeTests,
    }
}

fn linked(mode: Mode) -> LinkedTargetPackage<TestDialect> {
    let raw = empty(mode, SourceRole::Implementation, SourceRole::PublicApi);
    TargetLinker::new(raw.dialect().clone())
        .link_ast(&verified(raw))
        .unwrap()
}

#[test]
fn zero_symbol_files_have_one_authenticated_edge_without_a_name_binding() {
    for mode in [Mode::Normal, Mode::Duplicate, Mode::NoDirective] {
        let package = linked(mode);
        assert!(package.bindings.is_empty());
        assert!(
            package
                .files
                .iter()
                .all(|file| file.references.is_empty() && file.imports.is_empty())
        );
        assert_eq!(package.files[0].dependencies, [package.files[1].file]);
        assert!(package.files[1].dependencies.is_empty());
        assert_eq!(
            package.files[0].file_imports.len(),
            usize::from(mode != Mode::NoDirective)
        );
        verify_linked_package(&package).unwrap();
        assert_eq!(package, linked(mode));
    }
}

#[test]
fn wrong_owners_missing_paths_self_edges_roles_and_cycles_fail() {
    for mode in [
        Mode::WrongOwner,
        Mode::MissingPath,
        Mode::SelfReference,
        Mode::Cycle,
    ] {
        let raw = empty(mode, SourceRole::Implementation, SourceRole::Implementation);
        assert!(
            TargetLinker::new(raw.dialect().clone())
                .link_ast(&verified(raw))
                .is_err(),
            "{mode:?}"
        );
    }
    for (left, right) in [
        (SourceRole::PublicApi, SourceRole::Implementation),
        (SourceRole::Runtime, SourceRole::PublicApi),
        (SourceRole::Implementation, SourceRole::NativeTest),
    ] {
        let raw = empty(Mode::Normal, left, right);
        assert!(
            TargetLinker::new(raw.dialect().clone())
                .link_ast(&verified(raw))
                .is_err()
        );
    }
}

#[test]
fn file_requirement_budget_counts_duplicate_requests_before_deduplication() {
    let package = linked(Mode::AtLimit);
    assert_eq!(package.files[0].dependencies.len(), 1);
    let raw = empty(
        Mode::OverLimit,
        SourceRole::Implementation,
        SourceRole::PublicApi,
    );
    let error = TargetLinker::new(raw.dialect().clone())
        .link_ast(&verified(raw))
        .unwrap_err();
    assert!(
        error
            .iter()
            .any(|error| error.code == DiagnosticCode::TargetResourceLimit)
    );
}

#[derive(Clone, Copy, Debug)]
enum Mutation {
    MissingEdge,
    MissingImport,
    BothMissing,
    ExtraEdge,
    ExtraImport,
    Retargeted,
    WrongKind,
}

#[test]
fn coordinated_edge_and_import_mutations_cannot_erase_file_only_requirements() {
    let original = linked(Mode::Normal);
    for mutation in [
        Mutation::MissingEdge,
        Mutation::MissingImport,
        Mutation::BothMissing,
        Mutation::ExtraEdge,
        Mutation::ExtraImport,
        Mutation::Retargeted,
        Mutation::WrongKind,
    ] {
        let mut changed = original.clone();
        let right = changed.files[1].file;
        let left = changed.files[0].file;
        match mutation {
            Mutation::MissingEdge => changed.files[0].dependencies.clear(),
            Mutation::MissingImport => changed.files[0].file_imports.clear(),
            Mutation::BothMissing => {
                changed.files[0].dependencies.clear();
                changed.files[0].file_imports.clear();
            }
            Mutation::ExtraEdge => changed.files[1].dependencies.push(left),
            Mutation::ExtraImport => {
                changed.files[1].file_imports = changed.files[0].file_imports.clone()
            }
            Mutation::Retargeted => {
                changed.files[0].dependencies = vec![left];
                changed.files[0].file_imports[0].destination = left;
            }
            Mutation::WrongKind => {
                changed.files[0].file_imports[0].kind =
                    ImportKind::File(RelativeOutputPath::new("wrong.test").unwrap())
            }
        }
        assert_eq!(original.files[0].file_imports[0].destination, right);
        assert!(verify_linked_package(&changed).is_err(), "{mutation:?}");
    }
}

#[test]
fn explicit_and_symbol_edges_are_deduplicated_without_bypassing_symbol_visibility() {
    for visibility in [Visibility::Public, Visibility::Private] {
        let raw = file_graph_package(
            CatalogueMode::RequiredFiles(Mode::Normal),
            SourceRole::PublicApi,
            SourceRole::PublicApi,
            visibility.clone(),
            false,
        );
        let result = TargetLinker::new(raw.dialect().clone()).link_ast(&verified(raw));
        if visibility == Visibility::Private {
            assert!(result.is_err());
        } else {
            let package = result.unwrap();
            assert_eq!(package.files[0].dependencies, [package.files[1].file]);
            assert_eq!(package.files[0].file_imports.len(), 1);
            verify_linked_package(&package).unwrap();
        }
    }
}

#[test]
fn budget_is_package_wide_and_duplicate_paths_are_ambiguous() {
    let raw = empty(
        Mode::SplitOverLimit,
        SourceRole::Implementation,
        SourceRole::Implementation,
    );
    let error = TargetLinker::new(raw.dialect().clone())
        .link_ast(&verified(raw))
        .unwrap_err();
    assert!(
        error
            .iter()
            .any(|error| error.code == DiagnosticCode::TargetResourceLimit)
    );
    let raw = empty(
        Mode::Normal,
        SourceRole::Implementation,
        SourceRole::PublicApi,
    );
    let mut builder = TargetAstBuilder::new(raw.dialect().clone());
    for file in raw.files().chain(raw.files().skip(1)) {
        builder.file(file.clone());
    }
    let ambiguous = builder.build();
    let mut diagnostics = vec![];
    super::super::file_requirements::derive(ambiguous.dialect(), &ambiguous, &mut diagnostics);
    assert!(
        diagnostics
            .iter()
            .any(|error| error.message.contains("ambiguous"))
    );
}

#[test]
fn dialect_may_explicitly_permit_a_file_cycle() {
    let raw = empty(
        Mode::PermittedCycle,
        SourceRole::Implementation,
        SourceRole::Implementation,
    );
    let package = TargetLinker::new(raw.dialect().clone())
        .link_ast(&verified(raw))
        .unwrap();
    assert_eq!(package.files[0].dependencies, [package.files[1].file]);
    assert_eq!(package.files[1].dependencies, [package.files[0].file]);
    verify_linked_package(&package).unwrap();
}

pub(super) fn permits(mode: CatalogueMode, cycle: &[TargetFileId]) -> bool {
    match mode {
        CatalogueMode::PermittedFileCycle
        | CatalogueMode::RequiredFiles(Mode::PermittedCycle | Mode::AllCycles) => true,
        CatalogueMode::RequiredFiles(Mode::SelectiveCycles) => {
            cycle == [TargetFileId::from_index(0), TargetFileId::from_index(1)]
        }
        _ => false,
    }
}

#[test]
fn selective_policy_cannot_hide_a_second_cycle_during_linking_or_reconstruction() {
    let raw = |mode| {
        let mut builder = TargetAstBuilder::new(TestDialect(CatalogueMode::RequiredFiles(mode)));
        let mut members = vec![];
        for name in ["0.test", "1.test", "2.test", "3.test"] {
            let file = builder.file(TargetFile::new(
                RelativeOutputPath::new(name).unwrap(),
                SourceRole::Implementation,
                Module::Generated,
                Placement::Implementation,
                vec![],
                Template::Source,
                source(name),
            ));
            members.push(TargetFileMember::Source(file));
        }
        builder.group(TargetFileGroup::new(
            FileGroupRole::Implementation,
            members,
            source("group"),
        ));
        builder.build()
    };
    let allowed = raw(Mode::AllCycles);
    let mut linked = TargetLinker::new(allowed.dialect().clone())
        .link_ast(&verified(allowed))
        .unwrap();
    verify_linked_package(&linked).unwrap();
    let forbidden = raw(Mode::SelectiveCycles);
    assert!(
        TargetLinker::new(forbidden.dialect().clone())
            .link_ast(&verified(forbidden.clone()))
            .is_err()
    );
    linked.dialect = forbidden.dialect().clone();
    linked.unresolved = forbidden;
    let errors = verify_linked_package(&linked).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("forbidden cycle"))
    );
}
