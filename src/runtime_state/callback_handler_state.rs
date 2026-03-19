use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ScriptHandler {
    pub(crate) params: Vec<FunctionParam>,
    pub(crate) stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FunctionParam {
    pub(crate) name: String,
    pub(crate) default: Option<Expr>,
    pub(crate) is_rest: bool,
}

impl ScriptHandler {
    pub(crate) fn first_event_param(&self) -> Option<&str> {
        self.params.first().map(|param| param.name.as_str())
    }

    pub(crate) fn listener_callback_reference(&self) -> Option<&str> {
        if self.params.len() != 1 || self.stmts.len() != 1 {
            return None;
        }
        let event_param = self.params[0].name.as_str();
        match &self.stmts[0] {
            Stmt::Expr(Expr::FunctionCall { target, args }) if args.len() == 1 => match &args[0] {
                Expr::Var(arg_name) if arg_name == event_param => Some(target.as_str()),
                _ => None,
            },
            Stmt::Expr(Expr::MemberCall {
                target,
                member,
                args,
                optional: false,
                optional_call: false,
            }) if member == "call" && args.len() == 2 => match (&**target, &args[0], &args[1]) {
                (Expr::Var(callback_name), Expr::Var(this_name), Expr::Var(arg_name))
                    if this_name == "this" && arg_name == event_param =>
                {
                    Some(callback_name.as_str())
                }
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TimerCallback {
    Inline(ScriptHandler),
    Reference(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TimerInvocation {
    pub(crate) callback: TimerCallback,
    pub(crate) args: Vec<Expr>,
}
