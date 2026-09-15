//! Catalogue validation and reconstruction from original package authority.
use super::*;

pub(super) fn verify_dialect<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
) -> Result<(), Vec<Diagnostic>> {
    if dialect != package.dialect() {
        return Err(vec![link_error(
            DiagnosticCode::InterfaceNonconformance,
            "linker dialect differs from original package dialect",
            "catalogue",
        )]);
    }
    Ok(())
}

pub(super) fn derive<D: LinkerDialect>(
    dialect: &D,
    package: &TargetAstPackage<D>,
) -> Result<SymbolCatalogue<D>, Vec<Diagnostic>> {
    match dialect.package_symbol_catalogue(package) {
        Err(errors) if errors.is_empty() => Err(vec![link_error(
            DiagnosticCode::InvalidStructure,
            "package catalogue construction failed without a diagnostic",
            "catalogue",
        )]),
        result => result,
    }
}

pub(super) fn verify_package<D: LinkerDialect>(
    package: &LinkedTargetPackage<D>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match derive(&package.dialect, &package.unresolved) {
        Ok(expected) if expected == package.catalogue => {}
        Ok(_) => diagnostics.push(link_error(
            DiagnosticCode::InterfaceNonconformance,
            "linked catalogue is not exactly original-package-derived",
            "catalogue",
        )),
        Err(mut errors) => diagnostics.append(&mut errors),
    }
}

impl<D: LinkerDialect> SymbolCatalogue<D> {
    pub fn verify(&self, dialect: &D) -> Result<(), Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        dependency_callables::verify(self, dialect, &mut diagnostics);
        dependency_values::verify(self, dialect, &mut diagnostics);
        check_unique(
            &mut diagnostics,
            self.types.iter().map(|spec| (&spec.symbol, &spec.source)),
            "known type",
        );
        check_unique(
            &mut diagnostics,
            self.callables
                .iter()
                .map(|spec| (&spec.symbol, &spec.source)),
            "known callable",
        );
        check_unique(
            &mut diagnostics,
            self.runtime_callables
                .iter()
                .map(|spec| (&spec.symbol, &spec.source)),
            "runtime callable",
        );
        check_unique(
            &mut diagnostics,
            self.fields.iter().map(|spec| (&spec.symbol, &spec.source)),
            "known field",
        );
        check_unique(
            &mut diagnostics,
            self.constructors
                .iter()
                .map(|spec| (&spec.symbol, &spec.source)),
            "known constructor",
        );
        check_unique(
            &mut diagnostics,
            self.methods.iter().map(|spec| (&spec.symbol, &spec.source)),
            "known method",
        );
        check_unique(
            &mut diagnostics,
            self.helpers.iter().map(|spec| (&spec.id, &spec.source)),
            "runtime helper",
        );
        check_unique(
            &mut diagnostics,
            self.helpers.iter().map(|spec| (&spec.order, &spec.source)),
            "runtime helper order",
        );
        for helper in &self.helpers {
            if dialect.is_public(&helper.visibility) {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidStructure,
                    "runtime helpers cannot be exposed as public declarations",
                    helper.source.clone(),
                ));
            }
            if helper.items.is_empty() {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::InvalidStructure,
                    "runtime helper must expand to structural target AST items",
                    helper.source.clone(),
                ));
            }
        }

        for spec in &self.callables {
            validate_callable_pattern(&mut diagnostics, &spec.signature, &spec.source);
            if let Some(concrete) = concrete_signature(&spec.signature) {
                let authoritative = dialect.known_callable_signature(&spec.symbol);
                if concrete != authoritative {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        "known-callable metadata disagrees with its typed signature",
                        spec.source.clone(),
                    ));
                }
            }
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                spec.qualified_name.as_ref(),
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        for spec in &self.runtime_callables {
            validate_callable_pattern(&mut diagnostics, &spec.signature, &spec.source);
            if let Some(concrete) = concrete_signature(&spec.signature) {
                let authoritative = dialect.runtime_callable_signature(&spec.symbol);
                if concrete != authoritative {
                    diagnostics.push(Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        "runtime-callable metadata disagrees with its typed signature",
                        spec.source.clone(),
                    ));
                }
            }
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                spec.qualified_name.as_ref(),
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        for spec in &self.constructors {
            validate_callable_pattern(&mut diagnostics, &spec.signature, &spec.source);
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                spec.qualified_name.as_ref(),
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        for spec in &self.methods {
            validate_callable_pattern(&mut diagnostics, &spec.signature, &spec.source);
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                None,
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        for spec in &self.types {
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                spec.qualified_name.as_ref(),
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        for spec in &self.fields {
            validate_dependency(
                &mut diagnostics,
                &spec.origin,
                &spec.policy,
                None,
                spec.dependency.as_ref(),
                &spec.source,
            );
        }
        sort_diagnostics(&mut diagnostics);
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(diagnostics)
        }
    }

    pub(super) fn helper(&self, id: &D::HelperId) -> Option<&RuntimeHelperSpec<D>> {
        self.helpers.iter().find(|spec| &spec.id == id)
    }
}
