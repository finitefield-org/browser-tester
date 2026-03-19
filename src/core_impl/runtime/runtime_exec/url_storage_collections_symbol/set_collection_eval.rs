use super::*;

impl Harness {
    fn set_instance_method_name(method: SetInstanceMethod) -> &'static str {
        match method {
            SetInstanceMethod::Add => "add",
            SetInstanceMethod::Union => "union",
            SetInstanceMethod::Intersection => "intersection",
            SetInstanceMethod::Difference => "difference",
            SetInstanceMethod::SymmetricDifference => "symmetricDifference",
            SetInstanceMethod::IsDisjointFrom => "isDisjointFrom",
            SetInstanceMethod::IsSubsetOf => "isSubsetOf",
            SetInstanceMethod::IsSupersetOf => "isSupersetOf",
        }
    }

    pub(crate) fn eval_set_construct(
        &mut self,
        iterable: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "Set constructor must be called with new".into(),
            ));
        }

        let set = Rc::new(RefCell::new(SetValue {
            values: Vec::new(),
            properties: ObjectValue::default(),
        }));

        let Some(iterable) = iterable else {
            return Ok(Value::Set(set));
        };

        let iterable = self.eval_expr(iterable, env, event_param, event)?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::Set(set));
        }

        let values = self.array_like_values_from_value(&iterable)?;
        for value in values {
            self.set_add_value(&mut set.borrow_mut(), value);
        }
        Ok(Value::Set(set))
    }

    pub(crate) fn eval_weak_set_construct(
        &mut self,
        iterable: &Option<Box<Expr>>,
        called_with_new: bool,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        if !called_with_new {
            return Err(Error::ScriptRuntime(
                "WeakSet constructor must be called with new".into(),
            ));
        }

        let weak_set = Rc::new(RefCell::new(WeakSetValue {
            values: Vec::new(),
            properties: ObjectValue::default(),
        }));

        let Some(iterable) = iterable else {
            return Ok(Value::WeakSet(weak_set));
        };

        let iterable = self.eval_expr(iterable, env, event_param, event)?;
        if matches!(iterable, Value::Undefined | Value::Null) {
            return Ok(Value::WeakSet(weak_set));
        }

        match iterable {
            Value::WeakSet(source) => {
                let source = source.borrow();
                weak_set.borrow_mut().values = source.values.clone();
            }
            other => {
                let values = self.array_like_values_from_value(&other)?;
                for value in values {
                    Self::ensure_weak_set_value(&value)?;
                    self.weak_set_add_value(&mut weak_set.borrow_mut(), value);
                }
            }
        }
        Ok(Value::WeakSet(weak_set))
    }

    pub(crate) fn eval_set_method(
        &mut self,
        target: &str,
        method: SetInstanceMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let target_value =
            self.eval_expr(&Expr::Var(target.to_string()), env, event_param, event)?;

        if let Some(value) = self.eval_cache_storage_set_method_dispatch(
            &target_value,
            method,
            args,
            env,
            event_param,
            event,
        )? {
            return Ok(value);
        }

        let evaluated_args = args
            .iter()
            .map(|arg| self.eval_expr(arg, env, event_param, event))
            .collect::<Result<Vec<_>>>()?;
        let member = Self::set_instance_method_name(method);

        let supports_direct_dispatch = match &target_value {
            Value::Set(set) => {
                let set_ref = set.borrow();
                Self::object_get_entry(&set_ref.properties, member).is_none()
                    && Self::object_get_entry(&set_ref.properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
                        .is_none()
            }
            Value::WeakSet(weak_set) => {
                let weak_set_ref = weak_set.borrow();
                Self::object_get_entry(&weak_set_ref.properties, member).is_none()
                    && Self::object_get_entry(
                        &weak_set_ref.properties,
                        INTERNAL_OBJECT_PROTOTYPE_KEY,
                    )
                    .is_none()
            }
            _ => false,
        };
        if !supports_direct_dispatch {
            let callee = self.object_property_from_value(&target_value, member)?;
            if !matches!(callee, Value::Undefined | Value::Null)
                && !Self::is_builtin_placeholder_callable(&callee)
            {
                return self.execute_named_member_with_receiver(
                    &callee,
                    &target_value,
                    member,
                    &evaluated_args,
                    env,
                    event,
                );
            }
        }

        if let Value::WeakSet(weak_set) = &target_value {
            let weak_set = weak_set.clone();
            return match method {
                SetInstanceMethod::Add => {
                    if evaluated_args.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "WeakSet.add requires exactly one argument".into(),
                        ));
                    }
                    Self::ensure_weak_set_value(&evaluated_args[0])?;
                    self.weak_set_add_value(&mut weak_set.borrow_mut(), evaluated_args[0].clone());
                    Ok(Value::WeakSet(weak_set))
                }
                SetInstanceMethod::Union => Err(Error::ScriptRuntime(
                    "WeakSet.union is not a function".into(),
                )),
                SetInstanceMethod::Intersection => Err(Error::ScriptRuntime(
                    "WeakSet.intersection is not a function".into(),
                )),
                SetInstanceMethod::Difference => Err(Error::ScriptRuntime(
                    "WeakSet.difference is not a function".into(),
                )),
                SetInstanceMethod::SymmetricDifference => Err(Error::ScriptRuntime(
                    "WeakSet.symmetricDifference is not a function".into(),
                )),
                SetInstanceMethod::IsDisjointFrom => Err(Error::ScriptRuntime(
                    "WeakSet.isDisjointFrom is not a function".into(),
                )),
                SetInstanceMethod::IsSubsetOf => Err(Error::ScriptRuntime(
                    "WeakSet.isSubsetOf is not a function".into(),
                )),
                SetInstanceMethod::IsSupersetOf => Err(Error::ScriptRuntime(
                    "WeakSet.isSupersetOf is not a function".into(),
                )),
            };
        }

        let Value::Set(set) = &target_value else {
            if method == SetInstanceMethod::Add {
                return self.eval_expr(
                    &Expr::MemberCall {
                        target: Box::new(Expr::Var(target.to_string())),
                        member: "add".to_string(),
                        args: args.to_vec(),
                        optional: false,
                        optional_call: false,
                    },
                    env,
                    event_param,
                    event,
                );
            }
            return Err(Error::ScriptRuntime(format!(
                "variable '{}' is not a Set",
                target
            )));
        };
        let set = set.clone();

        match method {
            SetInstanceMethod::Add => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.add requires exactly one argument".into(),
                    ));
                }
                self.set_add_value(&mut set.borrow_mut(), evaluated_args[0].clone());
                Ok(Value::Set(set))
            }
            SetInstanceMethod::Union => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.union requires exactly one argument".into(),
                    ));
                }
                let other_keys = self.set_like_keys_snapshot(&evaluated_args[0])?;
                let mut out = SetValue {
                    values: set.borrow().values.clone(),
                    properties: ObjectValue::default(),
                };
                for key in other_keys {
                    self.set_add_value(&mut out, key);
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::Intersection => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.intersection requires exactly one argument".into(),
                    ));
                }
                let snapshot = set.borrow().values.clone();
                let mut out = SetValue {
                    values: Vec::new(),
                    properties: ObjectValue::default(),
                };
                for value in snapshot {
                    if self.set_like_has_value(&evaluated_args[0], &value)? {
                        self.set_add_value(&mut out, value);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::Difference => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.difference requires exactly one argument".into(),
                    ));
                }
                let snapshot = set.borrow().values.clone();
                let mut out = SetValue {
                    values: Vec::new(),
                    properties: ObjectValue::default(),
                };
                for value in snapshot {
                    if !self.set_like_has_value(&evaluated_args[0], &value)? {
                        self.set_add_value(&mut out, value);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::SymmetricDifference => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.symmetricDifference requires exactly one argument".into(),
                    ));
                }
                let other_keys = self.set_like_keys_snapshot(&evaluated_args[0])?;
                let mut out = SetValue {
                    values: set.borrow().values.clone(),
                    properties: ObjectValue::default(),
                };
                for key in other_keys {
                    if let Some(index) = self.set_value_index(&out, &key) {
                        out.values.remove(index);
                    } else {
                        out.values.push(key);
                    }
                }
                Ok(Value::Set(Rc::new(RefCell::new(out))))
            }
            SetInstanceMethod::IsDisjointFrom => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.isDisjointFrom requires exactly one argument".into(),
                    ));
                }
                for value in &set.borrow().values {
                    if self.set_like_has_value(&evaluated_args[0], value)? {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            SetInstanceMethod::IsSubsetOf => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.isSubsetOf requires exactly one argument".into(),
                    ));
                }
                for value in &set.borrow().values {
                    if !self.set_like_has_value(&evaluated_args[0], value)? {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
            SetInstanceMethod::IsSupersetOf => {
                if evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Set.isSupersetOf requires exactly one argument".into(),
                    ));
                }
                for value in self.set_like_keys_snapshot(&evaluated_args[0])? {
                    if self.set_value_index(&set.borrow(), &value).is_none() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }
        }
    }
}
