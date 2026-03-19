use crate::ScriptFunction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Program {
    pub(crate) statements: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Statement {
    VariableDeclaration { name: String, value: Expr },
    Assignment { target: AssignTarget, value: Expr },
    Expression(Expr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AssignTarget {
    Property { object: Expr, property: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Expr {
    Identifier(String),
    String(String),
    Number(String),
    Boolean(bool),
    Null,
    Undefined,
    Member { object: Box<Expr>, property: String },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    BinaryAdd { left: Box<Expr>, right: Box<Expr> },
    ArrowFunction(ScriptFunction),
}
