use super::*;

impl Harness {
    pub(crate) fn is_super_target_expr(expr: &Expr) -> bool {
        matches!(expr, Expr::Var(name) if name == "super")
    }

    pub(crate) fn super_constructor_from_env(env: &HashMap<String, Value>) -> Result<Value> {
        env.get(INTERNAL_CLASS_SUPER_CONSTRUCTOR_KEY)
            .cloned()
            .ok_or_else(|| {
                Error::ScriptRuntime("super() is only valid in a derived class constructor".into())
            })
    }

    pub(crate) fn super_prototype_from_env(env: &HashMap<String, Value>) -> Result<Value> {
        env.get(INTERNAL_CLASS_SUPER_PROTOTYPE_KEY)
            .cloned()
            .ok_or_else(|| {
                Error::ScriptRuntime("super property access is only valid in a class method".into())
            })
    }

    pub(crate) fn super_this_from_env(env: &HashMap<String, Value>) -> Result<Value> {
        match env.get("this").cloned().unwrap_or(Value::Undefined) {
            Value::Null | Value::Undefined => Err(Error::ScriptRuntime(
                "super requires an initialized this value".into(),
            )),
            value => Ok(value),
        }
    }
}
