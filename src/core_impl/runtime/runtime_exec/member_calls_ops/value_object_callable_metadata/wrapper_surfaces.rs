use super::*;

impl Harness {
    pub(crate) fn variant_callable_public_storage_key(value: &Value) -> Option<String> {
        match value {
            Value::StringConstructor => Some("String".to_string()),
            Value::SymbolConstructor => Some("Symbol".to_string()),
            Value::MapConstructor => Some("Map".to_string()),
            Value::WeakMapConstructor => Some("WeakMap".to_string()),
            Value::SetConstructor => Some("Set".to_string()),
            Value::WeakSetConstructor => Some("WeakSet".to_string()),
            Value::PromiseConstructor => Some("Promise".to_string()),
            Value::BlobConstructor => Some("Blob".to_string()),
            Value::ArrayBufferConstructor => Some("ArrayBuffer".to_string()),
            Value::RegExpConstructor => Some("RegExp".to_string()),
            Value::UrlSearchParamsConstructor => Some("URLSearchParams".to_string()),
            Value::TypedArrayConstructor(kind) => Some(format!(
                "TypedArrayConstructor:{}",
                match kind {
                    TypedArrayConstructorKind::Concrete(kind) => kind.name(),
                    TypedArrayConstructorKind::Abstract => "TypedArray",
                }
            )),
            _ => None,
        }
    }

    pub(crate) fn variant_callable_internal_prototype_value(&self, value: &Value) -> Option<Value> {
        let storage_key = Self::variant_callable_public_storage_key(value)?;
        let entries = self
            .script_runtime
            .variant_callable_public_properties
            .get(&storage_key)?;
        Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY)
    }

    pub(crate) fn new_string_wrapper_value(value: String) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_STRING_WRAPPER_VALUE_KEY.to_string(),
            Value::String(value),
        )])
    }

    pub(crate) fn new_boolean_wrapper_value(value: bool) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_BOOLEAN_WRAPPER_VALUE_KEY.to_string(),
            Value::Bool(value),
        )])
    }

    pub(crate) fn new_number_wrapper_value(value: Value) -> Value {
        Self::new_object_value(vec![(INTERNAL_NUMBER_WRAPPER_VALUE_KEY.to_string(), value)])
    }

    pub(crate) fn new_bigint_wrapper_value(value: JsBigInt) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_BIGINT_WRAPPER_VALUE_KEY.to_string(),
            Value::BigInt(value),
        )])
    }

    pub(crate) fn new_symbol_wrapper_value(symbol_id: usize) -> Value {
        Self::new_object_value(vec![(
            INTERNAL_SYMBOL_WRAPPER_KEY.to_string(),
            Value::Number(symbol_id as i64),
        )])
    }

    pub(crate) fn box_primitive_value(value: Value) -> Value {
        match value {
            Value::String(text) => Self::new_string_wrapper_value(text),
            Value::Bool(value) => Self::new_boolean_wrapper_value(value),
            Value::Number(value) => Self::new_number_wrapper_value(Value::Number(value)),
            Value::Float(value) => Self::new_number_wrapper_value(Value::Float(value)),
            Value::BigInt(value) => Self::new_bigint_wrapper_value(value),
            Value::Symbol(symbol) => Self::new_symbol_wrapper_value(symbol.id),
            other => other,
        }
    }
}
