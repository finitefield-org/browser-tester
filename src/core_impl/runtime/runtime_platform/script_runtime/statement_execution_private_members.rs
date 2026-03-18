use super::*;

impl Harness {
    fn private_accessor_get_key() -> &'static str {
        "\u{0}\u{0}bt_private_accessor:get"
    }

    fn private_accessor_set_key() -> &'static str {
        "\u{0}\u{0}bt_private_accessor:set"
    }

    fn private_accessor_value(getter: Value, setter: Value) -> Value {
        Self::new_object_value(vec![
            (Self::private_accessor_get_key().to_string(), getter),
            (Self::private_accessor_set_key().to_string(), setter),
        ])
    }

    fn private_accessor_parts(value: &Value) -> (Value, Value) {
        let Value::Object(entries) = value else {
            return (Value::Undefined, Value::Undefined);
        };
        let entries = entries.borrow();
        (
            Self::object_get_entry(&entries, Self::private_accessor_get_key())
                .unwrap_or(Value::Undefined),
            Self::object_get_entry(&entries, Self::private_accessor_set_key())
                .unwrap_or(Value::Undefined),
        )
    }

    fn private_instance_key(value: &Value) -> Option<usize> {
        match value {
            Value::Object(object) => Some(Rc::as_ptr(object) as usize),
            _ => None,
        }
    }

    fn private_static_key(value: &Value) -> Option<usize> {
        match value {
            Value::Function(function) => Some(function.function_id),
            _ => None,
        }
    }

    fn private_read_brand_error(name: &str) -> Error {
        Error::ScriptRuntime(format!(
            "Cannot read private member #{name} from an object whose class did not declare it"
        ))
    }

    fn private_write_brand_error(name: &str) -> Error {
        Error::ScriptRuntime(format!(
            "Cannot write private member #{name} to an object whose class did not declare it"
        ))
    }

    pub(crate) fn resolve_private_binding(&self, name: &str) -> Result<PrivateBindingRuntime> {
        for bindings in self.script_runtime.private_binding_stack.iter().rev() {
            if let Some(binding) = bindings.get(name) {
                return Ok(binding.clone());
            }
        }
        Err(Error::ScriptRuntime(format!(
            "private identifier '#{name}' is not declared"
        )))
    }

    fn private_slot_exists(&self, binding: &PrivateBindingRuntime, receiver: &Value) -> bool {
        if binding.is_static {
            let Some(class_id) = Self::private_static_key(receiver) else {
                return false;
            };
            return self
                .script_runtime
                .private_static_slots
                .get(&class_id)
                .is_some_and(|slots| slots.contains_key(&binding.slot_id));
        }
        let Some(instance_id) = Self::private_instance_key(receiver) else {
            return false;
        };
        self.script_runtime
            .private_instance_slots
            .get(&instance_id)
            .is_some_and(|slots| slots.contains_key(&binding.slot_id))
    }

    fn private_slot_read(
        &self,
        binding: &PrivateBindingRuntime,
        receiver: &Value,
    ) -> Option<Value> {
        if binding.is_static {
            let class_id = Self::private_static_key(receiver)?;
            return self
                .script_runtime
                .private_static_slots
                .get(&class_id)
                .and_then(|slots| slots.get(&binding.slot_id).cloned());
        }
        let instance_id = Self::private_instance_key(receiver)?;
        self.script_runtime
            .private_instance_slots
            .get(&instance_id)
            .and_then(|slots| slots.get(&binding.slot_id).cloned())
    }

    fn private_slot_initialize_value(
        &mut self,
        binding: &PrivateBindingRuntime,
        receiver: &Value,
        value: Value,
    ) -> Result<()> {
        if binding.is_static {
            let Some(class_id) = Self::private_static_key(receiver) else {
                return Err(Self::private_write_brand_error(&binding.name));
            };
            let slots = self
                .script_runtime
                .private_static_slots
                .entry(class_id)
                .or_default();
            if slots.contains_key(&binding.slot_id) {
                return Err(Error::ScriptRuntime(
                    "Initializing an object twice is an error with private fields".into(),
                ));
            }
            slots.insert(binding.slot_id, value);
            return Ok(());
        }

        let Some(instance_id) = Self::private_instance_key(receiver) else {
            return Err(Self::private_write_brand_error(&binding.name));
        };
        let slots = self
            .script_runtime
            .private_instance_slots
            .entry(instance_id)
            .or_default();
        if slots.contains_key(&binding.slot_id) {
            return Err(Error::ScriptRuntime(
                "Initializing an object twice is an error with private fields".into(),
            ));
        }
        slots.insert(binding.slot_id, value);
        Ok(())
    }

    fn private_slot_initialize_accessor(
        &mut self,
        binding: &PrivateBindingRuntime,
        receiver: &Value,
        getter: Option<Value>,
        setter: Option<Value>,
    ) -> Result<()> {
        let slot_map = if binding.is_static {
            let Some(class_id) = Self::private_static_key(receiver) else {
                return Err(Self::private_write_brand_error(&binding.name));
            };
            self.script_runtime
                .private_static_slots
                .entry(class_id)
                .or_default()
        } else {
            let Some(instance_id) = Self::private_instance_key(receiver) else {
                return Err(Self::private_write_brand_error(&binding.name));
            };
            self.script_runtime
                .private_instance_slots
                .entry(instance_id)
                .or_default()
        };

        if let Some(existing) = slot_map.get(&binding.slot_id).cloned() {
            let (mut current_getter, mut current_setter) = Self::private_accessor_parts(&existing);
            if let Some(next_getter) = getter {
                if !matches!(&current_getter, Value::Undefined) {
                    return Err(Error::ScriptRuntime(
                        "Initializing an object twice is an error with private fields".into(),
                    ));
                }
                current_getter = next_getter;
            }
            if let Some(next_setter) = setter {
                if !matches!(&current_setter, Value::Undefined) {
                    return Err(Error::ScriptRuntime(
                        "Initializing an object twice is an error with private fields".into(),
                    ));
                }
                current_setter = next_setter;
            }
            slot_map.insert(
                binding.slot_id,
                Self::private_accessor_value(current_getter, current_setter),
            );
            return Ok(());
        }

        slot_map.insert(
            binding.slot_id,
            Self::private_accessor_value(
                getter.unwrap_or(Value::Undefined),
                setter.unwrap_or(Value::Undefined),
            ),
        );
        Ok(())
    }

    fn private_slot_write(
        &mut self,
        binding: &PrivateBindingRuntime,
        receiver: &Value,
        value: Value,
    ) -> Result<()> {
        if binding.is_static {
            let Some(class_id) = Self::private_static_key(receiver) else {
                return Err(Self::private_write_brand_error(&binding.name));
            };
            let Some(slots) = self.script_runtime.private_static_slots.get_mut(&class_id) else {
                return Err(Self::private_write_brand_error(&binding.name));
            };
            if !slots.contains_key(&binding.slot_id) {
                return Err(Self::private_write_brand_error(&binding.name));
            }
            slots.insert(binding.slot_id, value);
            return Ok(());
        }

        let Some(instance_id) = Self::private_instance_key(receiver) else {
            return Err(Self::private_write_brand_error(&binding.name));
        };
        let Some(slots) = self
            .script_runtime
            .private_instance_slots
            .get_mut(&instance_id)
        else {
            return Err(Self::private_write_brand_error(&binding.name));
        };
        if !slots.contains_key(&binding.slot_id) {
            return Err(Self::private_write_brand_error(&binding.name));
        }
        slots.insert(binding.slot_id, value);
        Ok(())
    }

    pub(crate) fn apply_private_initializer_to_receiver(
        &mut self,
        initializer: &PrivateInitializerRuntime,
        receiver: &Value,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        match initializer.binding.kind {
            PrivateBindingKind::Field => {
                let value = if let Some(expr) = initializer.initializer.as_ref() {
                    self.eval_expr(expr, env, event_param, event)?
                } else {
                    Value::Undefined
                };
                self.private_slot_initialize_value(&initializer.binding, receiver, value)
            }
            PrivateBindingKind::Method => {
                let value = initializer.value.clone().unwrap_or(Value::Undefined);
                self.private_slot_initialize_value(&initializer.binding, receiver, value)
            }
            PrivateBindingKind::Accessor => self.private_slot_initialize_accessor(
                &initializer.binding,
                receiver,
                initializer.value.clone(),
                initializer.setter_value.clone(),
            ),
        }
    }

    fn define_public_field_on_receiver(
        &mut self,
        receiver: &Value,
        name: &str,
        value: Value,
    ) -> Result<()> {
        match receiver {
            Value::Object(object) => {
                Self::object_set_entry(&mut object.borrow_mut(), name.to_string(), value);
                Ok(())
            }
            Value::Array(array) => {
                Self::set_array_property(array, name.to_string(), value);
                Ok(())
            }
            Value::Function(function) => {
                let entries = self
                    .script_runtime
                    .function_public_properties
                    .entry(function.function_id)
                    .or_default();
                Self::object_set_entry(entries, name.to_string(), value);
                Ok(())
            }
            Value::Map(map) => {
                Self::object_set_entry(&mut map.borrow_mut().properties, name.to_string(), value);
                Ok(())
            }
            Value::WeakMap(weak_map) => {
                Self::object_set_entry(
                    &mut weak_map.borrow_mut().properties,
                    name.to_string(),
                    value,
                );
                Ok(())
            }
            Value::Set(set) => {
                Self::object_set_entry(&mut set.borrow_mut().properties, name.to_string(), value);
                Ok(())
            }
            Value::WeakSet(weak_set) => {
                Self::object_set_entry(
                    &mut weak_set.borrow_mut().properties,
                    name.to_string(),
                    value,
                );
                Ok(())
            }
            Value::RegExp(regex) => {
                Self::object_set_entry(&mut regex.borrow_mut().properties, name.to_string(), value);
                Ok(())
            }
            _ => Err(Error::ScriptRuntime(
                "class field target must be an object".into(),
            )),
        }
    }

    pub(crate) fn apply_constructor_instance_initializer_to_receiver(
        &mut self,
        initializer: &ConstructorInstanceInitializerRuntime,
        receiver: &Value,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<()> {
        match initializer {
            ConstructorInstanceInitializerRuntime::Private(private) => self
                .apply_private_initializer_to_receiver(private, receiver, env, event_param, event),
            ConstructorInstanceInitializerRuntime::Public(public) => {
                let value = if let Some(expr) = public.initializer.as_ref() {
                    self.eval_expr(expr, env, event_param, event)?
                } else {
                    Value::Undefined
                };
                self.define_public_field_on_receiver(receiver, &public.name, value)
            }
        }
    }

    pub(crate) fn private_has_member(&self, member: &str, receiver: &Value) -> Result<bool> {
        if Self::is_primitive_value(receiver) {
            return Err(Error::ScriptRuntime(
                "right-hand side of private in must be an object".into(),
            ));
        }
        let binding = self.resolve_private_binding(member)?;
        Ok(self.private_slot_exists(&binding, receiver))
    }

    pub(crate) fn private_get_member(
        &mut self,
        member: &str,
        receiver: &Value,
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        let binding = self.resolve_private_binding(member)?;
        let Some(slot_value) = self.private_slot_read(&binding, receiver) else {
            return Err(Self::private_read_brand_error(member));
        };
        match binding.kind {
            PrivateBindingKind::Accessor => {
                let (getter, _) = Self::private_accessor_parts(&slot_value);
                if matches!(getter, Value::Null | Value::Undefined) {
                    return Ok(Value::Undefined);
                }
                if !self.is_callable_value(&getter) {
                    return Err(Error::ScriptRuntime(
                        "private accessor getter is not callable".into(),
                    ));
                }
                self.execute_callable_value_with_this_and_env(
                    &getter,
                    &[],
                    event,
                    Some(env),
                    Some(receiver.clone()),
                )
            }
            _ => Ok(slot_value),
        }
    }

    pub(crate) fn private_call_member(
        &mut self,
        member: &str,
        receiver: &Value,
        args: &[Value],
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<Value> {
        let callee = self.private_get_member(member, receiver, env, event)?;
        self.execute_callable_value_with_this_and_env(
            &callee,
            args,
            event,
            Some(env),
            Some(receiver.clone()),
        )
        .map_err(|err| match err {
            Error::ScriptRuntime(msg) if msg == "callback is not a function" => {
                Error::ScriptRuntime(format!("'{}' is not a function", member))
            }
            other => other,
        })
    }

    pub(crate) fn private_set_member(
        &mut self,
        member: &str,
        receiver: &Value,
        value: Value,
        env: &HashMap<String, Value>,
        event: &EventState,
    ) -> Result<()> {
        let binding = self.resolve_private_binding(member)?;
        let Some(slot_value) = self.private_slot_read(&binding, receiver) else {
            return Err(Self::private_write_brand_error(member));
        };
        match binding.kind {
            PrivateBindingKind::Field => self.private_slot_write(&binding, receiver, value),
            PrivateBindingKind::Method => Err(Error::ScriptRuntime(format!(
                "Cannot write to private method #{member}"
            ))),
            PrivateBindingKind::Accessor => {
                let (_, setter) = Self::private_accessor_parts(&slot_value);
                if matches!(setter, Value::Null | Value::Undefined) {
                    return Err(Error::ScriptRuntime(format!(
                        "Cannot set private member #{member} without a setter"
                    )));
                }
                if !self.is_callable_value(&setter) {
                    return Err(Error::ScriptRuntime(
                        "private accessor setter is not callable".into(),
                    ));
                }
                let _ = self.execute_callable_value_with_this_and_env(
                    &setter,
                    &[value],
                    event,
                    Some(env),
                    Some(receiver.clone()),
                )?;
                Ok(())
            }
        }
    }
}
