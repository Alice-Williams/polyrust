//! Checked record/interface edges survive lowering even when no call references them.

use super::{
    JavaHeritage, JavaKnownType, JavaMember, JavaMethodDeclaration, JavaType, JavaTypeDeclaration,
    JavaTypeName,
};
use crate::dialect::JavaDialect;
use portable_codegen::{AstViolation, GeneratedOrigin, TargetAstContext};
use portable_core_ir::{
    CoreDeclaration, CoreImplementationId, CoreImplementationMethodId, CoreInterfaceId,
    CoreProgram, CoreRecordId,
};
use portable_diagnostics::DiagnosticCode;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Conformance {
    implementation: CoreImplementationId,
    interface: CoreInterfaceId,
    methods: Vec<CoreImplementationMethodId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JavaConformanceInventory {
    records: BTreeMap<CoreRecordId, Vec<Conformance>>,
}

impl JavaConformanceInventory {
    /// Structural shells cannot claim to declare checked portable records.
    pub fn structural() -> Self {
        Self {
            records: BTreeMap::new(),
        }
    }

    pub(crate) fn from_checked(core: &CoreProgram) -> Self {
        let mut records = BTreeMap::new();
        for declaration in &core.module().declarations {
            if let CoreDeclaration::Record(id) = *declaration {
                records.insert(id, Vec::new());
            }
        }
        for declaration in &core.module().declarations {
            if let CoreDeclaration::Implementation(id) = *declaration {
                let value = core.implementation(id).expect("checked conformance");
                records
                    .get_mut(&value.record)
                    .expect("checked record")
                    .push(Conformance {
                        implementation: id,
                        interface: value.interface,
                        methods: value.methods.clone(),
                    });
            }
        }
        Self { records }
    }

    pub(super) fn verify(
        &self,
        declaration: &JavaTypeDeclaration,
        context: &TargetAstContext<'_, JavaDialect>,
    ) -> Vec<AstViolation> {
        let mut actual = BTreeMap::new();
        let mut violations = Vec::new();
        collect_records(declaration, context, &mut actual, &mut violations);
        if self.records.keys().ne(actual.keys()) {
            violations.push(error(
                "Java checked record inventory differs from its declarations",
            ));
        }
        for (record, expected) in &self.records {
            let Some(declaration) = actual.get(record) else {
                continue;
            };
            let mut interfaces = Vec::new();
            if let JavaHeritage::Interfaces(values) = &declaration.heritage {
                for value in values {
                    match value {
                        JavaType::Reference(JavaTypeName::Known(
                            JavaKnownType::RuntimeSemanticValue,
                        )) => {}
                        JavaType::Reference(JavaTypeName::Generated(id)) => {
                            match context.generated_type(*id).map(|value| &value.origin) {
                                Some(GeneratedOrigin::CoreDeclaration(
                                    CoreDeclaration::Interface(id),
                                )) => interfaces.push(*id),
                                _ => violations.push(error(
                                    "Java checked record has an unregistered conformance edge",
                                )),
                            }
                        }
                        _ => violations.push(error(
                            "Java checked record has a non-portable conformance edge",
                        )),
                    }
                }
            }
            let mut expected_interfaces = expected
                .iter()
                .map(|value| value.interface)
                .collect::<Vec<_>>();
            interfaces.sort();
            expected_interfaces.sort();
            let mut methods = declaration
                .members
                .iter()
                .filter_map(|member| match member {
                    JavaMember::Method(method) => match method.declared {
                        JavaMethodDeclaration::Implementation { method, .. } => Some(method),
                        _ => None,
                    },
                    _ => None,
                })
                .collect::<Vec<_>>();
            let mut expected_methods = expected
                .iter()
                .flat_map(|value| value.methods.iter().copied())
                .collect::<Vec<_>>();
            methods.sort();
            expected_methods.sort();
            if interfaces != expected_interfaces || methods != expected_methods {
                let identities = expected
                    .iter()
                    .map(|value| value.implementation)
                    .collect::<Vec<_>>();
                violations.push(error(&format!("Java conformance edges and implementation methods must exactly match checked Core {identities:?}")));
            }
        }
        violations
    }
}

fn collect_records<'a>(
    declaration: &'a JavaTypeDeclaration,
    context: &TargetAstContext<'_, JavaDialect>,
    actual: &mut BTreeMap<CoreRecordId, &'a JavaTypeDeclaration>,
    violations: &mut Vec<AstViolation>,
) {
    let core_record = declaration
        .declared
        .and_then(|id| context.generated_type(id))
        .is_some_and(|value| {
            matches!(
                value.origin,
                GeneratedOrigin::CoreDeclaration(CoreDeclaration::Record(_))
            )
        });
    if !core_record && let JavaHeritage::Interfaces(values) = &declaration.heritage {
        for value in values {
            if let JavaType::Reference(JavaTypeName::Generated(id)) = value
                && context.generated_type(*id).is_some_and(|value| {
                    matches!(
                        value.origin,
                        GeneratedOrigin::CoreDeclaration(CoreDeclaration::Interface(_))
                    )
                })
                && declaration.kind != super::JavaDeclarationKind::UninhabitedEnum(*id)
            {
                violations.push(error("only checked records or certified uninhabited enums may implement portable interfaces"));
            }
        }
    }
    if let Some(GeneratedOrigin::CoreDeclaration(CoreDeclaration::Record(id))) = declaration
        .declared
        .and_then(|id| context.generated_type(id))
        .map(|value| &value.origin)
        && actual.insert(*id, declaration).is_some()
    {
        violations.push(error("Java checked record is declared more than once"));
    }
    for member in &declaration.members {
        if let JavaMember::NestedType(nested) = member {
            collect_records(nested, context, actual, violations);
        }
    }
}

fn error(message: &str) -> AstViolation {
    AstViolation::new(DiagnosticCode::InterfaceNonconformance, message)
}
