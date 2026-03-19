use super::*;

impl Harness {
    fn set_super_assignment_property(
        &mut self,
        super_base: &Value,
        receiver: &Value,
        key_value: &Value,
        value: Value,
        target: &str,
        env: &mut HashMap<String, Value>,
        event: &EventState,
    ) -> Result<()> {
        let key = self.property_key_to_storage_key(key_value);
        let mut current = Some(super_base.clone());
        while let Some(container) = current {
            match container {
                Value::Object(object) => {
                    let (setter, getter, next) = {
                        let entries = object.borrow();
                        (
                            Self::object_setter_from_entries(&entries, &key),
                            Self::has_object_accessor_property(&entries, &key),
                            Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY),
                        )
                    };
                    if let Some(setter) = setter {
                        if !self.is_callable_value(&setter) {
                            return Err(Error::ScriptRuntime(
                                "object setter is not callable".into(),
                            ));
                        }
                        self.execute_callable_value_with_this_and_env(
                            &setter,
                            &[value],
                            event,
                            None,
                            Some(receiver.clone()),
                        )?;
                        return Ok(());
                    }
                    if getter {
                        return Ok(());
                    }
                    current = next;
                }
                Value::Function(function) => {
                    let (setter, getter, next) = {
                        if let Some(entries) = self
                            .script_runtime
                            .function_public_properties
                            .get(&function.function_id)
                        {
                            (
                                Self::object_setter_from_entries(entries, &key),
                                Self::has_object_accessor_property(entries, &key),
                                function.class_super_constructor.clone(),
                            )
                        } else {
                            (None, false, function.class_super_constructor.clone())
                        }
                    };
                    if let Some(setter) = setter {
                        if !self.is_callable_value(&setter) {
                            return Err(Error::ScriptRuntime(
                                "object setter is not callable".into(),
                            ));
                        }
                        self.execute_callable_value_with_this_and_env(
                            &setter,
                            &[value],
                            event,
                            None,
                            Some(receiver.clone()),
                        )?;
                        return Ok(());
                    }
                    if getter {
                        return Ok(());
                    }
                    current = next;
                }
                _ => break,
            }
        }

        self.set_object_assignment_property(receiver, key_value, value, target, env, event)
    }

    pub(crate) fn execute_object_assignment_stmt(
        &mut self,
        target: &str,
        path: &[Expr],
        op: VarAssignOp,
        expr: &Expr,
        env: &mut HashMap<String, Value>,
        event_param: &Option<String>,
        event: &mut EventState,
    ) -> Result<()> {
        if path.is_empty() {
            return Err(Error::ScriptRuntime(
                "object assignment path cannot be empty".into(),
            ));
        }

        let mut keys = Vec::with_capacity(path.len());
        for segment in path {
            keys.push(self.eval_expr(segment, env, event_param, event)?);
        }

        if target == "super" {
            let super_base = Self::super_prototype_from_env(env)?;
            let this_value = Self::super_this_from_env(env)?;

            let final_key = keys.last().ok_or_else(|| {
                Error::ScriptRuntime("object assignment key cannot be empty".into())
            })?;
            let key = self.property_key_to_storage_key(final_key);

            let mut container = super_base.clone();
            for (index, key_value) in keys.iter().take(keys.len().saturating_sub(1)).enumerate() {
                if index == 0 {
                    container = self.object_property_from_value_with_receiver(
                        &container,
                        &self.property_key_to_storage_key(key_value),
                        &this_value,
                    )?;
                } else {
                    container =
                        self.read_object_assignment_property(&container, key_value, target)?;
                }
            }

            if matches!(
                op,
                VarAssignOp::LogicalAnd | VarAssignOp::LogicalOr | VarAssignOp::Nullish
            ) {
                let previous = if keys.len() <= 1 {
                    self.object_property_from_value_with_receiver(&super_base, &key, &this_value)?
                } else {
                    self.object_property_from_value(&container, &key)?
                };
                let should_assign = match op {
                    VarAssignOp::LogicalAnd => previous.truthy(),
                    VarAssignOp::LogicalOr => !previous.truthy(),
                    VarAssignOp::Nullish => matches!(&previous, Value::Null | Value::Undefined),
                    _ => true,
                };
                if !should_assign {
                    return Ok(());
                }
            }

            let value = self.eval_expr(expr, env, event_param, event)?;
            if keys.len() <= 1 {
                self.set_super_assignment_property(
                    &super_base,
                    &this_value,
                    final_key,
                    value,
                    target,
                    env,
                    event,
                )?;
            } else {
                self.set_object_assignment_property(
                    &container, final_key, value, target, env, event,
                )?;
            }
            return Ok(());
        }

        let mut container = env
            .get(target)
            .cloned()
            .ok_or_else(|| Error::ScriptRuntime(format!("unknown variable: {}", target)))?;
        for key in keys.iter().take(keys.len().saturating_sub(1)) {
            container = self.read_object_assignment_property(&container, key, target)?;
        }

        let final_key = keys
            .last()
            .ok_or_else(|| Error::ScriptRuntime("object assignment key cannot be empty".into()))?;
        let key = self.property_key_to_storage_key(final_key);

        if matches!(
            op,
            VarAssignOp::LogicalAnd | VarAssignOp::LogicalOr | VarAssignOp::Nullish
        ) {
            let previous = self
                .object_property_from_value(&container, &key)
                .map_err(|err| match err {
                    Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                        Error::ScriptRuntime(format!(
                            "variable '{}' is not an object (key '{}')",
                            target, key
                        ))
                    }
                    other => other,
                })?;
            let should_assign = match op {
                VarAssignOp::LogicalAnd => previous.truthy(),
                VarAssignOp::LogicalOr => !previous.truthy(),
                VarAssignOp::Nullish => matches!(&previous, Value::Null | Value::Undefined),
                _ => true,
            };
            if !should_assign {
                return Ok(());
            }
        }

        let mut value = self.eval_expr(expr, env, event_param, event)?;

        let assigning_before_unload_return_value = key == "returnValue"
            && event_param.as_ref().is_some_and(|param| param == target)
            && (event.before_unload_interface
                || event.event_type.eq_ignore_ascii_case("beforeunload"));
        if assigning_before_unload_return_value {
            let return_value = value.as_string();
            event.before_unload_interface = true;
            event.before_unload_return_value = return_value.clone();
            if event.cancelable && !return_value.is_empty() {
                event.default_prevented = true;
            }
            value = Value::String(return_value);
        }

        let assigns_window_local_storage = if let Value::Object(object) = &container {
            if key == "localStorage" {
                let entries = object.borrow();
                Self::is_window_object(&entries)
            } else {
                false
            }
        } else {
            false
        };

        self.set_object_assignment_property(
            &container,
            final_key,
            value.clone(),
            target,
            env,
            event,
        )?;
        if assigns_window_local_storage {
            env.insert("localStorage".to_string(), value.clone());
            self.sync_global_binding_if_needed(env, "localStorage", &value);
        }
        Ok(())
    }
}
