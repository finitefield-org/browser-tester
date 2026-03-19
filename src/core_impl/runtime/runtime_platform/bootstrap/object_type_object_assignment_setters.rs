use super::*;

impl Harness {
    fn arguments_param_name_for_index(
        env: &HashMap<String, Value>,
        index: usize,
    ) -> Option<String> {
        let Some(Value::Array(bindings)) = env.get(INTERNAL_ARGUMENTS_PARAM_BINDINGS_KEY) else {
            return None;
        };
        match bindings.borrow().get(index) {
            Some(Value::String(name)) => Some(name.clone()),
            _ => None,
        }
    }

    pub(crate) fn read_object_assignment_property(
        &mut self,
        container: &Value,
        key_value: &Value,
        target: &str,
    ) -> Result<Value> {
        let key = self.property_key_to_storage_key(key_value);
        let value = self
            .object_property_from_value(container, &key)
            .map_err(|err| match err {
                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                    Error::ScriptRuntime(format!(
                        "variable '{}' is not an object (key '{}')",
                        target, key
                    ))
                }
                other => other,
            })?;

        if matches!(value, Value::Null | Value::Undefined) {
            let kind = if matches!(value, Value::Null) {
                "null"
            } else {
                "undefined"
            };
            return Err(Error::ScriptRuntime(format!(
                "cannot set property '{}' of {}",
                key, kind
            )));
        }
        Ok(value)
    }

    fn sync_param_after_arguments_write(
        &mut self,
        env: &mut HashMap<String, Value>,
        arguments_array: &Rc<RefCell<ArrayValue>>,
        index: usize,
        value: &Value,
    ) {
        let Some(Value::Array(arguments)) = env.get("arguments").cloned() else {
            return;
        };
        if !Rc::ptr_eq(&arguments, arguments_array) {
            return;
        }
        let Some(param_name) = Self::arguments_param_name_for_index(env, index) else {
            return;
        };
        env.insert(param_name, value.clone());
    }

    pub(crate) fn set_object_assignment_property(
        &mut self,
        container: &Value,
        key_value: &Value,
        value: Value,
        target: &str,
        env: &mut HashMap<String, Value>,
        event: &EventState,
    ) -> Result<()> {
        let reflect_set = target == "Reflect.set target";
        match container {
            Value::Object(object) => {
                let key = self.property_key_to_storage_key(key_value);
                if self.set_event_target_event_handler_property(object, &key, value.clone())? {
                    return Ok(());
                }
                let (
                    is_before_unload_event,
                    is_location,
                    is_history,
                    is_navigation,
                    is_window,
                    is_navigator,
                    is_document,
                    is_url,
                    is_storage,
                    is_data_transfer,
                    is_computed_style,
                    is_canvas_2d_context,
                    dom_string_map_owner,
                ) = {
                    let entries = object.borrow();
                    (
                        Self::is_before_unload_event_object(&entries),
                        Self::is_location_object(&entries),
                        Self::is_history_object(&entries),
                        Self::is_navigation_object(&entries),
                        Self::is_window_object(&entries),
                        Self::is_navigator_object(&entries),
                        Self::is_document_object(&entries),
                        Self::is_url_object(&entries),
                        Self::is_storage_object(&entries),
                        Self::is_data_transfer_object(&entries),
                        Self::is_computed_style_object(&entries),
                        Self::is_canvas_2d_context_object(&entries),
                        if Self::is_dom_string_map_object(&entries) {
                            Self::dom_string_map_owner_node(&entries)
                        } else {
                            None
                        },
                    )
                };
                if is_before_unload_event && key == "returnValue" {
                    let return_value = value.as_string();
                    let cancelable = {
                        let entries = object.borrow();
                        Self::object_get_entry(&entries, "cancelable")
                            .is_some_and(|value| value.truthy())
                    };
                    let mut entries = object.borrow_mut();
                    Self::object_set_entry(
                        &mut entries,
                        "returnValue".to_string(),
                        Value::String(return_value.clone()),
                    );
                    if cancelable && !return_value.is_empty() {
                        Self::object_set_entry(
                            &mut entries,
                            "defaultPrevented".to_string(),
                            Value::Bool(true),
                        );
                    }
                    return Ok(());
                }
                if is_location {
                    self.set_location_property(&key, value)?;
                    return Ok(());
                }
                if is_history {
                    self.set_history_property(&key, value)?;
                    return Ok(());
                }
                if is_navigation {
                    self.set_navigation_property(&key, value)?;
                    return Ok(());
                }
                if is_window {
                    self.set_window_property(&key, value)?;
                    return Ok(());
                }
                if is_navigator {
                    self.set_navigator_property(object, &key, value)?;
                    return Ok(());
                }
                if is_document {
                    self.set_document_property(object, &key, value)?;
                    return Ok(());
                }
                if is_url {
                    self.set_url_object_property(object, &key, value)?;
                    return Ok(());
                }
                if is_storage {
                    self.set_storage_object_property(object, &key, value)?;
                    return Ok(());
                }
                if is_data_transfer {
                    self.set_data_transfer_object_property(object, &key, value)?;
                    return Ok(());
                }
                if let Some(owner) =
                    dom_string_map_owner.filter(|_| !Self::is_symbol_storage_key(&key))
                {
                    let value = value.as_string();
                    self.dom.dataset_set(owner, &key, &value)?;
                    return Ok(());
                }
                if is_computed_style {
                    return Err(Error::ScriptRuntime(
                        "CSSStyleProperties is read-only".into(),
                    ));
                }
                if is_canvas_2d_context {
                    self.set_canvas_2d_context_property(object, &key, value)?;
                    return Ok(());
                }
                if Self::string_wrapper_builtin_has_own_property(&object.borrow(), &key) {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                let (own_setter, own_has_accessor, own_data, mut prototype) = {
                    let entries = object.borrow();
                    (
                        Self::object_setter_from_entries(&entries, &key),
                        Self::has_object_accessor_property(&entries, &key),
                        Self::object_get_entry(&entries, &key).is_some(),
                        Self::object_get_entry(&entries, INTERNAL_OBJECT_PROTOTYPE_KEY),
                    )
                };
                if let Some(setter) = own_setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        event,
                        None,
                        Some(container.clone()),
                    )?;
                    return Ok(());
                }
                if own_has_accessor {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if own_data && !Self::is_writable_object_key(&*object.borrow(), &key) {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if !own_data
                    && Self::callable_kind_from_value(container).is_some()
                    && Self::is_callable_own_surface_key(&key)
                {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if !own_data {
                    while let Some(Value::Object(proto)) = prototype {
                        let (setter, has_accessor, next) = {
                            let proto_ref = proto.borrow();
                            (
                                Self::object_setter_from_entries(&proto_ref, &key),
                                Self::has_object_accessor_property(&proto_ref, &key),
                                Self::object_get_entry(&proto_ref, INTERNAL_OBJECT_PROTOTYPE_KEY),
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
                                Some(container.clone()),
                            )?;
                            return Ok(());
                        }
                        if has_accessor {
                            if reflect_set {
                                return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                            }
                            return Ok(());
                        }
                        prototype = next;
                    }
                }
                Self::object_set_entry(&mut object.borrow_mut(), key, value);
                Ok(())
            }
            Value::Function(function) => {
                let key = self.property_key_to_storage_key(key_value);
                let (own_setter, own_getter, own_data, writable) = {
                    if let Some(entries) = self
                        .script_runtime
                        .function_public_properties
                        .get(&function.function_id)
                    {
                        (
                            Self::object_setter_from_entries(entries, &key),
                            Self::has_object_accessor_property(entries, &key),
                            Self::object_get_entry(entries, &key).is_some(),
                            Self::is_writable_object_key(entries, &key),
                        )
                    } else {
                        (None, false, false, true)
                    }
                };
                if let Some(setter) = own_setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        event,
                        None,
                        Some(container.clone()),
                    )?;
                    return Ok(());
                }
                if own_getter {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if own_data && !writable {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if !own_data && Self::is_callable_own_surface_key(&key) {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if !own_data {
                    let mut prototype = function.class_super_constructor.clone();
                    while let Some(current) = prototype {
                        match current {
                            Value::Function(proto_function) => {
                                let (setter, getter, next) = if let Some(entries) = self
                                    .script_runtime
                                    .function_public_properties
                                    .get(&proto_function.function_id)
                                {
                                    (
                                        Self::object_setter_from_entries(entries, &key),
                                        Self::has_object_accessor_property(entries, &key),
                                        proto_function.class_super_constructor.clone(),
                                    )
                                } else {
                                    (None, false, proto_function.class_super_constructor.clone())
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
                                        Some(container.clone()),
                                    )?;
                                    return Ok(());
                                }
                                if getter {
                                    if reflect_set {
                                        return Err(Error::ScriptRuntime(
                                            "Reflect.set failed".into(),
                                        ));
                                    }
                                    return Ok(());
                                }
                                prototype = next;
                            }
                            Value::Object(proto_object) => {
                                let (setter, getter, next) = {
                                    let proto_ref = proto_object.borrow();
                                    (
                                        Self::object_setter_from_entries(&proto_ref, &key),
                                        Self::has_object_accessor_property(&proto_ref, &key),
                                        Self::object_get_entry(
                                            &proto_ref,
                                            INTERNAL_OBJECT_PROTOTYPE_KEY,
                                        ),
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
                                        Some(container.clone()),
                                    )?;
                                    return Ok(());
                                }
                                if getter {
                                    if reflect_set {
                                        return Err(Error::ScriptRuntime(
                                            "Reflect.set failed".into(),
                                        ));
                                    }
                                    return Ok(());
                                }
                                prototype = next;
                            }
                            _ => break,
                        }
                    }
                }
                let entries = self
                    .script_runtime
                    .function_public_properties
                    .entry(function.function_id)
                    .or_default();
                if Self::is_function_builtin_prototype_key(function, &key) {
                    Self::set_function_builtin_prototype_property(entries, value, true);
                } else {
                    Self::object_set_entry(entries, key, value);
                }
                Ok(())
            }
            Value::UrlConstructor => {
                let key = self.property_key_to_storage_key(key_value);
                self.set_url_constructor_property(&key, value);
                Ok(())
            }
            Value::Array(array_values) => {
                if let Some(index) = self.value_as_index(key_value) {
                    if !Self::is_writable_object_key(
                        &array_values.borrow().properties,
                        &index.to_string(),
                    ) {
                        if reflect_set {
                            return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                        }
                        return Ok(());
                    }
                    let value_for_sync = value.clone();
                    {
                        let mut elements = array_values.borrow_mut();
                        if index >= elements.len() {
                            elements.resize(index + 1, Value::Undefined);
                        }
                        elements[index] = value;
                    }
                    Self::clear_array_hole(array_values, index);
                    self.sync_param_after_arguments_write(
                        env,
                        array_values,
                        index,
                        &value_for_sync,
                    );
                    return Ok(());
                }
                let key = self.property_key_to_storage_key(key_value);
                if key == "length" {
                    if !Self::is_writable_object_key(&array_values.borrow().properties, &key) {
                        if reflect_set {
                            return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                        }
                        return Ok(());
                    }
                    let mut values = array_values.borrow_mut();
                    let next = Self::value_to_i64(&value);
                    let next = if next <= 0 { 0usize } else { next as usize };
                    if next < values.len() {
                        values.truncate(next);
                    } else if next > values.len() {
                        values.resize(next, Value::Undefined);
                    }
                    return Ok(());
                }
                Self::set_array_property(array_values, key, value);
                Ok(())
            }
            Value::TypedArray(values) => {
                let Some(index) = self.value_as_index(key_value) else {
                    return Ok(());
                };
                self.typed_array_set_index(values, index, value)
            }
            Value::NodeList(nodes) => {
                let key = self.property_key_to_storage_key(key_value);
                let (own_setter, own_has_accessor, own_data, own_writable) = {
                    let nodes_ref = nodes.borrow();
                    (
                        Self::object_setter_from_entries(&nodes_ref.properties, &key),
                        Self::has_object_accessor_property(&nodes_ref.properties, &key),
                        Self::object_get_entry(&nodes_ref.properties, &key).is_some(),
                        Self::is_writable_object_key(&nodes_ref.properties, &key),
                    )
                };
                if let Some(setter) = own_setter {
                    if !self.is_callable_value(&setter) {
                        return Err(Error::ScriptRuntime("object setter is not callable".into()));
                    }
                    self.execute_callable_value_with_this_and_env(
                        &setter,
                        &[value],
                        event,
                        None,
                        Some(container.clone()),
                    )?;
                    return Ok(());
                }
                if own_has_accessor {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if own_data && !own_writable {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if key == "length" && !own_data {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if let Ok(index) = key.parse::<usize>()
                    && index < self.node_list_len(nodes)
                    && !own_data
                {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if !own_data {
                    let mut prototype = self.value_internal_prototype_value(container);
                    while let Some(current) = prototype {
                        match current {
                            Value::Object(proto) => {
                                let (setter, has_accessor, next) = {
                                    let proto_ref = proto.borrow();
                                    (
                                        Self::object_setter_from_entries(&proto_ref, &key),
                                        Self::has_object_accessor_property(&proto_ref, &key),
                                        Self::object_get_entry(
                                            &proto_ref,
                                            INTERNAL_OBJECT_PROTOTYPE_KEY,
                                        ),
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
                                        Some(container.clone()),
                                    )?;
                                    return Ok(());
                                }
                                if has_accessor {
                                    if reflect_set {
                                        return Err(Error::ScriptRuntime(
                                            "Reflect.set failed".into(),
                                        ));
                                    }
                                    return Ok(());
                                }
                                prototype = next;
                            }
                            _ => break,
                        }
                    }
                }
                Self::object_set_entry(&mut nodes.borrow_mut().properties, key, value);
                Ok(())
            }
            Value::Map(map) => {
                let key = self.property_key_to_storage_key(key_value);
                let has_own_surface = {
                    let map_ref = map.borrow();
                    Self::object_get_entry(&map_ref.properties, &key).is_some()
                        || Self::has_object_accessor_property(&map_ref.properties, &key)
                };
                if key == "size" && !has_own_surface {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if !Self::is_writable_object_key(&map.borrow().properties, &key) {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                Self::object_set_entry(&mut map.borrow_mut().properties, key, value);
                Ok(())
            }
            Value::WeakMap(weak_map) => {
                let key = self.property_key_to_storage_key(key_value);
                if !Self::is_writable_object_key(&weak_map.borrow().properties, &key) {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                Self::object_set_entry(&mut weak_map.borrow_mut().properties, key, value);
                Ok(())
            }
            Value::Set(set) => {
                let key = self.property_key_to_storage_key(key_value);
                let has_own_surface = {
                    let set_ref = set.borrow();
                    Self::object_get_entry(&set_ref.properties, &key).is_some()
                        || Self::has_object_accessor_property(&set_ref.properties, &key)
                };
                if key == "size" && !has_own_surface {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                if !Self::is_writable_object_key(&set.borrow().properties, &key) {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                Self::object_set_entry(&mut set.borrow_mut().properties, key, value);
                Ok(())
            }
            Value::WeakSet(weak_set) => {
                let key = self.property_key_to_storage_key(key_value);
                if !Self::is_writable_object_key(&weak_set.borrow().properties, &key) {
                    if reflect_set {
                        return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                    }
                    return Ok(());
                }
                Self::object_set_entry(&mut weak_set.borrow_mut().properties, key, value);
                Ok(())
            }
            Value::RegExp(regex) => {
                let key = self.property_key_to_storage_key(key_value);
                if key == "lastIndex" {
                    if !Self::is_writable_object_key(&regex.borrow().properties, &key) {
                        if reflect_set {
                            return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                        }
                        return Ok(());
                    }
                    let mut regex = regex.borrow_mut();
                    let next = Self::value_to_i64(&value);
                    regex.last_index = if next <= 0 { 0 } else { next as usize };
                } else {
                    let has_own_surface = {
                        let regex_ref = regex.borrow();
                        Self::object_get_entry(&regex_ref.properties, &key).is_some()
                            || Self::has_object_accessor_property(&regex_ref.properties, &key)
                    };
                    if Self::is_regexp_builtin_own_key(&key) && !has_own_surface {
                        if reflect_set {
                            return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                        }
                        return Ok(());
                    }
                    if !Self::is_writable_object_key(&regex.borrow().properties, &key) {
                        if reflect_set {
                            return Err(Error::ScriptRuntime("Reflect.set failed".into()));
                        }
                        return Ok(());
                    }
                    Self::object_set_entry(&mut regex.borrow_mut().properties, key, value);
                }
                Ok(())
            }
            Value::Node(node) => {
                let key = self.property_key_to_storage_key(key_value);
                self.set_node_assignment_property(*node, &key, value, event, reflect_set)
            }
            _ => Err(Error::ScriptRuntime(format!(
                "variable '{}' is not an object (assignment target)",
                target
            ))),
        }
    }
}
