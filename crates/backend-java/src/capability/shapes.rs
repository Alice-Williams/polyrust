//! Java-specific legality checks for the dynamic compatibility boundary.

use super::*;

pub(super) fn java_illegal_shape_diagnostics(program: &CoreProgram) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for record in program.records() {
        collect_restricted_record_components(program, &record.fields, &mut diagnostics);
    }
    for variant in program.variants() {
        collect_restricted_record_components(program, &variant.fields, &mut diagnostics);
    }

    for function in program.functions() {
        let name = JavaIdentifier::from_portable(&function.header.name);
        let parameters = function
            .parameters
            .iter()
            .map(|parameter| erased_type(program, parameter.ty))
            .collect::<Vec<_>>();
        if JavaObjectMethod::from_erased_signature(name.as_str(), &parameters).is_some() {
            diagnostics.push(java_capability_diagnostic(
                format!(
                    "Java static method {}({}) conflicts with an inherited java.lang.Object instance method",
                    name.as_str(),
                    parameters.join(", ")
                ),
                function.header.source.clone(),
            ));
        }
    }

    for declaration in &program.module().declarations {
        let CoreDeclaration::Interface(interface_id) = *declaration else {
            continue;
        };
        let Some(interface) = program.interface(interface_id) else {
            continue;
        };
        if !program
            .implementations()
            .iter()
            .any(|implementation| implementation.interface == interface_id)
        {
            diagnostics.push(java_capability_diagnostic(
                format!(
                    "Java immutable interface {} has no generated implementation to seal",
                    interface.header.name
                ),
                interface.header.source.clone(),
            ));
        }
        for method_id in &interface.methods {
            let Some(method) = program.interface_method(*method_id) else {
                continue;
            };
            let name = JavaIdentifier::from_portable(&method.header.name);
            let parameters = method
                .parameters
                .iter()
                .map(|parameter| erased_type(program, parameter.ty))
                .collect::<Vec<_>>();
            if JavaObjectMethod::from_erased_signature(name.as_str(), &parameters).is_some() {
                diagnostics.push(java_capability_diagnostic(
                    format!(
                        "Java interface method {}({}) conflicts with its inherited java.lang.Object method",
                        name.as_str(),
                        parameters.join(", ")
                    ),
                    method.header.source.clone(),
                ));
            }
        }
    }

    for implementation in program.implementations() {
        let Some(record) = program.record(implementation.record) else {
            continue;
        };
        let accessors = record
            .fields
            .iter()
            .filter_map(|field| program.field(*field))
            .map(|field| {
                JavaIdentifier::from_portable(&field.header.name)
                    .as_str()
                    .to_owned()
            })
            .collect::<Vec<_>>();
        for method_id in &implementation.methods {
            let Some(method) = program.implementation_method(*method_id) else {
                continue;
            };
            let Some(interface_method) = program.interface_method(method.interface_method) else {
                continue;
            };
            let name = JavaIdentifier::from_portable(&interface_method.header.name);
            if interface_method.parameters.is_empty()
                && accessors.iter().any(|accessor| accessor == name.as_str())
            {
                diagnostics.push(java_capability_diagnostic(
                    format!(
                        "Java record accessor {}() collides with an interface implementation method after erasure",
                        name.as_str()
                    ),
                    method.header.source.clone(),
                ));
            }
        }
    }

    diagnostics
}

fn collect_restricted_record_components(
    program: &CoreProgram,
    fields: &[portable_core_ir::CoreFieldId],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for field_id in fields {
        let Some(field) = program.field(*field_id) else {
            continue;
        };
        let name = JavaIdentifier::from_portable(&field.header.name);
        if is_restricted_record_component(name.as_str()) {
            diagnostics.push(java_capability_diagnostic(
                format!(
                    "Java record component name {:?} is forbidden by Java 21",
                    name.as_str()
                ),
                field.header.source.clone(),
            ));
        }
    }
}

fn is_restricted_record_component(name: &str) -> bool {
    matches!(
        name,
        "clone"
            | "finalize"
            | "getClass"
            | "hashCode"
            | "notify"
            | "notifyAll"
            | "toString"
            | "wait"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JavaObjectMethod {
    GetClass,
    HashCode,
    Clone,
    ToString,
    Notify,
    NotifyAll,
    Wait,
    WaitMillis,
    WaitMillisNanos,
    Finalize,
}

impl JavaObjectMethod {
    pub(super) fn from_erased_signature(name: &str, parameters: &[String]) -> Option<Self> {
        match (name, parameters) {
            ("getClass", []) => Some(Self::GetClass),
            ("hashCode", []) => Some(Self::HashCode),
            ("clone", []) => Some(Self::Clone),
            ("toString", []) => Some(Self::ToString),
            ("notify", []) => Some(Self::Notify),
            ("notifyAll", []) => Some(Self::NotifyAll),
            ("wait", []) => Some(Self::Wait),
            ("wait", [millis]) if millis == "long" => Some(Self::WaitMillis),
            ("wait", [millis, nanos]) if millis == "long" && nanos == "int" => {
                Some(Self::WaitMillisNanos)
            }
            ("finalize", []) => Some(Self::Finalize),
            _ => None,
        }
    }
}

pub(super) fn valid_shape(feature: CoreFeature, shape: &FeatureShape) -> bool {
    match feature {
        CoreFeature::Declaration(feature) => match feature {
            DeclarationFeature::Record | DeclarationFeature::Enum => {
                matches!(shape, FeatureShape::Aggregate { .. })
            }
            DeclarationFeature::Function => matches!(shape, FeatureShape::Callable { .. }),
            DeclarationFeature::Constant
            | DeclarationFeature::Alias
            | DeclarationFeature::Interface
            | DeclarationFeature::Implementation
            | DeclarationFeature::Test => matches!(shape, FeatureShape::Unit),
        },
        CoreFeature::Type(feature) => match feature {
            TypeFeature::Unit
            | TypeFeature::Bool
            | TypeFeature::I32
            | TypeFeature::I64
            | TypeFeature::F64
            | TypeFeature::Char
            | TypeFeature::String
            | TypeFeature::Bytes
            | TypeFeature::List
            | TypeFeature::Option
            | TypeFeature::Result
            | TypeFeature::Record
            | TypeFeature::Enum
            | TypeFeature::Interface => matches!(shape, FeatureShape::Unit),
        },
        CoreFeature::Control(feature) => match feature {
            ControlFeature::Match => matches!(shape, FeatureShape::Aggregate { .. }),
            ControlFeature::Block
            | ControlFeature::Let
            | ControlFeature::ForEach
            | ControlFeature::Return
            | ControlFeature::Evaluate
            | ControlFeature::If
            | ControlFeature::WildcardPattern
            | ControlFeature::BoolPattern
            | ControlFeature::EnumPattern
            | ControlFeature::NonePattern
            | ControlFeature::SomePattern
            | ControlFeature::OkPattern
            | ControlFeature::ErrPattern => matches!(shape, FeatureShape::Unit),
        },
        CoreFeature::Interface(feature) => match feature {
            InterfaceFeature::Declaration => matches!(shape, FeatureShape::Interface(_)),
            InterfaceFeature::Conformance
            | InterfaceFeature::MultipleConformance
            | InterfaceFeature::StaticDispatch
            | InterfaceFeature::DynamicDispatch
            | InterfaceFeature::InterfaceValue => matches!(shape, FeatureShape::Unit),
        },
        CoreFeature::Operation(feature) => match feature {
            OperationFeature::ConstructRecord
            | OperationFeature::ConstructEnum
            | OperationFeature::ConstructList
            | OperationFeature::Match => matches!(shape, FeatureShape::Aggregate { .. }),
            OperationFeature::Call
            | OperationFeature::StaticMethodCall
            | OperationFeature::InterfaceCall => matches!(shape, FeatureShape::Callable { .. }),
            OperationFeature::Variadic(_) => matches!(shape, FeatureShape::Variadic { .. }),
            OperationFeature::Local => matches!(shape, FeatureShape::LocalBinding(_)),
            OperationFeature::Binary(
                CoreBinaryIntrinsic::Equal | CoreBinaryIntrinsic::NotEqual,
            ) => {
                matches!(shape, FeatureShape::Equality(_))
            }
            OperationFeature::Literal
            | OperationFeature::Constant
            | OperationFeature::SelfValue
            | OperationFeature::ConstructSome
            | OperationFeature::ConstructNone
            | OperationFeature::ConstructOk
            | OperationFeature::ConstructErr
            | OperationFeature::CoerceInterface
            | OperationFeature::Field
            | OperationFeature::Unary(_)
            | OperationFeature::Binary(_)
            | OperationFeature::Ternary(_)
            | OperationFeature::If
            | OperationFeature::Block => matches!(shape, FeatureShape::Unit),
        },
        CoreFeature::Ownership(feature) => match feature {
            OwnershipFeature::OnceLeftToRight | OwnershipFeature::OwnedImmutableValue => {
                matches!(shape, FeatureShape::Unit)
            }
        },
    }
}

pub(super) fn erased_conformance_collisions(program: &CoreProgram) -> Vec<Diagnostic> {
    let mut methods = BTreeMap::<(CoreRecordId, String, Vec<String>), _>::new();
    let mut diagnostics = Vec::new();
    for implementation in program.implementations() {
        for method_id in &implementation.methods {
            let Some(method) = program.implementation_method(*method_id) else {
                continue;
            };
            let Some(interface_method) = program.interface_method(method.interface_method) else {
                continue;
            };
            let key = (
                implementation.record,
                JavaIdentifier::from_portable(&interface_method.header.name)
                    .as_str()
                    .to_owned(),
                interface_method
                    .parameters
                    .iter()
                    .map(|parameter| erased_type(program, parameter.ty))
                    .collect::<Vec<_>>(),
            );
            if methods
                .insert(key.clone(), method.header.source.clone())
                .is_some()
            {
                let record_name = program
                    .record(implementation.record)
                    .map_or("<missing>", |record| record.header.name.as_str());
                let mut diagnostic = Diagnostic::error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "target org.polyrust.java cannot preserve distinct interface witnesses for record {record_name:?}: Java-erased method {}({}) collides",
                        key.1,
                        key.2.join(", ")
                    ),
                    method.header.source.clone(),
                );
                diagnostic.target = Some("org.polyrust.java".to_owned());
                diagnostics.push(diagnostic);
            }
        }
    }
    diagnostics
}

pub(super) fn fallible_constant_diagnostics(program: &CoreProgram) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for constant in program.constants() {
        collect_fallible_constants(&constant.value, &mut diagnostics);
    }
    diagnostics
}

fn collect_fallible_constants(value: &CoreConstantExpr, diagnostics: &mut Vec<Diagnostic>) {
    match &value.kind {
        CoreConstantExprKind::Intrinsic(intrinsic) => {
            if constant_intrinsic_is_fallible(intrinsic) {
                let mut diagnostic = Diagnostic::error(
                    DiagnosticCode::UnsupportedCapability,
                    "target org.polyrust.java cannot preserve a fallible intrinsic in a static constant initializer",
                    value.source.clone(),
                );
                diagnostic.target = Some("org.polyrust.java".to_owned());
                diagnostics.push(diagnostic);
            }
            match &**intrinsic {
                CoreIntrinsicExpr::Unary { operand, .. } => {
                    collect_fallible_constants(operand, diagnostics);
                }
                CoreIntrinsicExpr::Binary { left, right, .. } => {
                    collect_fallible_constants(left, diagnostics);
                    collect_fallible_constants(right, diagnostics);
                }
                CoreIntrinsicExpr::Ternary {
                    first,
                    second,
                    third,
                    ..
                } => {
                    collect_fallible_constants(first, diagnostics);
                    collect_fallible_constants(second, diagnostics);
                    collect_fallible_constants(third, diagnostics);
                }
                CoreIntrinsicExpr::Variadic { arguments, .. } => {
                    for child in arguments {
                        collect_fallible_constants(child, diagnostics);
                    }
                }
            }
        }
        CoreConstantExprKind::Record { fields, .. } | CoreConstantExprKind::Enum { fields, .. } => {
            for field in fields {
                collect_fallible_constants(&field.value, diagnostics);
            }
        }
        CoreConstantExprKind::Some(child)
        | CoreConstantExprKind::Ok { value: child, .. }
        | CoreConstantExprKind::Err { value: child, .. } => {
            collect_fallible_constants(child, diagnostics)
        }
        CoreConstantExprKind::List { elements, .. } => {
            for child in elements {
                collect_fallible_constants(child, diagnostics);
            }
        }
        CoreConstantExprKind::Literal(_)
        | CoreConstantExprKind::Constant(_)
        | CoreConstantExprKind::None { .. } => {}
    }
}

fn constant_intrinsic_is_fallible(intrinsic: &CoreIntrinsicExpr<CoreConstantExpr>) -> bool {
    match intrinsic {
        CoreIntrinsicExpr::Unary { operation, .. } => matches!(
            operation,
            CoreUnaryIntrinsic::IntNegChecked
                | CoreUnaryIntrinsic::StringScalarLength
                | CoreUnaryIntrinsic::NarrowI64ToI32Checked
                | CoreUnaryIntrinsic::StringFromUtf8Checked
        ),
        CoreIntrinsicExpr::Binary { operation, .. } => matches!(
            operation,
            CoreBinaryIntrinsic::IntAddChecked
                | CoreBinaryIntrinsic::IntSubChecked
                | CoreBinaryIntrinsic::IntMulChecked
                | CoreBinaryIntrinsic::IntDivChecked
                | CoreBinaryIntrinsic::IntRemChecked
                | CoreBinaryIntrinsic::IntShiftLeftChecked
                | CoreBinaryIntrinsic::IntShiftRightChecked
                | CoreBinaryIntrinsic::ListGetChecked
        ),
        CoreIntrinsicExpr::Ternary { operation, .. } => match operation {
            CoreTernaryIntrinsic::StringSliceScalars
            | CoreTernaryIntrinsic::StringReplaceAll
            | CoreTernaryIntrinsic::BytesReplaceAll => false,
        },
        CoreIntrinsicExpr::Variadic { operation, .. } => match operation {
            CoreVariadicIntrinsic::StringReplaceMany => false,
        },
    }
}

fn erased_type(program: &CoreProgram, ty: CoreTypeId) -> String {
    match program.types().get(ty) {
        Some(CoreType::Unit) => "Runtime.Unit".to_owned(),
        Some(CoreType::Bool) => "boolean".to_owned(),
        Some(CoreType::I32) => "int".to_owned(),
        Some(CoreType::I64) => "long".to_owned(),
        Some(CoreType::F64) => "double".to_owned(),
        Some(CoreType::Char) => "Runtime.Scalar".to_owned(),
        Some(CoreType::String) => "String".to_owned(),
        Some(CoreType::Bytes) => "Runtime.Bytes".to_owned(),
        Some(CoreType::List(_)) => "List".to_owned(),
        Some(CoreType::Option(_)) => "Runtime.PolyOption".to_owned(),
        Some(CoreType::Result { .. }) => "Runtime.PolyValueResult".to_owned(),
        Some(CoreType::Record(id)) => program
            .record(*id)
            .map(|record| {
                JavaIdentifier::from_portable(&record.header.name)
                    .as_str()
                    .to_owned()
            })
            .unwrap_or_else(|| "<missing-record>".to_owned()),
        Some(CoreType::Enum(id)) => program
            .enumeration(*id)
            .map(|item| {
                JavaIdentifier::from_portable(&item.header.name)
                    .as_str()
                    .to_owned()
            })
            .unwrap_or_else(|| "<missing-enum>".to_owned()),
        Some(CoreType::Interface(id)) => program
            .interface(*id)
            .map(|item| {
                JavaIdentifier::from_portable(&item.header.name)
                    .as_str()
                    .to_owned()
            })
            .unwrap_or_else(|| "<missing-interface>".to_owned()),
        None => "<missing-type>".to_owned(),
    }
}
