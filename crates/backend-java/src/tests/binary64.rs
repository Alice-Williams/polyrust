//! Exact double values use finite payloads and original-owner certificates.
use crate::tests::source_dependency_fixture as f;
use crate::{ast::*, dialect::*};
use portable_codegen::*;

pub(super) fn values() -> Vec<u64> {
    let mut bits = vec![
        0,
        1,
        0x000f_ffff_ffff_ffff,
        0x0010_0000_0000_0000,
        0x3fef_ffff_ffff_ffff,
        0x3ff0_0000_0000_0000,
        0x3ff0_0000_0000_0001,
        0x7fef_ffff_ffff_ffff,
    ];
    let mut seed = 0x1462_9538_784a_6b29_u64;
    for _ in 0..128 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let magnitude = seed & 0x7fff_ffff_ffff_ffff;
        bits.push(if magnitude >> 52 == 0x7ff {
            magnitude ^ (1 << 52)
        } else {
            magnitude
        });
    }
    bits.extend(bits.clone().into_iter().map(|bits| bits | (1 << 63)));
    bits
}
pub(super) fn double() -> JavaType {
    JavaType::primitive(JavaPrimitive::Double)
}
pub(super) fn functions(bits: &[u64]) -> Vec<f::Function> {
    let mut functions: Vec<_> = bits
        .iter()
        .enumerate()
        .map(|(index, bits)| f::Function {
            hash: 10 + index as u64,
            public: true,
            name: f::name(&format!("literal{index}")),
            parameters: vec![],
            result: double(),
            body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::literal(
                double(),
                JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(*bits).unwrap()),
            )))]),
        })
        .collect();
    functions.push(f::Function {
        hash: 10 + bits.len() as u64,
        public: true,
        name: f::name("identity"),
        parameters: vec![JavaParameter {
            ty: double(),
            name: f::name("input"),
            final_parameter: true,
        }],
        result: double(),
        body: JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr::local(
            double(),
            f::name("input"),
        )))]),
    });
    functions
}
pub(super) fn chain(bits: &[u64]) -> Vec<JavaDependencyApi> {
    let first =
        JavaDependencyApi::from_certificate(f::certify(f::package(91, functions(bits)))).unwrap();
    let mut scope = JavaDependencyScope::new();
    let mut declarations = functions(bits);
    for (function, declaration) in first.functions().zip(&mut declarations) {
        let (next, callable) = scope.import(function.clone());
        scope = next;
        let arguments = declaration
            .parameters
            .iter()
            .map(|p| JavaExpr::local(p.ty.clone(), p.name.clone()))
            .collect();
        declaration.body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: double(),
            precedence: JavaPrecedence::Primary,
            kind: JavaExprKind::Call {
                callable: JavaCallableRef::Dependency(callable),
                receiver: None,
                arguments,
            },
        }))]);
    }
    let second = JavaDependencyApi::from_certificate(f::certify(f::package_with_dependencies(
        92,
        declarations,
        scope.finish(),
    )))
    .unwrap();
    vec![first, second]
}

#[test]
fn finite_values_and_original_owner_transport_are_certified() {
    for owner in chain(&values()) {
        let output = render_certified_package(&JavaStructuralRenderer, owner.package()).unwrap();
        let OutputContents::Text(text) = output.files()[0].contents() else {
            panic!("source")
        };
        assert!(text.len() as u64 <= owner.source_byte_bound().unwrap());
        assert!(text.contains("double "));
        assert!(!text.contains("Runtime") && !text.contains("longBitsToDouble"));
    }
}

#[test]
fn finite_literal_cannot_claim_a_narrower_type() {
    let mut declarations = functions(&[1]);
    let JavaStmt::Return(Some(value)) = &mut declarations[0].body.statements[0] else {
        panic!("return")
    };
    value.ty = JavaType::primitive(JavaPrimitive::Long);
    assert!(verify_unresolved_package(&JavaDialect, f::package(91, declarations)).is_err());
}

#[path = "binary64_native.rs"]
mod native;

#[test]
fn closed_double_owner_admits_negation_not_binary_arithmetic_or_casts() {
    for operator in [
        JavaBinaryOperator::Add,
        JavaBinaryOperator::Subtract,
        JavaBinaryOperator::Multiply,
        JavaBinaryOperator::Divide,
        JavaBinaryOperator::Remainder,
    ] {
        let mut declarations = functions(&[1]);
        let literal = JavaExpr::literal(
            double(),
            JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(1).unwrap()),
        );
        let value = JavaExpr {
            ty: double(),
            precedence: JavaPrecedence::Additive,
            kind: JavaExprKind::Binary {
                operator,
                left: Box::new(literal.clone()),
                right: Box::new(literal),
            },
        };
        declarations[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(value))]);
        let package = f::certify(f::package(91, declarations));
        assert!(JavaDependencyApi::from_certificate(package).is_err());
    }
    for target in [JavaPrimitive::Int, JavaPrimitive::Long] {
        let mut declarations = functions(&[1]);
        let target = JavaType::primitive(target);
        declarations[0].result = target.clone();
        declarations[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
            ty: target.clone(),
            precedence: JavaPrecedence::Unary,
            kind: JavaExprKind::Cast {
                target,
                value: Box::new(JavaExpr::literal(
                    double(),
                    JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(1).unwrap()),
                )),
            },
        }))]);
        assert!(
            JavaDependencyApi::from_certificate(f::certify(f::package(91, declarations))).is_err()
        );
    }
    let mut declarations = functions(&[1]);
    let literal = JavaExpr::literal(
        double(),
        JavaLiteral::F64(portable_binary64::FiniteBinary64::from_bits(1).unwrap()),
    );
    declarations[0].body = JavaBlock::new(vec![JavaStmt::Return(Some(JavaExpr {
        ty: double(),
        precedence: JavaPrecedence::Unary,
        kind: JavaExprKind::Unary {
            operator: JavaUnaryOperator::Negate,
            operand: Box::new(literal),
        },
    }))]);
    assert!(JavaDependencyApi::from_certificate(f::certify(f::package(91, declarations))).is_ok());
}

#[path = "binary64_comparisons.rs"]
mod comparisons;

#[path = "binary64_records.rs"]
mod records;

#[path = "binary64_trace.rs"]
mod trace;

#[path = "binary64_negation.rs"]
mod negation;
