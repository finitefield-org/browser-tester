use super::script_ast_expr::*;
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EventMethod {
    PreventDefault,
    StopPropagation,
    StopImmediatePropagation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeTreeMethod {
    After,
    Append,
    AppendChild,
    Before,
    ReplaceWith,
    Prepend,
    RemoveChild,
    InsertBefore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InsertAdjacentPosition {
    BeforeBegin,
    AfterBegin,
    BeforeEnd,
    AfterEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListenerRegistrationOp {
    Add,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VarDeclKind {
    Var,
    Let,
    Const,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClassMethodKind {
    Method,
    Getter,
    Setter,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClassMethodDecl {
    pub(crate) name: String,
    pub(crate) is_private: bool,
    pub(crate) is_static: bool,
    pub(crate) handler: ScriptHandler,
    pub(crate) is_async: bool,
    pub(crate) is_generator: bool,
    pub(crate) kind: ClassMethodKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClassFieldDecl {
    pub(crate) name: String,
    pub(crate) computed_name: Option<Expr>,
    pub(crate) is_private: bool,
    pub(crate) is_static: bool,
    pub(crate) initializer: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ClassStaticInitializerDecl {
    Field(usize),
    Block(ScriptHandler),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SwitchClause {
    pub(crate) test: Option<Expr>,
    pub(crate) stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportBinding {
    pub(crate) imported: String,
    pub(crate) local: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Stmt {
    ImportDecl {
        specifier: String,
        default_binding: Option<String>,
        namespace_binding: Option<String>,
        named_bindings: Vec<ImportBinding>,
        attribute_type: Option<String>,
    },
    VarDecl {
        name: String,
        kind: VarDeclKind,
        expr: Expr,
    },
    ExportDecl {
        declaration: Box<Stmt>,
        bindings: Vec<(String, String)>,
    },
    ExportNamed {
        bindings: Vec<(String, String)>,
    },
    ExportDefaultExpr {
        expr: Expr,
    },
    FunctionDecl {
        name: String,
        handler: ScriptHandler,
        is_async: bool,
        is_generator: bool,
    },
    ClassDecl {
        name: String,
        super_class: Option<Expr>,
        constructor: Option<ScriptHandler>,
        fields: Vec<ClassFieldDecl>,
        methods: Vec<ClassMethodDecl>,
        static_initializers: Vec<ClassStaticInitializerDecl>,
    },
    PrivateAssign {
        target: Expr,
        member: String,
        expr: Expr,
    },
    Label {
        name: String,
        stmt: Box<Stmt>,
    },
    Block {
        stmts: Vec<Stmt>,
    },
    VarAssign {
        name: String,
        op: VarAssignOp,
        expr: Expr,
    },
    VarUpdate {
        name: String,
        delta: i8,
    },
    ArrayDestructureAssign {
        pattern: ArrayDestructurePattern,
        expr: Expr,
        decl_kind: Option<VarDeclKind>,
    },
    ObjectDestructureAssign {
        pattern: ObjectDestructurePattern,
        expr: Expr,
        decl_kind: Option<VarDeclKind>,
    },
    ObjectAssign {
        target: String,
        path: Vec<Expr>,
        op: VarAssignOp,
        expr: Expr,
    },
    FormDataAppend {
        target_var: String,
        name: Expr,
        value: Expr,
        filename: Option<Expr>,
    },
    DomAssign {
        target: DomQuery,
        prop: DomProp,
        op: VarAssignOp,
        expr: Expr,
    },
    ClassListCall {
        target: DomQuery,
        optional: bool,
        method: ClassListMethod,
        class_names: Vec<Expr>,
        force: Option<Expr>,
    },
    ClassListForEach {
        target: DomQuery,
        optional: bool,
        item_var: String,
        index_var: Option<String>,
        body: Vec<Stmt>,
    },
    DomSetAttribute {
        target: DomQuery,
        name: String,
        value: Expr,
    },
    DomRemoveAttribute {
        target: DomQuery,
        name: String,
    },
    NodeTreeMutation {
        target: DomQuery,
        method: NodeTreeMethod,
        child: Expr,
        reference: Option<Expr>,
    },
    InsertAdjacentElement {
        target: DomQuery,
        position: InsertAdjacentPosition,
        node: Expr,
    },
    InsertAdjacentText {
        target: DomQuery,
        position: InsertAdjacentPosition,
        text: Expr,
    },
    InsertAdjacentHTML {
        target: DomQuery,
        position: Expr,
        html: Expr,
    },
    SetTimeout {
        handler: TimerInvocation,
        delay_ms: Expr,
    },
    SetInterval {
        handler: TimerInvocation,
        delay_ms: Expr,
    },
    QueueMicrotask {
        handler: ScriptHandler,
    },
    ClearTimeout {
        timer_id: Expr,
    },
    NodeRemove {
        target: DomQuery,
    },
    ForEach {
        target: Option<DomQuery>,
        selector: String,
        item_var: String,
        index_var: Option<String>,
        body: Vec<Stmt>,
    },
    ArrayForEach {
        target: String,
        callback: ScriptHandler,
    },
    ArrayForEachExpr {
        target: Expr,
        callback: ScriptHandler,
    },
    For {
        init: Vec<Stmt>,
        cond: Option<Expr>,
        post: Vec<Stmt>,
        body: Vec<Stmt>,
    },
    ForIn {
        item_var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    ForOf {
        item_var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    ForAwaitOf {
        item_var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Switch {
        expr: Expr,
        clauses: Vec<SwitchClause>,
    },
    Empty,
    Debugger,
    Break {
        label: Option<String>,
    },
    Continue {
        label: Option<String>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Try {
        try_stmts: Vec<Stmt>,
        catch_binding: Option<CatchBinding>,
        catch_stmts: Option<Vec<Stmt>>,
        finally_stmts: Option<Vec<Stmt>>,
    },
    Throw {
        value: Expr,
    },
    Return {
        value: Option<Expr>,
    },
    If {
        cond: Expr,
        then_stmts: Vec<Stmt>,
        else_stmts: Vec<Stmt>,
    },
    EventCall {
        event_var: String,
        method: EventMethod,
    },
    ListenerMutation {
        target: DomQuery,
        op: ListenerRegistrationOp,
        event_type: Expr,
        capture: bool,
        is_arrow: bool,
        handler: ScriptHandler,
    },
    DispatchEvent {
        target: DomQuery,
        event_type: Expr,
    },
    DomMethodCall {
        target: DomQuery,
        method: DomMethod,
        arg: Option<Expr>,
    },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ArrayDestructureBinding {
    pub(crate) target: String,
    pub(crate) default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ArrayDestructurePattern {
    pub(crate) items: Vec<Option<ArrayDestructureBinding>>,
    pub(crate) rest: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ObjectDestructureBinding {
    pub(crate) source: String,
    pub(crate) target: String,
    pub(crate) default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ObjectDestructurePattern {
    pub(crate) bindings: Vec<ObjectDestructureBinding>,
    pub(crate) rest: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CatchBinding {
    Identifier(String),
    ArrayPattern(Vec<Option<String>>),
    ObjectPattern(Vec<(String, String)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExecFlow {
    Continue,
    Break(Option<String>),
    ContinueLoop(Option<String>),
    Return,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DomMethod {
    Focus,
    Blur,
    Click,
    ScrollIntoView,
    Submit,
    RequestSubmit,
    Reset,
    Show,
    ShowModal,
    Close,
    RequestClose,
}
