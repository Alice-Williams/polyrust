//! Canonical file graph from symbol references and typed file requirements.
use super::*;

pub(super) fn derive_and_validate_file_graph<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    files: &mut [RawFile<D>],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut declarations = BTreeMap::new();
    for (file_index, file) in package.files().enumerate() {
        let file_id = TargetFileId::from_index(file_index);
        for declaration in file
            .items()
            .iter()
            .flat_map(|item| dialect.file_item_roots(item).declarations)
        {
            if generated_symbol_source(package, declaration).is_none() {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnresolvedReference,
                    "file item declares a missing generated symbol",
                    file.source().clone(),
                ));
            } else if declarations.insert(declaration, file_id).is_some() {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::DuplicateDeclaration,
                    "generated symbol is declared in more than one source file",
                    file.source().clone(),
                ));
            }
        }
    }
    for symbol in (0..package.generated_types().len())
        .map(|index| GeneratedSymbolId::Type(GeneratedTypeId::from_index(index)))
        .chain(
            (0..package.callables().len())
                .map(|index| GeneratedSymbolId::Callable(GeneratedCallableId::from_index(index))),
        )
        .chain((0..package.interface_methods().len()).map(|index| {
            GeneratedSymbolId::InterfaceMethod(GeneratedInterfaceMethodId::from_index(index))
        }))
        .chain(
            (0..package.values().len())
                .map(|index| GeneratedSymbolId::Value(GeneratedValueId::from_index(index))),
        )
    {
        if !declarations.contains_key(&symbol) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::InvalidStructure,
                "generated symbol is not placed in a source file",
                generated_symbol_source(package, symbol)
                    .cloned()
                    .unwrap_or_else(|| SourceRef::logical(["target-linker", "file-graph"])),
            ));
        }
    }

    let requirements = super::file_requirements::derive(dialect, package, diagnostics);
    let mut graph = BTreeMap::new();
    for raw_file in files.iter_mut() {
        let from_role = package.file(raw_file.file).map(TargetFile::role);
        let mut edges = requirements
            .get(&raw_file.file)
            .cloned()
            .unwrap_or_default();
        for reference in &raw_file.references {
            let TargetSymbolRef::Generated(symbol) = reference.symbol else {
                continue;
            };
            let Some(destination) = declarations.get(&symbol).copied() else {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnresolvedReference,
                    "generated symbol reference has no structural source-file declaration",
                    reference.source.clone(),
                ));
                continue;
            };
            if destination == raw_file.file {
                continue;
            }
            edges.insert(destination);
            let to_role = package.file(destination).map(TargetFile::role);
            if violates_file_visibility(dialect, package, from_role, to_role, symbol) {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidStructure,
                    "cross-file reference violates source-role or public-API visibility",
                    reference.source.clone(),
                ));
            }
        }
        raw_file.dependencies = edges.iter().copied().collect();
        graph.insert(raw_file.file, edges);
    }

    if let Err(error) = super::file_cycle::check(&graph, |cycle| dialect.permits_file_cycle(cycle))
    {
        let (code, message) = match error {
            super::file_cycle::Error::Forbidden => (
                DiagnosticCode::InvalidStructure,
                "target source-file dependency graph contains a forbidden cycle",
            ),
            super::file_cycle::Error::Budget => (
                DiagnosticCode::TargetResourceLimit,
                "selective file-cycle policy traversal budget exceeded",
            ),
        };
        diagnostics.push(Diagnostic::error(
            code,
            message,
            SourceRef::logical(["target-linker", "file-cycle"]),
        ));
    }
}

fn generated_symbol_source<D: LinkerDialect>(
    package: &TargetAstPackage<D>,
    symbol: GeneratedSymbolId,
) -> Option<&SourceRef> {
    match symbol {
        GeneratedSymbolId::Type(id) => package.generated_type(id).map(|value| &value.source),
        GeneratedSymbolId::Callable(id) => package.callable(id).map(|value| &value.source),
        GeneratedSymbolId::InterfaceMethod(id) => {
            package.interface_method(id).map(|value| &value.source)
        }
        GeneratedSymbolId::Value(id) => package.value(id).map(|value| &value.source),
    }
}

pub(super) fn generated_symbol_is_public<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    symbol: GeneratedSymbolId,
) -> bool {
    match symbol {
        GeneratedSymbolId::Type(id) => package
            .generated_type(id)
            .is_some_and(|value| dialect.is_public(&value.visibility)),
        GeneratedSymbolId::Callable(id) => package
            .callable(id)
            .is_some_and(|value| dialect.is_public(&value.visibility)),
        GeneratedSymbolId::InterfaceMethod(id) => package
            .interface_method(id)
            .and_then(|value| package.generated_type(value.owner))
            .is_some_and(|owner| dialect.is_public(&owner.visibility)),
        GeneratedSymbolId::Value(id) => package
            .value(id)
            .is_some_and(|value| dialect.is_public(&value.visibility)),
    }
}

fn violates_file_visibility<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
    from: Option<SourceRole>,
    to: Option<SourceRole>,
    symbol: GeneratedSymbolId,
) -> bool {
    let (Some(from), Some(to)) = (from, to) else {
        return true;
    };
    violates_role(from, to)
        || (from == SourceRole::PublicApi && !generated_symbol_is_public(dialect, package, symbol))
}

pub(super) fn violates_role(from: SourceRole, to: SourceRole) -> bool {
    let is_test = |role| {
        matches!(
            role,
            SourceRole::NativeTest | SourceRole::Conformance | SourceRole::NegativeTest
        )
    };
    (from == SourceRole::Runtime && to != SourceRole::Runtime)
        || (!is_test(from) && is_test(to))
        || (from == SourceRole::PublicApi && to == SourceRole::Implementation)
}
