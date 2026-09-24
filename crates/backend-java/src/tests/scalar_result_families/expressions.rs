use crate::ast::*;
use crate::dialect::JavaKnownCallable;
use crate::tests::source_record_fixture::{boolean, int, name};
use portable_codegen::{GeneratedCallableId, GeneratedTypeId};

pub fn reference(id: GeneratedTypeId) -> JavaType {
    JavaType::Reference(JavaTypeName::Generated(id))
}
pub fn local(ty: JavaType, spelling: &str) -> JavaExpr {
    JavaExpr::local(ty, name(spelling))
}
pub fn parameter(ty: JavaType, spelling: &str) -> JavaParameter {
    JavaParameter {
        ty,
        name: name(spelling),
        final_parameter: true,
    }
}
pub fn signature(parameters: Vec<JavaType>, result: JavaType) -> JavaMethodSignature {
    JavaMethodSignature {
        receiver: None,
        parameters,
        result,
        checked_exceptions: vec![],
        nullable_result: false,
        pure: true,
    }
}
pub fn new(owner: GeneratedTypeId, arguments: Vec<JavaExpr>) -> JavaExpr {
    JavaExpr {
        ty: reference(owner),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::New {
            constructor: JavaConstructorRef::Generated {
                owner,
                parameters: arguments.iter().map(|arg| arg.ty.clone()).collect(),
            },
            arguments,
        },
    }
}
pub fn upcast(value: JavaExpr, owner: GeneratedTypeId, interface: GeneratedTypeId) -> JavaExpr {
    JavaExpr {
        ty: reference(interface),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::InterfaceCoercion {
            implementation: JavaInterfaceWitness::for_generated_adapter(owner, interface),
            target: reference(interface),
            value: Box::new(value),
        },
    }
}
pub fn call(
    symbol: GeneratedCallableId,
    signature: JavaMethodSignature,
    arguments: Vec<JavaExpr>,
) -> JavaExpr {
    JavaExpr {
        ty: signature.result.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Generated { symbol, signature },
            receiver: None,
            arguments,
        },
    }
}
pub fn nonnull(value: JavaExpr) -> JavaExpr {
    let ty = value.ty.clone();
    JavaExpr {
        ty: ty.clone(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Known {
                callable: JavaKnownCallable::ObjectsRequireNonNull,
                signature: signature(vec![ty.clone()], ty),
            },
            receiver: None,
            arguments: vec![value],
        },
    }
}
pub fn payload(owner: GeneratedTypeId) -> JavaSynthesizedField {
    JavaSynthesizedField {
        owner,
        role: JavaSynthesizedFieldRole::ScalarResultPayload,
    }
}
pub fn read_payload(owner: GeneratedTypeId, spelling: &str) -> JavaExpr {
    let owner_type = reference(owner);
    let mut signature = signature(vec![], int());
    signature.receiver = Some(owner_type.clone());
    JavaExpr {
        ty: int(),
        precedence: JavaPrecedence::Primary,
        kind: JavaExprKind::Call {
            callable: JavaCallableRef::Member {
                owner: owner_type.clone(),
                name: name("value"),
                signature: Box::new(signature),
                origin: JavaMemberOrigin::SynthesizedField(payload(owner)),
            },
            receiver: Some(Box::new(local(owner_type, spelling))),
            arguments: vec![],
        },
    }
}
pub fn instance(value: JavaExpr, target: GeneratedTypeId, binding: Option<&str>) -> JavaExpr {
    JavaExpr {
        ty: boolean(),
        precedence: JavaPrecedence::Relational,
        kind: JavaExprKind::InstanceOf {
            value: Box::new(value),
            target: reference(target),
            binding: binding.map(name),
        },
    }
}
pub fn final_local(spelling: &str, value: JavaExpr) -> JavaStmt {
    JavaStmt::Local {
        finality: JavaLocalFinality::Final,
        ty: value.ty.clone(),
        name: name(spelling),
        value: Some(value),
    }
}
pub fn returned(value: JavaExpr) -> JavaStmt {
    JavaStmt::Return(Some(value))
}
