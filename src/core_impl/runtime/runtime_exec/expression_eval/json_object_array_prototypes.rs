use super::*;

impl Harness {
    pub(crate) fn object_get_prototype_of_value(&mut self, value: &Value) -> Result<Value> {
        if let Value::TypedArrayConstructor(TypedArrayConstructorKind::Concrete(_)) = value {
            if let Some(prototype) = self.variant_callable_internal_prototype_value(value) {
                return Ok(prototype);
            }
            return Ok(Value::TypedArrayConstructor(
                TypedArrayConstructorKind::Abstract,
            ));
        }
        Ok(self
            .value_internal_prototype_value(value)
            .unwrap_or_else(|| Value::Object(Rc::new(RefCell::new(ObjectValue::default())))))
    }

    pub(crate) fn object_create_value(
        &mut self,
        prototype: &Value,
        properties: Option<&Value>,
    ) -> Result<Value> {
        if !matches!(prototype, Value::Null) && Self::is_primitive_value(prototype) {
            return Err(Error::ScriptRuntime(
                "Object prototype may only be an Object or null".into(),
            ));
        }

        let created = Self::new_object_value(Vec::new());
        let Value::Object(entries) = &created else {
            unreachable!("new_object_value always returns an object");
        };
        Self::set_internal_prototype(entries, prototype.clone());

        if let Some(properties) = properties
            && !matches!(properties, Value::Undefined)
        {
            let own_keys = self.reflect_own_keys_value(properties)?;
            let Value::Array(keys) = own_keys else {
                unreachable!("Reflect.ownKeys returns an array");
            };
            for key in keys.borrow().iter() {
                let storage_key = self.property_key_to_storage_key(key);
                let descriptor =
                    self.object_get_own_property_descriptor_value(properties, &storage_key)?;
                if !matches!(descriptor, Value::Object(_)) {
                    continue;
                }
                let enumerable = self
                    .object_property_from_value(&descriptor, "enumerable")?
                    .truthy();
                if !enumerable {
                    continue;
                }
                let property_descriptor =
                    self.object_property_from_value(properties, &storage_key)?;
                self.object_define_property_value(&created, &storage_key, &property_descriptor)?;
            }
        }

        Ok(created)
    }

    fn object_set_prototype_would_cycle(&mut self, target: &Value, prototype: &Value) -> bool {
        let mut current = Some(prototype.clone());
        let mut hops = 0usize;
        while let Some(value) = current {
            if self.strict_equal(target, &value) {
                return true;
            }
            hops += 1;
            if hops > 256 {
                break;
            }
            current = self.value_internal_prototype_value(&value);
        }
        false
    }

    fn set_object_like_internal_prototype(
        &mut self,
        target: &Value,
        prototype: Value,
    ) -> Result<()> {
        match target {
            Value::Object(entries) => {
                Self::object_set_entry(
                    &mut entries.borrow_mut(),
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::Function(function) => {
                let entries = self
                    .script_runtime
                    .function_public_properties
                    .entry(function.function_id)
                    .or_default();
                Self::object_set_entry(
                    entries,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::Array(values) => {
                Self::object_set_entry(
                    &mut values.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::Map(map) => {
                Self::object_set_entry(
                    &mut map.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::WeakMap(map) => {
                Self::object_set_entry(
                    &mut map.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::Set(set) => {
                Self::object_set_entry(
                    &mut set.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::WeakSet(set) => {
                Self::object_set_entry(
                    &mut set.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::RegExp(regex) => {
                Self::object_set_entry(
                    &mut regex.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::TypedArray(values) => {
                Self::object_set_entry(
                    &mut values.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::NodeList(nodes) => {
                Self::object_set_entry(
                    &mut nodes.borrow_mut().properties,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            Value::UrlConstructor => {
                Self::object_set_entry(
                    &mut self.browser_apis.url_constructor_properties.borrow_mut(),
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            _ if Self::variant_callable_public_storage_key(target).is_some() => {
                let storage_key = Self::variant_callable_public_storage_key(target)
                    .expect("checked variant callable storage key");
                let entries = self
                    .script_runtime
                    .variant_callable_public_properties
                    .entry(storage_key)
                    .or_default();
                Self::object_set_entry(
                    entries,
                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                    prototype,
                );
                Ok(())
            }
            _ => Err(Error::ScriptRuntime(
                "Object.setPrototypeOf target must be an object".into(),
            )),
        }
    }

    pub(crate) fn object_set_prototype_of_value(
        &mut self,
        target: &Value,
        prototype: &Value,
    ) -> Result<Value> {
        if !matches!(prototype, Value::Null) && Self::is_primitive_value(prototype) {
            return Err(Error::ScriptRuntime(
                "Object.setPrototypeOf prototype must be an object or null".into(),
            ));
        }

        if matches!(target, Value::Null | Value::Undefined) {
            return Err(Error::ScriptRuntime(
                "Object.setPrototypeOf target must be an object".into(),
            ));
        }
        if Self::is_primitive_value(target) {
            return Ok(target.clone());
        }

        if self.object_set_prototype_would_cycle(target, prototype) {
            return Err(Error::ScriptRuntime("Cyclic __proto__ value".into()));
        }
        self.set_object_like_internal_prototype(target, prototype.clone())?;
        Ok(target.clone())
    }

    pub(crate) fn object_freeze_value(&mut self, value: &Value) -> Result<Value> {
        match value {
            Value::TypedArray(array) => {
                if array.borrow().observed_length() > 0 {
                    return Err(Error::ScriptRuntime(
                        "Cannot freeze array buffer views with elements".into(),
                    ));
                }
                Ok(Value::TypedArray(array.clone()))
            }
            other => Ok(other.clone()),
        }
    }
}
