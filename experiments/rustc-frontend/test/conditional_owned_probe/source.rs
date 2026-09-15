//! Exact canonical fixture syntax, with branch and exit identities retained.
use rustc_abi::FieldIdx;
use rustc_hir::{self as hir, HirId, def::Res, def_id::LocalDefId};
use rustc_middle::ty::{TyCtxt, TypeckResults};

#[derive(Clone, Copy)]
pub(super) enum Case {
    Initialize { reversed: bool },
    Partial(FieldIdx),
    Early(FieldIdx),
}
impl Case {
    pub fn named(name: &str) -> Self {
        match name {
            "initialize" => Self::Initialize { reversed: false },
            "initialize_reversed" => Self::Initialize { reversed: true },
            "partial_first" => Self::Partial(FieldIdx::from_usize(0)),
            "partial_second" => Self::Partial(FieldIdx::from_usize(1)),
            "early_first" => Self::Early(FieldIdx::from_usize(0)),
            "early_second" => Self::Early(FieldIdx::from_usize(1)),
            _ => panic!("unexpected fixture {name}"),
        }
    }
}
pub(super) enum Exit<'tcx> {
    Tail(&'tcx hir::Expr<'tcx>),
    Return {
        expression: &'tcx hir::Expr<'tcx>,
        value: &'tcx hir::Expr<'tcx>,
    },
}
impl<'tcx> Exit<'tcx> {
    pub fn value(&self) -> &'tcx hir::Expr<'tcx> {
        match self {
            Self::Tail(value) | Self::Return { value, .. } => value,
        }
    }
}
pub(super) struct Source<'tcx> {
    pub case: Case,
    pub parameters: [HirId; 4],
    pub root: &'tcx hir::Block<'tcx>,
    pub branch: &'tcx hir::Block<'tcx>,
    pub constructors: Vec<(HirId, &'tcx hir::Expr<'tcx>)>,
    pub record: &'tcx hir::Expr<'tcx>,
    pub record_binding: HirId,
    pub guard: &'tcx hir::Expr<'tcx>,
    pub moves: [Option<(HirId, FieldIdx)>; 2],
    pub exits: [Exit<'tcx>; 2],
    pub reads: [HirId; 2],
}
pub(super) fn read(tcx: TyCtxt<'_>, owner: LocalDefId, case: Case) -> Source<'_> {
    let body = tcx.hir_body_owned_by(owner);
    let checked = tcx.typeck(owner);
    assert_eq!(body.params.len(), 4);
    let parameters: Vec<_> = body.params.iter().map(|p| binding(p.pat)).collect();
    for (index, id) in parameters.iter().enumerate() {
        assert_eq!(
            checked.node_type(*id),
            if index == 0 {
                tcx.types.bool
            } else {
                tcx.types.i32
            }
        );
    }
    assert_eq!(tcx.generics_of(owner).count(), 0);
    let signature = tcx.fn_sig(owner).instantiate_identity().skip_binder();
    assert!(signature.safety().is_safe());
    assert_eq!(signature.abi(), rustc_abi::ExternAbi::Rust);
    assert_eq!(signature.output(), tcx.types.i32);
    let hir::ExprKind::Block(root, None) = body.value.kind else {
        panic!("root")
    };
    assert_eq!(
        root.stmts.len(),
        if matches!(case, Case::Early(_)) { 6 } else { 5 }
    );
    let mut constructors = Vec::new();
    for (index, statement) in root.stmts[..3].iter().enumerate() {
        let declaration = declaration(statement);
        let expression = declaration.init.unwrap();
        let hir::ExprKind::Call(_, arguments) = expression.kind else {
            panic!("constructor")
        };
        assert_eq!(arguments.len(), 1);
        assert_eq!(
            local(checked, &arguments[0]),
            parameters[index + 1],
            "scalar parameter anchor"
        );
        constructors.push((binding(declaration.pat), expression));
    }
    let record_decl = declaration(&root.stmts[3]);
    let record_binding = binding(record_decl.pat);
    let condition = statement_expression(&root.stmts[4]);
    let hir::ExprKind::If(guard, then, None) = condition.kind else {
        panic!("one source if without else")
    };
    assert_eq!(
        local(checked, guard),
        parameters[0],
        "Boolean parameter anchor"
    );
    let hir::ExprKind::Block(branch, None) = then.kind else {
        panic!("true arm")
    };
    assert!(branch.expr.is_none());
    let tail = root.expr.unwrap();
    let mut exits = [Exit::Tail(tail), Exit::Tail(tail)];
    let mut reads = [constructors[2].0; 2];
    let mut moves = [None, None];
    let record = match case {
        Case::Initialize { .. } => {
            assert!(record_decl.init.is_none());
            assert_eq!(branch.stmts.len(), 1);
            let expression = statement_expression(&branch.stmts[0]);
            let hir::ExprKind::Assign(destination, value, _) = expression.kind else {
                panic!("conditional record assignment")
            };
            assert_eq!(local(checked, destination), record_binding);
            value
        }
        Case::Partial(field) => {
            assert_eq!(branch.stmts.len(), 1);
            moves[1] = Some(extraction(
                checked,
                declaration(&branch.stmts[0]),
                record_binding,
                field,
            ));
            record_decl.init.unwrap()
        }
        Case::Early(field) => {
            assert_eq!(branch.stmts.len(), 2);
            let taken = extraction(
                checked,
                declaration(&branch.stmts[0]),
                record_binding,
                field,
            );
            let continuation = extraction(
                checked,
                declaration(&root.stmts[5]),
                record_binding,
                FieldIdx::from_usize(1 - field.as_usize()),
            );
            moves = [Some(continuation), Some(taken)];
            reads = [continuation.0, taken.0];
            let expression = statement_expression(&branch.stmts[1]);
            let hir::ExprKind::Ret(Some(value)) = expression.kind else {
                panic!("explicit early return")
            };
            exits[1] = Exit::Return { expression, value };
            record_decl.init.unwrap()
        }
    };
    assert_eq!(checked.expr_ty(record), checked.node_type(record_binding));
    assert!(checked.expr_adjustments(record).is_empty());
    for (exit, expected) in exits.iter().zip(reads) {
        let value = exit.value();
        assert!(checked.expr_adjustments(value).is_empty());
        assert_eq!(checked.expr_ty(value), tcx.types.i32);
        let hir::ExprKind::Unary(hir::UnOp::Deref, operand) = value.kind else {
            panic!("scalar read")
        };
        assert_eq!(local(checked, operand), expected, "selected source owner");
    }
    Source {
        case,
        parameters: parameters.try_into().unwrap(),
        root,
        branch,
        constructors,
        record,
        record_binding,
        guard,
        moves,
        exits,
        reads,
    }
}
fn extraction(
    checked: &TypeckResults<'_>,
    declaration: &hir::LetStmt<'_>,
    record: HirId,
    field: FieldIdx,
) -> (HirId, FieldIdx) {
    let value = declaration.init.unwrap();
    assert!(checked.expr_adjustments(value).is_empty());
    let hir::ExprKind::Field(base, _) = value.kind else {
        panic!("field extraction")
    };
    assert_eq!(local(checked, base), record);
    assert_eq!(
        checked.field_index(value.hir_id),
        field,
        "selected source field"
    );
    (binding(declaration.pat), field)
}
pub(super) fn local(checked: &TypeckResults<'_>, expression: &hir::Expr<'_>) -> HirId {
    assert!(checked.expr_adjustments(expression).is_empty());
    let hir::ExprKind::Path(ref path) = expression.kind else {
        panic!("local path")
    };
    let Res::Local(id) = checked.qpath_res(path, expression.hir_id) else {
        panic!("local identity")
    };
    id
}
fn binding(pattern: &hir::Pat<'_>) -> HirId {
    let hir::PatKind::Binding(rustc_ast::BindingMode::NONE, id, _, None) = pattern.kind else {
        panic!("immutable simple binding")
    };
    id
}
fn declaration<'tcx>(statement: &'tcx hir::Stmt<'tcx>) -> &'tcx hir::LetStmt<'tcx> {
    let hir::StmtKind::Let(declaration) = statement.kind else {
        panic!("let declaration")
    };
    assert!(declaration.els.is_none());
    binding(declaration.pat);
    declaration
}
fn statement_expression<'tcx>(statement: &'tcx hir::Stmt<'tcx>) -> &'tcx hir::Expr<'tcx> {
    let (hir::StmtKind::Semi(expression) | hir::StmtKind::Expr(expression)) = statement.kind else {
        panic!("expression statement")
    };
    expression
}
