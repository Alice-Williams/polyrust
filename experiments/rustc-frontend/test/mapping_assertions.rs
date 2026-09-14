//! Compiler-produced AST coverage for every admitted source capability owner.
use portable_backend_c::ast::*;
use std::collections::BTreeSet;

#[derive(Default)]
struct Facts {
    comparisons: Vec<CBinaryOperator>,
    integers: BTreeSet<i32>,
    booleans: BTreeSet<bool>,
    bool_conversions: usize,
    const_conversions: usize,
    dereferences: usize,
    nested_blocks: usize,
    pointer_depth: usize,
    record_initializers: usize,
    scopes: BTreeSet<CScopeRef>,
}

pub(super) fn check(source: &CSourceFile) {
    let records: Vec<_> = source
        .items()
        .iter()
        .filter_map(|item| match item {
            CFileItem::Declaration(declaration) => match declaration.kind() {
                CDeclarationKind::Aggregate {
                    owner: CAggregateRef::Struct(owner),
                    members,
                } => Some((owner, members)),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(records.len(), 1);
    let (record, members) = records[0];
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].ty(), &CObjectType::scalar(CScalarType::I32));
    assert_eq!(members[1].ty(), &CObjectType::scalar(CScalarType::Bool));
    let functions: Vec<_> = source
        .items()
        .iter()
        .filter_map(|item| match item {
            CFileItem::Definition(definition) => match definition.kind() {
                CDefinitionKind::Function {
                    function,
                    parameters,
                    body,
                    ..
                } => Some((function, parameters, body)),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(functions.len(), 1);
    let (function, parameters, body) = functions[0];
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters[0].ty(), &CObjectType::scalar(CScalarType::I32));
    let CReturnType::Value(result) = function.signature().return_type() else {
        panic!("scalar result");
    };
    assert_eq!(
        result.declared_type(),
        &CObjectType::scalar(CScalarType::I32)
    );
    let mut facts = Facts::default();
    assert_eq!(body.scope().function(), function);
    facts.block(body, record, members, None);
    for expected in [
        CBinaryOperator::Equal,
        CBinaryOperator::NotEqual,
        CBinaryOperator::Less,
        CBinaryOperator::LessEqual,
        CBinaryOperator::Greater,
        CBinaryOperator::GreaterEqual,
    ] {
        assert!(
            facts.comparisons.contains(&expected),
            "missing {expected:?}"
        );
    }
    assert_eq!(facts.bool_conversions, facts.comparisons.len());
    assert_eq!(facts.const_conversions, 2);
    assert_eq!(facts.pointer_depth, 2);
    assert!(facts.dereferences >= 2);
    assert!(facts.nested_blocks > 0);
    assert_eq!(facts.record_initializers, 1);
    assert_eq!(facts.booleans, BTreeSet::from([false, true]));
    assert_eq!(
        facts.integers,
        BTreeSet::from([i32::MIN, -10, -7, 0, 1, 10])
    );
}

impl Facts {
    fn block(
        &mut self,
        block: &CBlock,
        record: &CStructRef,
        members: &[CMemberRef],
        parent: Option<&CScopeRef>,
    ) {
        assert_eq!(block.scope().parent(), parent);
        assert!(
            self.scopes.insert(block.scope().clone()),
            "duplicate lexical identity"
        );
        if let Some(parent) = parent {
            assert_eq!(block.scope().function(), parent.function());
        }
        for statement in block.statements() {
            match statement.kind() {
                CStatementKind::Declare(local) => {
                    assert_eq!(local.local().scope(), block.scope());
                    let mut ty = local.local().ty();
                    let mut depth = 0;
                    while let CObjectTypeKind::Pointer(CPointerTarget::Object(pointee)) = ty.kind()
                    {
                        assert_eq!(pointee.constness(), CConstness::Const);
                        depth += 1;
                        ty = pointee;
                    }
                    self.pointer_depth = self.pointer_depth.max(depth);
                    self.initializer(local.initializer().unwrap(), record, members);
                }
                CStatementKind::Discard(value) | CStatementKind::Return(Some(value)) => {
                    self.value(value)
                }
                CStatementKind::Block(child) => {
                    self.nested_blocks += 1;
                    self.block(child, record, members, Some(block.scope()));
                }
                CStatementKind::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    self.value(condition);
                    self.block(then_block, record, members, Some(block.scope()));
                    self.block(else_block, record, members, Some(block.scope()));
                }
                _ => panic!("unadmitted statement in mapping inventory"),
            }
        }
    }

    fn initializer(
        &mut self,
        initializer: &CInitializer,
        record: &CStructRef,
        fields: &[CMemberRef],
    ) {
        match initializer.kind() {
            CInitializerKind::Expression(value) => self.value(value),
            CInitializerKind::Struct { owner, members } => {
                self.record_initializers += 1;
                assert_eq!(owner, record);
                assert_eq!(members.len(), fields.len());
                for ((member, value), expected) in members.iter().zip(fields) {
                    assert_eq!(
                        member, expected,
                        "source field order must map to declaration identity"
                    );
                    self.initializer(value, record, fields);
                }
            }
            _ => panic!("unadmitted initializer"),
        }
    }

    fn value(&mut self, value: &CValue) {
        match value.kind() {
            CValueKind::Literal(CLiteral::Signed(CSignedLiteral::I32(value))) => {
                self.integers.insert(*value);
            }
            CValueKind::Literal(CLiteral::Bool(value)) => {
                self.booleans.insert(*value);
            }
            CValueKind::Read(place) | CValueKind::AddressOf(place) => self.place(place),
            CValueKind::Binary {
                operator,
                left,
                right,
            } => {
                self.comparisons.push(*operator);
                self.value(left);
                self.value(right);
            }
            CValueKind::Convert {
                conversion,
                operand,
            } => {
                match conversion {
                    CConversion::Numeric(CScalarType::Bool) => self.bool_conversions += 1,
                    CConversion::AddConst(_) => self.const_conversions += 1,
                    _ => panic!("unadmitted conversion"),
                }
                self.value(operand);
            }
            _ => panic!("unadmitted expression"),
        }
    }

    fn place(&mut self, place: &CPlace) {
        match place.kind() {
            CPlaceKind::Local(_) | CPlaceKind::Parameter(_) => {}
            CPlaceKind::Dereference(value) => {
                self.dereferences += 1;
                self.value(value);
            }
            CPlaceKind::Member { base, member } => {
                let CObjectTypeKind::Struct(owner) = base.ty().kind() else {
                    panic!("nominal owner");
                };
                assert_eq!(member.owner(), &CAggregateRef::Struct(owner.clone()));
                self.place(base);
            }
            _ => panic!("unadmitted place"),
        }
    }
}
