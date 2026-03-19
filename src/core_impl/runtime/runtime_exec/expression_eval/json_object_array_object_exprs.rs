use super::*;

impl Harness {
    pub(crate) fn try_eval_object_expr(
        &mut self,
        expr: &Expr,
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let result = match expr {
            Expr::ObjectConstruct { value } => {
                let value = value
                    .as_ref()
                    .map(|value| self.eval_expr(value, env, event_param, event))
                    .transpose()?
                    .unwrap_or(Value::Undefined);
                match value {
                    Value::Null | Value::Undefined => Ok(Self::new_object_value(Vec::new())),
                    Value::Object(object) => Ok(Value::Object(object)),
                    Value::Array(array) => Ok(Value::Array(array)),
                    Value::Date(date) => Ok(Value::Date(date)),
                    Value::Map(map) => Ok(Value::Map(map)),
                    Value::Set(set) => Ok(Value::Set(set)),
                    Value::Blob(blob) => Ok(Value::Blob(blob)),
                    Value::ArrayBuffer(buffer) => Ok(Value::ArrayBuffer(buffer)),
                    Value::TypedArray(array) => Ok(Value::TypedArray(array)),
                    Value::Promise(promise) => Ok(Value::Promise(promise)),
                    Value::RegExp(regex) => Ok(Value::RegExp(regex)),
                    primitive => Ok(Self::box_primitive_value(primitive)),
                }
            }
            Expr::ObjectLiteral(entries) => {
                let mut object_entries = Vec::with_capacity(entries.len());
                for entry in entries {
                    match entry {
                        ObjectLiteralEntry::Pair(key, value) => {
                            let key = match key {
                                ObjectLiteralKey::Static(key) => key.clone(),
                                ObjectLiteralKey::Computed(expr) => {
                                    let key = self.eval_expr(expr, env, event_param, event)?;
                                    self.property_key_to_storage_key(&key)
                                }
                            };

                            let value = match value {
                                Expr::Function {
                                    handler,
                                    name: _,
                                    is_async,
                                    is_generator,
                                    is_arrow,
                                    is_method,
                                } if *is_method => {
                                    let super_prototype = match Self::object_get_entry(
                                        &object_entries,
                                        INTERNAL_OBJECT_PROTOTYPE_KEY,
                                    ) {
                                        Some(Value::Object(proto)) => Some(Value::Object(proto)),
                                        _ => None,
                                    };
                                    self.make_function_value_with_super(
                                        handler.clone(),
                                        env,
                                        false,
                                        *is_async,
                                        *is_generator,
                                        *is_arrow,
                                        *is_method,
                                        None,
                                        super_prototype,
                                    )
                                }
                                _ => self.eval_expr(value, env, event_param, event)?,
                            };

                            Self::define_object_literal_data_entry(&mut object_entries, key, value);
                        }
                        ObjectLiteralEntry::ProtoSetter(expr) => {
                            let value = self.eval_expr(expr, env, event_param, event)?;
                            if matches!(value, Value::Object(_) | Value::Null) {
                                Self::object_set_entry(
                                    &mut object_entries,
                                    INTERNAL_OBJECT_PROTOTYPE_KEY.to_string(),
                                    value,
                                );
                            }
                        }
                        ObjectLiteralEntry::Getter(key, handler) => {
                            let key = match key {
                                ObjectLiteralKey::Static(key) => key.clone(),
                                ObjectLiteralKey::Computed(expr) => {
                                    let key = self.eval_expr(expr, env, event_param, event)?;
                                    self.property_key_to_storage_key(&key)
                                }
                            };
                            let getter = self.make_function_value(
                                handler.clone(),
                                env,
                                false,
                                false,
                                false,
                                false,
                                true,
                            );
                            Self::define_object_literal_getter_entry(
                                &mut object_entries,
                                key,
                                getter,
                            );
                        }
                        ObjectLiteralEntry::Setter(key, handler) => {
                            let key = match key {
                                ObjectLiteralKey::Static(key) => key.clone(),
                                ObjectLiteralKey::Computed(expr) => {
                                    let key = self.eval_expr(expr, env, event_param, event)?;
                                    self.property_key_to_storage_key(&key)
                                }
                            };
                            let setter = self.make_function_value(
                                handler.clone(),
                                env,
                                false,
                                false,
                                false,
                                false,
                                true,
                            );
                            Self::define_object_literal_setter_entry(
                                &mut object_entries,
                                key,
                                setter,
                            );
                        }
                        ObjectLiteralEntry::Spread(expr) => {
                            let spread_value = self.eval_expr(expr, env, event_param, event)?;
                            match spread_value {
                                Value::Null | Value::Undefined => {}
                                Value::Object(entries) => {
                                    let source = Value::Object(entries.clone());
                                    let keys = self.object_like_enumerable_keys(&source)?;
                                    for key in keys {
                                        let value =
                                            self.object_property_from_value(&source, &key)?;
                                        Self::define_object_literal_data_entry(
                                            &mut object_entries,
                                            key,
                                            value,
                                        );
                                    }
                                }
                                Value::NodeList(nodes) => {
                                    let source = Value::NodeList(nodes.clone());
                                    let keys = self.object_like_enumerable_keys(&source)?;
                                    for key in keys {
                                        let value =
                                            self.object_property_from_value(&source, &key)?;
                                        Self::define_object_literal_data_entry(
                                            &mut object_entries,
                                            key,
                                            value,
                                        );
                                    }
                                }
                                Value::Node(node) => {
                                    let source = Value::Node(node);
                                    let keys = self.object_like_enumerable_keys(&source)?;
                                    for key in keys {
                                        let value =
                                            self.object_property_from_value(&source, &key)?;
                                        Self::define_object_literal_data_entry(
                                            &mut object_entries,
                                            key,
                                            value,
                                        );
                                    }
                                }
                                Value::Array(values) => {
                                    for (index, value) in values.borrow().iter().enumerate() {
                                        let key = index.to_string();
                                        Self::define_object_literal_data_entry(
                                            &mut object_entries,
                                            key,
                                            value.clone(),
                                        );
                                    }
                                }
                                Value::String(text) => {
                                    for (index, ch) in text.chars().enumerate() {
                                        let key = index.to_string();
                                        Self::define_object_literal_data_entry(
                                            &mut object_entries,
                                            key,
                                            Value::String(ch.to_string()),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Self::new_object_value(object_entries))
            }
            Expr::ObjectGet { target, key } => {
                match self.resolve_target_value_with_pending(env, target) {
                    _ if target == "super" => {
                        let super_prototype = Self::super_prototype_from_env(env)?;
                        let this_value = Self::super_this_from_env(env)?;
                        self.object_property_from_value_with_receiver(
                            &super_prototype,
                            key,
                            &this_value,
                        )
                    }
                    Some(value) => {
                        self.object_property_from_value(&value, key)
                            .map_err(|err| match err {
                                Error::ScriptRuntime(msg) if msg == "value is not an object" => {
                                    Error::ScriptRuntime(format!(
                                        "variable '{}' is not an object (key '{}')",
                                        target, key
                                    ))
                                }
                                other => other,
                            })
                    }
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                }
            }
            Expr::ObjectPathGet { target, path } => {
                if target == "super" {
                    let mut value = Self::super_prototype_from_env(env)?;
                    let this_value = Self::super_this_from_env(env)?;
                    for (index, key) in path.iter().enumerate() {
                        if index == 0 {
                            value = self.object_property_from_value_with_receiver(
                                &value,
                                key,
                                &this_value,
                            )?;
                        } else {
                            value = self.object_property_from_value(&value, key)?;
                        }
                    }
                    Ok(value)
                } else {
                    let Some(mut value) = self.resolve_target_value_with_pending(env, target)
                    else {
                        return Err(Error::ScriptRuntime(format!(
                            "unknown variable: {}",
                            target
                        )));
                    };
                    for key in path {
                        value = self.object_property_from_value(&value, key)?;
                    }
                    Ok(value)
                }
            }
            Expr::ObjectGetOwnPropertySymbols(object) => {
                let object = self.eval_expr(object, env, event_param, event)?;
                self.object_get_own_property_symbols_value(&object)
            }
            Expr::ObjectGetOwnPropertyDescriptor { object, key } => {
                let object = self.eval_expr(object, env, event_param, event)?;
                let key = self.eval_expr(key, env, event_param, event)?;
                let key = self.property_key_to_storage_key(&key);
                self.object_get_own_property_descriptor_value(&object, &key)
            }
            Expr::ObjectDefineProperty {
                object,
                key,
                descriptor,
            } => {
                let object = self.eval_expr(object, env, event_param, event)?;
                let key = self.eval_expr(key, env, event_param, event)?;
                let key = self.property_key_to_storage_key(&key);
                let descriptor = self.eval_expr(descriptor, env, event_param, event)?;
                self.object_define_property_value(&object, &key, &descriptor)
            }
            Expr::ObjectGetOwnPropertyNames(object) => {
                let object = self.eval_expr(object, env, event_param, event)?;
                self.object_get_own_property_names_value(&object)
            }
            Expr::ObjectKeys(object) => {
                let object = self.eval_expr(object, env, event_param, event)?;
                self.object_keys_value(&object)
            }
            Expr::ObjectValues(object) => {
                let object = self.eval_expr(object, env, event_param, event)?;
                self.object_values_value(&object)
            }
            Expr::ObjectEntries(object) => {
                let object = self.eval_expr(object, env, event_param, event)?;
                self.object_entries_value(&object)
            }
            Expr::ObjectHasOwn { object, key } => {
                let object = self.eval_expr(object, env, event_param, event)?;
                let key = self.eval_expr(key, env, event_param, event)?;
                let key = self.property_key_to_storage_key(&key);
                self.object_has_own_value(&object, &key)
            }
            Expr::ObjectGetPrototypeOf(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                self.object_get_prototype_of_value(&value)
            }
            Expr::ObjectFreeze(value) => {
                let value = self.eval_expr(value, env, event_param, event)?;
                self.object_freeze_value(&value)
            }
            Expr::ReflectSet {
                target,
                key,
                value,
                receiver,
            } => {
                let target = self.eval_expr(target, env, event_param, event)?;
                let key = self.eval_expr(key, env, event_param, event)?;
                let key = self.property_key_to_storage_key(&key);
                let value = self.eval_expr(value, env, event_param, event)?;
                let receiver = receiver
                    .as_ref()
                    .map(|receiver| self.eval_expr(receiver, env, event_param, event))
                    .transpose()?
                    .unwrap_or_else(|| target.clone());
                Ok(Value::Bool(self.reflect_set_object_property_value(
                    &target, &key, value, &receiver, event,
                )?))
            }
            Expr::ReflectOwnKeys(object) => {
                let object = self.eval_expr(object, env, event_param, event)?;
                self.reflect_own_keys_value(&object)
            }
            Expr::ObjectHasOwnProperty { target, key } => {
                let key = self.eval_expr(key, env, event_param, event)?;
                let key = self.property_key_to_storage_key(&key);
                match self.resolve_target_value_with_pending(env, target) {
                    Some(value @ Value::Object(_)) => self.object_has_own_value(&value, &key),
                    Some(_) => Err(Error::ScriptRuntime(format!(
                        "variable '{}' is not an object",
                        target
                    ))),
                    None => Err(Error::ScriptRuntime(format!(
                        "unknown variable: {}",
                        target
                    ))),
                }
            }
            _ => Err(Error::ScriptRuntime(UNHANDLED_EXPR_CHUNK.into())),
        }?;
        Ok(result)
    }
}
