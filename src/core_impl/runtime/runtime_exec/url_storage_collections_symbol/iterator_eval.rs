use super::*;

impl Harness {
    pub(crate) fn new_iterator_constructor_value(&self) -> Value {
        let prototype = Self::new_object_value(vec![
            ("next".to_string(), Self::new_builtin_placeholder_function()),
            ("drop".to_string(), Self::new_builtin_placeholder_function()),
            (
                "every".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "filter".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("find".to_string(), Self::new_builtin_placeholder_function()),
            (
                "flatMap".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "forEach".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("map".to_string(), Self::new_builtin_placeholder_function()),
            (
                "reduce".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("some".to_string(), Self::new_builtin_placeholder_function()),
            ("take".to_string(), Self::new_builtin_placeholder_function()),
            (
                "toArray".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ]);
        Self::new_object_value(vec![
            (
                INTERNAL_ITERATOR_CONSTRUCTOR_OBJECT_KEY.to_string(),
                Value::Bool(true),
            ),
            ("from".to_string(), Self::new_builtin_placeholder_function()),
            (
                "concat".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("prototype".to_string(), prototype),
        ])
    }

    pub(crate) fn new_iterator_value(&self, values: Vec<Value>) -> Value {
        Self::new_object_value(vec![
            (INTERNAL_ITERATOR_OBJECT_KEY.to_string(), Value::Bool(true)),
            (
                INTERNAL_ITERATOR_VALUES_KEY.to_string(),
                Self::new_array_value(values),
            ),
            (INTERNAL_ITERATOR_INDEX_KEY.to_string(), Value::Number(0)),
            ("next".to_string(), Self::new_builtin_placeholder_function()),
            ("drop".to_string(), Self::new_builtin_placeholder_function()),
            (
                "every".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "filter".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("find".to_string(), Self::new_builtin_placeholder_function()),
            (
                "flatMap".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            (
                "forEach".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("map".to_string(), Self::new_builtin_placeholder_function()),
            (
                "reduce".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
            ("some".to_string(), Self::new_builtin_placeholder_function()),
            ("take".to_string(), Self::new_builtin_placeholder_function()),
            (
                "toArray".to_string(),
                Self::new_builtin_placeholder_function(),
            ),
        ])
    }

    pub(crate) fn is_iterator_constructor_object(
        entries: &(impl ObjectEntryLookup + ?Sized),
    ) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_ITERATOR_CONSTRUCTOR_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn is_iterator_object(entries: &(impl ObjectEntryLookup + ?Sized)) -> bool {
        matches!(
            Self::object_get_entry(entries, INTERNAL_ITERATOR_OBJECT_KEY),
            Some(Value::Bool(true))
        )
    }

    pub(crate) fn iterator_next_value_from_object(
        &self,
        iterator: &Rc<RefCell<ObjectValue>>,
    ) -> Result<Option<Value>> {
        let mut entries = iterator.borrow_mut();
        if !Self::is_iterator_object(&entries) {
            return Err(Error::ScriptRuntime("value is not an Iterator".into()));
        }
        let values = match Self::object_get_entry(&entries, INTERNAL_ITERATOR_VALUES_KEY) {
            Some(Value::Array(values)) => values,
            _ => {
                return Err(Error::ScriptRuntime(
                    "Iterator has invalid internal state".into(),
                ));
            }
        };
        let index = match Self::object_get_entry(&entries, INTERNAL_ITERATOR_INDEX_KEY) {
            Some(Value::Number(value)) if value >= 0 => value as usize,
            _ => 0,
        };
        let value = {
            let values = values.borrow();
            if index >= values.len() {
                return Ok(None);
            }
            values.get(index).cloned().unwrap_or(Value::Undefined)
        };
        Self::object_set_entry(
            &mut entries,
            INTERNAL_ITERATOR_INDEX_KEY.to_string(),
            Value::Number((index + 1) as i64),
        );
        Ok(Some(value))
    }

    pub(crate) fn iterator_collect_remaining_values(
        &self,
        iterator: &Rc<RefCell<ObjectValue>>,
    ) -> Result<Vec<Value>> {
        let mut out = Vec::new();
        while let Some(value) = self.iterator_next_value_from_object(iterator)? {
            out.push(value);
        }
        Ok(out)
    }

    pub(crate) fn iterator_values_from_source(&self, source: &Value) -> Result<Vec<Value>> {
        match source {
            Value::Object(entries) => {
                if Self::is_iterator_object(&entries.borrow()) {
                    self.iterator_collect_remaining_values(entries)
                } else {
                    self.array_like_values_from_value(source)
                }
            }
            _ => self.array_like_values_from_value(source),
        }
    }

    pub(crate) fn eval_iterator_constructor_member_call(
        &mut self,
        constructor: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        if !Self::is_iterator_constructor_object(&constructor.borrow()) {
            return Ok(None);
        }
        let value = match member {
            "from" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.from requires exactly one argument".into(),
                    ));
                }
                if let Value::Object(entries) = &evaluated_args[0] {
                    if Self::is_iterator_object(&entries.borrow()) {
                        return Ok(Some(evaluated_args[0].clone()));
                    }
                }
                let values = self.iterator_values_from_source(&evaluated_args[0])?;
                self.new_iterator_value(values)
            }
            "concat" => {
                let mut values = Vec::new();
                for arg in evaluated_args {
                    values.extend(self.iterator_values_from_source(arg)?);
                }
                self.new_iterator_value(values)
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }

    pub(crate) fn eval_iterator_member_call(
        &mut self,
        iterator: &Rc<RefCell<ObjectValue>>,
        member: &str,
        evaluated_args: &[Value],
        event: &EventState,
    ) -> Result<Option<Value>> {
        if !Self::is_iterator_object(&iterator.borrow()) {
            return Ok(None);
        }
        let value = match member {
            "next" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Iterator.next does not take arguments".into(),
                    ));
                }
                if let Some(value) = self.iterator_next_value_from_object(iterator)? {
                    Self::new_object_value(vec![
                        ("value".to_string(), value),
                        ("done".to_string(), Value::Bool(false)),
                    ])
                } else {
                    Self::new_object_value(vec![
                        ("value".to_string(), Value::Undefined),
                        ("done".to_string(), Value::Bool(true)),
                    ])
                }
            }
            "toArray" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "Iterator.toArray does not take arguments".into(),
                    ));
                }
                let values = self.iterator_collect_remaining_values(iterator)?;
                Self::new_array_value(values)
            }
            "map" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.map requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let source = self.iterator_collect_remaining_values(iterator)?;
                let mut out = Vec::with_capacity(source.len());
                for (index, item) in source.into_iter().enumerate() {
                    out.push(self.execute_callback_value(
                        &callback,
                        &[item, Value::Number(index as i64)],
                        event,
                    )?);
                }
                self.new_iterator_value(out)
            }
            "filter" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.filter requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let source = self.iterator_collect_remaining_values(iterator)?;
                let mut out = Vec::new();
                for (index, item) in source.into_iter().enumerate() {
                    let keep = self.execute_callback_value(
                        &callback,
                        &[item.clone(), Value::Number(index as i64)],
                        event,
                    )?;
                    if keep.truthy() {
                        out.push(item);
                    }
                }
                self.new_iterator_value(out)
            }
            "flatMap" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.flatMap requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let source = self.iterator_collect_remaining_values(iterator)?;
                let mut out = Vec::new();
                for (index, item) in source.into_iter().enumerate() {
                    let mapped = self.execute_callback_value(
                        &callback,
                        &[item, Value::Number(index as i64)],
                        event,
                    )?;
                    out.extend(self.iterator_values_from_source(&mapped)?);
                }
                self.new_iterator_value(out)
            }
            "drop" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.drop requires exactly one count argument".into(),
                    ));
                }
                let count = Self::value_to_i64(&evaluated_args[0]);
                if count < 0 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.drop count must be non-negative".into(),
                    ));
                }
                let source = self.iterator_collect_remaining_values(iterator)?;
                let count = usize::try_from(count).unwrap_or(usize::MAX);
                self.new_iterator_value(source.into_iter().skip(count).collect())
            }
            "take" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.take requires exactly one count argument".into(),
                    ));
                }
                let count = Self::value_to_i64(&evaluated_args[0]);
                if count < 0 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.take count must be non-negative".into(),
                    ));
                }
                let source = self.iterator_collect_remaining_values(iterator)?;
                let count = usize::try_from(count).unwrap_or(usize::MAX);
                self.new_iterator_value(source.into_iter().take(count).collect())
            }
            "reduce" => {
                if evaluated_args.is_empty() || evaluated_args.len() > 2 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.reduce requires callback and optional initial value".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let source = self.iterator_collect_remaining_values(iterator)?;
                let mut start_index = 0usize;
                let mut acc = if let Some(initial) = evaluated_args.get(1) {
                    initial.clone()
                } else {
                    let Some(first) = source.first().cloned() else {
                        return Err(Error::ScriptRuntime(
                            "Iterator.reduce of empty iterator with no initial value".into(),
                        ));
                    };
                    start_index = 1;
                    first
                };
                for (index, value) in source.into_iter().enumerate().skip(start_index) {
                    acc = self.execute_callback_value(
                        &callback,
                        &[acc, value, Value::Number(index as i64)],
                        event,
                    )?;
                }
                acc
            }
            "find" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.find requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let source = self.iterator_collect_remaining_values(iterator)?;
                let mut found = Value::Undefined;
                for (index, value) in source.into_iter().enumerate() {
                    let matched = self.execute_callback_value(
                        &callback,
                        &[value.clone(), Value::Number(index as i64)],
                        event,
                    )?;
                    if matched.truthy() {
                        found = value;
                        break;
                    }
                }
                found
            }
            "some" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.some requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let source = self.iterator_collect_remaining_values(iterator)?;
                let mut matched = false;
                for (index, value) in source.into_iter().enumerate() {
                    let keep = self.execute_callback_value(
                        &callback,
                        &[value, Value::Number(index as i64)],
                        event,
                    )?;
                    if keep.truthy() {
                        matched = true;
                        break;
                    }
                }
                Value::Bool(matched)
            }
            "every" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.every requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let source = self.iterator_collect_remaining_values(iterator)?;
                let mut all = true;
                for (index, value) in source.into_iter().enumerate() {
                    let keep = self.execute_callback_value(
                        &callback,
                        &[value, Value::Number(index as i64)],
                        event,
                    )?;
                    if !keep.truthy() {
                        all = false;
                        break;
                    }
                }
                Value::Bool(all)
            }
            "forEach" => {
                if evaluated_args.len() != 1 {
                    return Err(Error::ScriptRuntime(
                        "Iterator.forEach requires exactly one callback argument".into(),
                    ));
                }
                let callback = evaluated_args[0].clone();
                if !self.is_callable_value(&callback) {
                    return Err(Error::ScriptRuntime("callback is not a function".into()));
                }
                let source = self.iterator_collect_remaining_values(iterator)?;
                for (index, value) in source.into_iter().enumerate() {
                    let _ = self.execute_callback_value(
                        &callback,
                        &[value, Value::Number(index as i64)],
                        event,
                    )?;
                }
                Value::Undefined
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
