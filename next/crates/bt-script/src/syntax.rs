use crate::ScriptFunction;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Program {
    pub(crate) statements: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Statement {
    VariableDeclaration {
        name: String,
        value: Expr,
    },
    FunctionDeclaration {
        name: String,
        function: ScriptFunction,
    },
    Return(Option<Expr>),
    Break,
    Continue,
    If {
        condition: Expr,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
    For {
        init: Option<Box<Statement>>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Statement>,
    },
    ForIn {
        binding: String,
        iterable: Expr,
        body: Vec<Statement>,
    },
    ForOf {
        binding: String,
        iterable: Expr,
        body: Vec<Statement>,
    },
    TryCatch {
        try_body: Vec<Statement>,
        catch_binding: String,
        catch_body: Vec<Statement>,
    },
    Throw(Expr),
    Assignment {
        target: AssignTarget,
        value: Expr,
    },
    Expression(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AssignTarget {
    Identifier(String),
    Property {
        object: Box<Expr>,
        property: String,
    },
    ComputedProperty {
        object: Box<Expr>,
        property: Box<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ObjectPropertyName {
    Static(String),
    Computed(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ObjectProperty {
    KeyValue {
        name: ObjectPropertyName,
        value: Expr,
    },
    Method {
        name: ObjectPropertyName,
        function: ScriptFunction,
    },
    Getter {
        name: ObjectPropertyName,
        function: ScriptFunction,
    },
    Setter {
        name: ObjectPropertyName,
        function: ScriptFunction,
    },
    Spread(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ArrayElement {
    Expression(Expr),
    Spread(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expr {
    Identifier(String),
    String(String),
    Number(String),
    RegexLiteral {
        pattern: String,
        flags: String,
    },
    Boolean(bool),
    Null,
    Undefined,
    Member {
        object: Box<Expr>,
        property: String,
    },
    OptionalMember {
        object: Box<Expr>,
        property: String,
    },
    ComputedMember {
        object: Box<Expr>,
        property: Box<Expr>,
    },
    OptionalMemberCall {
        object: Box<Expr>,
        property: String,
        args: Vec<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    ArrayLiteral(Vec<ArrayElement>),
    ObjectLiteral(Vec<ObjectProperty>),
    New {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Assignment {
        target: AssignTarget,
        value: Box<Expr>,
        operator: AssignmentOperator,
    },
    BinaryAdd {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    BinarySub {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    BinaryMul {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    BinaryDiv {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    BinaryRem {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    LogicalAnd {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    LogicalOr {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    NullishCoalesce {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Equality {
        left: Box<Expr>,
        right: Box<Expr>,
        negated: bool,
        strict: bool,
    },
    Comparison {
        left: Box<Expr>,
        right: Box<Expr>,
        operator: ComparisonOperator,
    },
    UnaryNeg(Box<Expr>),
    UnaryNot(Box<Expr>),
    TypeOf(Box<Expr>),
    Void(Box<Expr>),
    Update {
        target: AssignTarget,
        increment: bool,
        prefix: bool,
    },
    Conditional {
        condition: Box<Expr>,
        consequent: Box<Expr>,
        alternate: Box<Expr>,
    },
    ArrowFunction(ScriptFunction),
    FunctionExpression(ScriptFunction),
    Spread(Box<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ComparisonOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    InstanceOf,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum AssignmentOperator {
    Assign,
    AddAssign,
}
