//! Java lowering: registration.

use super::{Lowering, java_visibility, source};
use crate::ast::{JavaDeclarationKind, JavaType, JavaTypeName, JavaVisibility};
use crate::dialect::{JavaDialect, JavaInvocationKind};
use portable_codegen::{
    GeneratedCallable, GeneratedInterfaceMethod, GeneratedOrigin, GeneratedSymbolId, GeneratedType,
    GeneratedValue, SynthesisReason, TargetCallableSignature,
};
use portable_core_ir::CoreDeclaration;
use portable_diagnostics::Diagnostic;

impl Lowering<'_> {
    pub(super) fn register_types(&mut self) {
        let entry = self.builder.generated_type(GeneratedType {
            name: "Generated".to_owned(),
            kind: JavaDeclarationKind::FinalClass,
            visibility: JavaVisibility::Public,
            origin: GeneratedOrigin::Synthesized(SynthesisReason::PackageEntryPoint),
            source: source("Generated"),
        });
        self.declared.push(GeneratedSymbolId::Type(entry));
        self.entry = Some(entry);
        for declaration in &self.core.module().declarations {
            match *declaration {
                CoreDeclaration::Record(id) => {
                    let item = self.core.record(id).expect("verified record");
                    let generated = self.builder.generated_type(GeneratedType {
                        name: item.header.name.clone(),
                        kind: JavaDeclarationKind::Record,
                        visibility: java_visibility(item.header.visibility),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: item.header.source.clone(),
                    });
                    self.records.insert(id, generated);
                    self.declared.push(GeneratedSymbolId::Type(generated));
                }
                CoreDeclaration::Enum(id) => {
                    let item = self.core.enumeration(id).expect("verified enum");
                    let payload_free = self.enum_is_payload_free(id);
                    let generated = self.builder.generated_type(GeneratedType {
                        name: item.header.name.clone(),
                        kind: if payload_free {
                            JavaDeclarationKind::Enum
                        } else {
                            JavaDeclarationKind::SealedInterface
                        },
                        visibility: java_visibility(item.header.visibility),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: item.header.source.clone(),
                    });
                    self.enums.insert(id, generated);
                    self.declared.push(GeneratedSymbolId::Type(generated));
                    for variant in item.variants.iter().filter(|_| !payload_free) {
                        let value = self.core.variant(*variant).expect("verified variant");
                        let generated = self.builder.generated_type(GeneratedType {
                            name: format!("{}{}", item.header.name, value.header.name),
                            kind: JavaDeclarationKind::Record,
                            visibility: java_visibility(item.header.visibility),
                            origin: GeneratedOrigin::CoreDeclaration(*declaration),
                            source: value.header.source.clone(),
                        });
                        self.variants.insert(*variant, generated);
                        self.declared.push(GeneratedSymbolId::Type(generated));
                    }
                }
                CoreDeclaration::Interface(id) => {
                    let item = self.core.interface(id).expect("verified interface");
                    let generated = self.builder.generated_type(GeneratedType {
                        name: item.header.name.clone(),
                        kind: JavaDeclarationKind::SealedInterface,
                        visibility: java_visibility(item.header.visibility),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: item.header.source.clone(),
                    });
                    self.interfaces.insert(id, generated);
                    self.declared.push(GeneratedSymbolId::Type(generated));
                }
                CoreDeclaration::Constant(_)
                | CoreDeclaration::Alias(_)
                | CoreDeclaration::Implementation(_)
                | CoreDeclaration::Function(_)
                | CoreDeclaration::Test(_) => {}
            }
        }
    }

    pub(super) fn register_values_and_callables(&mut self) -> Result<(), Vec<Diagnostic>> {
        for declaration in &self.core.module().declarations {
            match *declaration {
                CoreDeclaration::Constant(id) => {
                    let value = self.core.constant(id).expect("verified constant");
                    let java_type = self.ty(value.ty)?;
                    let symbol = self.builder.value(GeneratedValue {
                        name: value.header.name.clone(),
                        ty: JavaDialect.registered_type(&java_type),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: value.header.source.clone(),
                    });
                    self.constants.insert(id, symbol);
                    self.declared.push(GeneratedSymbolId::Value(symbol));
                }
                CoreDeclaration::Function(id) => {
                    let value = self.core.function(id).expect("verified function");
                    let parameters = value
                        .parameters
                        .iter()
                        .map(|parameter| {
                            self.ty(parameter.ty)
                                .map(|ty| JavaDialect.registered_type(&ty))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let result = self.poly_result_type(value.return_type)?;
                    let symbol = self.builder.callable(GeneratedCallable {
                        name: value.header.name.clone(),
                        signature: TargetCallableSignature {
                            invocation: JavaInvocationKind::Static,
                            receiver: None,
                            parameters,
                            return_type: JavaDialect.registered_type(&result),
                        },
                        visibility: java_visibility(value.header.visibility),
                        origin: GeneratedOrigin::CoreDeclaration(*declaration),
                        source: value.header.source.clone(),
                    });
                    self.functions.insert(id, symbol);
                    self.declared.push(GeneratedSymbolId::Callable(symbol));
                }
                CoreDeclaration::Interface(id) => {
                    let interface = self.core.interface(id).expect("verified interface");
                    let owner = self.interfaces[&id];
                    for method_id in &interface.methods {
                        let method = self
                            .core
                            .interface_method(*method_id)
                            .expect("verified method");
                        let parameters = method
                            .parameters
                            .iter()
                            .map(|parameter| {
                                self.ty(parameter.ty)
                                    .map(|ty| JavaDialect.registered_type(&ty))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let receiver = JavaType::Reference(JavaTypeName::Generated(owner));
                        let result = self.poly_result_type(method.return_type)?;
                        let symbol = self.builder.interface_method(GeneratedInterfaceMethod {
                            owner,
                            name: method.header.name.clone(),
                            signature: TargetCallableSignature {
                                invocation: JavaInvocationKind::Instance,
                                receiver: Some(JavaDialect.registered_type(&receiver)),
                                parameters,
                                return_type: JavaDialect.registered_type(&result),
                            },
                            origin: GeneratedOrigin::CoreDeclaration(*declaration),
                            source: method.header.source.clone(),
                        });
                        self.interface_methods.insert(*method_id, symbol);
                        self.declared
                            .push(GeneratedSymbolId::InterfaceMethod(symbol));
                    }
                }
                CoreDeclaration::Alias(_)
                | CoreDeclaration::Record(_)
                | CoreDeclaration::Implementation(_)
                | CoreDeclaration::Test(_) => {}
                CoreDeclaration::Enum(id) => {
                    if self.enum_is_payload_free(id) {
                        let enumeration = self.core.enumeration(id).expect("verified enum");
                        let ty = JavaType::Reference(JavaTypeName::Generated(self.enums[&id]));
                        for variant_id in &enumeration.variants {
                            let variant = self.core.variant(*variant_id).expect("verified variant");
                            let symbol = self.builder.value(GeneratedValue {
                                name: variant.header.name.clone(),
                                ty: JavaDialect.registered_type(&ty),
                                origin: GeneratedOrigin::CoreDeclaration(*declaration),
                                source: variant.header.source.clone(),
                            });
                            self.enum_values.insert(*variant_id, symbol);
                            self.declared.push(GeneratedSymbolId::Value(symbol));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
