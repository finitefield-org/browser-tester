use super::*;

impl Harness {
    pub(crate) fn collect_left_associative_binary_operands<'a>(
        expr: &'a Expr,
        op: BinaryOp,
    ) -> Vec<&'a Expr> {
        let mut right_operands = Vec::new();
        let mut cursor = expr;
        loop {
            match cursor {
                Expr::Binary {
                    left,
                    op: inner_op,
                    right,
                } if *inner_op == op => {
                    right_operands.push(right.as_ref());
                    cursor = left.as_ref();
                }
                _ => break,
            }
        }

        let mut out = Vec::with_capacity(right_operands.len() + 1);
        out.push(cursor);
        while let Some(operand) = right_operands.pop() {
            out.push(operand);
        }
        out
    }

    pub(crate) fn eval_binary(
        &mut self,
        op: &BinaryOp,
        left: &Value,
        right: &Value,
    ) -> Result<Value> {
        if matches!(left, Value::Symbol(_)) || matches!(right, Value::Symbol(_)) {
            if matches!(
                op,
                BinaryOp::BitOr
                    | BinaryOp::BitXor
                    | BinaryOp::BitAnd
                    | BinaryOp::ShiftLeft
                    | BinaryOp::ShiftRight
                    | BinaryOp::UnsignedShiftRight
                    | BinaryOp::Pow
                    | BinaryOp::Lt
                    | BinaryOp::Gt
                    | BinaryOp::Le
                    | BinaryOp::Ge
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Mod
                    | BinaryOp::Div
            ) {
                return Err(Error::ScriptRuntime(
                    "Cannot convert a Symbol value to a number".into(),
                ));
            }
        }
        let out = match op {
            BinaryOp::Or => {
                if left.truthy() {
                    left.clone()
                } else {
                    right.clone()
                }
            }
            BinaryOp::And => {
                if left.truthy() {
                    right.clone()
                } else {
                    left.clone()
                }
            }
            BinaryOp::Nullish => {
                if matches!(left, Value::Null | Value::Undefined) {
                    right.clone()
                } else {
                    left.clone()
                }
            }
            BinaryOp::Eq => Value::Bool(self.loose_equal(left, right)),
            BinaryOp::Ne => Value::Bool(!self.loose_equal(left, right)),
            BinaryOp::StrictEq => Value::Bool(self.strict_equal(left, right)),
            BinaryOp::StrictNe => Value::Bool(!self.strict_equal(left, right)),
            BinaryOp::In => Value::Bool(self.value_in(left, right)?),
            BinaryOp::InstanceOf => Value::Bool(self.value_instance_of(left, right)?),
            BinaryOp::BitOr => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l | r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) | self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::BitXor => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l ^ r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) ^ self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::BitAnd => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l & r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                Value::Number(i64::from(
                    self.to_i32_for_bitwise(left) & self.to_i32_for_bitwise(right),
                ))
            }
            BinaryOp::ShiftLeft => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(Self::bigint_shift_left(l, r)?));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_i32_for_bitwise(left) << shift))
            }
            BinaryOp::ShiftRight => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(Self::bigint_shift_right(l, r)?));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in bitwise operations".into(),
                    ));
                }
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_i32_for_bitwise(left) >> shift))
            }
            BinaryOp::UnsignedShiftRight => {
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "BigInt values do not support unsigned right shift".into(),
                    ));
                }
                let shift = self.to_u32_for_bitwise(right) & 0x1f;
                Value::Number(i64::from(self.to_u32_for_bitwise(left) >> shift))
            }
            BinaryOp::Pow => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    if r.sign() == Sign::Minus {
                        return Err(Error::ScriptRuntime(
                            "BigInt exponent must be non-negative".into(),
                        ));
                    }
                    let exp = r.to_u32().ok_or_else(|| {
                        Error::ScriptRuntime("BigInt exponent is too large".into())
                    })?;
                    return Ok(Value::BigInt(l.pow(exp)));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                let base = Self::coerce_number_for_global(left);
                let exponent = Self::coerce_number_for_global(right);
                if (base == 1.0 || base == -1.0) && !exponent.is_finite() {
                    Value::Float(f64::NAN)
                } else {
                    Value::Float(base.powf(exponent))
                }
            }
            BinaryOp::Lt => Value::Bool(self.compare(left, right, |l, r| l < r)),
            BinaryOp::Gt => Value::Bool(self.compare(left, right, |l, r| l > r)),
            BinaryOp::Le => Value::Bool(self.compare(left, right, |l, r| l <= r)),
            BinaryOp::Ge => Value::Bool(self.compare(left, right, |l, r| l >= r)),
            BinaryOp::Sub => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l - r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(self.numeric_value(left) - self.numeric_value(right))
            }
            BinaryOp::Mul => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    return Ok(Value::BigInt(l * r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(
                    Self::coerce_number_for_global(left) * Self::coerce_number_for_global(right),
                )
            }
            BinaryOp::Mod => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    if r.is_zero() {
                        return Err(Error::ScriptRuntime("modulo by zero".into()));
                    }
                    return Ok(Value::BigInt(l % r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                let rhs = self.numeric_value(right);
                if rhs == 0.0 {
                    return Err(Error::ScriptRuntime("modulo by zero".into()));
                }
                Value::Float(self.numeric_value(left) % rhs)
            }
            BinaryOp::Div => {
                if let (Value::BigInt(l), Value::BigInt(r)) = (left, right) {
                    if r.is_zero() {
                        return Err(Error::ScriptRuntime("division by zero".into()));
                    }
                    return Ok(Value::BigInt(l / r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(
                    Self::coerce_number_for_global(left) / Self::coerce_number_for_global(right),
                )
            }
        };
        Ok(out)
    }

    pub(crate) fn loose_equal(&self, left: &Value, right: &Value) -> bool {
        if self.strict_equal(left, right) {
            return true;
        }

        match (left, right) {
            (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => true,
            (Value::BigInt(l), Value::String(r)) => {
                Self::parse_js_bigint_from_string(r).is_ok_and(|parsed| parsed == *l)
            }
            (Value::String(l), Value::BigInt(r)) => {
                Self::parse_js_bigint_from_string(l).is_ok_and(|parsed| parsed == *r)
            }
            (Value::BigInt(_), Value::Number(_) | Value::Float(_))
            | (Value::Number(_) | Value::Float(_), Value::BigInt(_)) => {
                Self::number_bigint_loose_equal(left, right)
            }
            (Value::Number(_) | Value::Float(_), Value::String(_))
            | (Value::String(_), Value::Number(_) | Value::Float(_)) => {
                Self::coerce_number_for_global(left) == Self::coerce_number_for_global(right)
            }
            (Value::Bool(_), _) => {
                let coerced = Value::Float(Self::coerce_number_for_global(left));
                self.loose_equal(&coerced, right)
            }
            (_, Value::Bool(_)) => {
                let coerced = Value::Float(Self::coerce_number_for_global(right));
                self.loose_equal(left, &coerced)
            }
            _ if Self::is_loose_primitive(left) && Self::is_loose_object(right) => {
                let prim = self.to_primitive_for_loose(right);
                self.loose_equal(left, &prim)
            }
            _ if Self::is_loose_object(left) && Self::is_loose_primitive(right) => {
                let prim = self.to_primitive_for_loose(left);
                self.loose_equal(&prim, right)
            }
            _ => false,
        }
    }

    pub(crate) fn is_loose_primitive(value: &Value) -> bool {
        matches!(
            value,
            Value::String(_)
                | Value::Bool(_)
                | Value::Number(_)
                | Value::Float(_)
                | Value::BigInt(_)
                | Value::Symbol(_)
                | Value::Null
                | Value::Undefined
        )
    }

    pub(crate) fn is_loose_object(value: &Value) -> bool {
        matches!(
            value,
            Value::Array(_)
                | Value::Object(_)
                | Value::Promise(_)
                | Value::Map(_)
                | Value::WeakMap(_)
                | Value::Set(_)
                | Value::WeakSet(_)
                | Value::Blob(_)
                | Value::ArrayBuffer(_)
                | Value::TypedArray(_)
                | Value::StringConstructor
                | Value::TypedArrayConstructor(_)
                | Value::BlobConstructor
                | Value::UrlConstructor
                | Value::ArrayBufferConstructor
                | Value::PromiseConstructor
                | Value::MapConstructor
                | Value::WeakMapConstructor
                | Value::SetConstructor
                | Value::WeakSetConstructor
                | Value::SymbolConstructor
                | Value::RegExpConstructor
                | Value::PromiseCapability(_)
                | Value::RegExp(_)
                | Value::Date(_)
                | Value::Node(_)
                | Value::NodeList(_)
                | Value::FormData(_)
                | Value::Function(_)
        )
    }

    pub(crate) fn to_primitive_for_loose(&self, value: &Value) -> Value {
        match value {
            Value::Object(entries) => {
                if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries.borrow()) {
                    return Value::String(wrapped);
                }
                if let Some(id) = Self::symbol_wrapper_id_from_object(&entries.borrow()) {
                    if let Some(symbol) = self.symbol_runtime.symbols_by_id.get(&id) {
                        return Value::Symbol(symbol.clone());
                    }
                }
                Value::String(value.as_string())
            }
            Value::Array(_)
            | Value::Promise(_)
            | Value::Map(_)
            | Value::WeakMap(_)
            | Value::Set(_)
            | Value::WeakSet(_)
            | Value::Blob(_)
            | Value::ArrayBuffer(_)
            | Value::TypedArray(_)
            | Value::StringConstructor
            | Value::TypedArrayConstructor(_)
            | Value::BlobConstructor
            | Value::UrlConstructor
            | Value::ArrayBufferConstructor
            | Value::PromiseConstructor
            | Value::MapConstructor
            | Value::WeakMapConstructor
            | Value::SetConstructor
            | Value::WeakSetConstructor
            | Value::SymbolConstructor
            | Value::RegExpConstructor
            | Value::PromiseCapability(_)
            | Value::RegExp(_)
            | Value::Date(_)
            | Value::Node(_)
            | Value::NodeList(_)
            | Value::FormData(_)
            | Value::Function(_) => Value::String(value.as_string()),
            _ => value.clone(),
        }
    }

    fn object_has_property_in_chain(entries: &ObjectValue, key: &str) -> bool {
        if Self::object_get_entry(entries, key).is_some() {
            return true;
        }

        let mut prototype = Self::object_get_entry(entries, INTERNAL_OBJECT_PROTOTYPE_KEY);
        while let Some(Value::Object(object)) = prototype {
            let object_ref = object.borrow();
            if Self::object_get_entry(&object_ref, key).is_some() {
                return true;
            }
            prototype = Self::object_get_entry(&object_ref, INTERNAL_OBJECT_PROTOTYPE_KEY);
        }
        false
    }

    pub(crate) fn value_in(&self, left: &Value, right: &Value) -> Result<bool> {
        if Self::is_primitive_value(right) {
            return Err(Error::ScriptRuntime(
                "right-hand side of in must be an object".into(),
            ));
        }

        let key = self.property_key_to_storage_key(left);
        let has_property = match right {
            Value::NodeList(nodes) => {
                if key == "length" {
                    true
                } else {
                    self.value_as_index(left)
                        .is_some_and(|index| index < nodes.len())
                }
            }
            Value::Array(values) => {
                let values = values.borrow();
                if key == "length" || Self::object_get_entry(&values.properties, &key).is_some() {
                    true
                } else {
                    self.value_as_index(left).is_some_and(|index| {
                        index < values.len() && !Self::array_index_is_hole(&values, index)
                    })
                }
            }
            Value::TypedArray(values) => {
                if key == "length" {
                    true
                } else {
                    self.value_as_index(left)
                        .is_some_and(|index| index < values.borrow().observed_length())
                }
            }
            Value::Object(entries) => {
                let entries = entries.borrow();
                if let Some(text) = Self::string_wrapper_value_from_object(&entries) {
                    if key == "length" || key == "constructor" {
                        return Ok(true);
                    }
                    if key
                        .parse::<usize>()
                        .ok()
                        .is_some_and(|index| text.chars().nth(index).is_some())
                    {
                        return Ok(true);
                    }
                }
                Self::object_has_property_in_chain(&entries, &key)
            }
            Value::FormData(entries) => entries.iter().any(|(name, _)| name == &key),
            _ => false,
        };

        Ok(has_property)
    }

    pub(crate) fn value_instance_of(&mut self, left: &Value, right: &Value) -> Result<bool> {
        if let (Value::Object(left_obj), Value::Object(right_obj)) = (left, right) {
            if Self::is_iterator_constructor_object(&right_obj.borrow()) {
                return Ok(Self::is_iterator_object(&left_obj.borrow()));
            }
        }

        if let Value::Node(node) = left {
            if self.is_named_constructor_value(right, "HTMLElement") {
                return Ok(self.dom.element(*node).is_some());
            }
            if self.is_named_constructor_value(right, "HTMLInputElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("input"))
                    .unwrap_or(false));
            }
        }

        if matches!(right, Value::BlobConstructor) {
            return Ok(matches!(left, Value::Blob(_)));
        }
        if matches!(right, Value::UrlConstructor) {
            return Ok(matches!(left, Value::Object(left_obj) if Self::is_url_object(&left_obj.borrow())));
        }
        if matches!(right, Value::StringConstructor) {
            return Ok(matches!(left, Value::Object(left_obj) if Self::string_wrapper_value_from_object(&left_obj.borrow()).is_some()));
        }

        if !Self::is_instanceof_rhs_object_like(right) {
            return Err(Error::ScriptRuntime(
                "right-hand side of instanceof is not an object".into(),
            ));
        }

        let has_instance_symbol = self.eval_symbol_static_property(SymbolStaticProperty::HasInstance);
        let has_instance_key = self.property_key_to_storage_key(&has_instance_symbol);
        let has_instance = self.object_property_from_value(right, &has_instance_key)?;
        if !matches!(has_instance, Value::Undefined) {
            if !self.is_callable_value(&has_instance) {
                return Err(Error::ScriptRuntime(
                    "Symbol.hasInstance is not callable".into(),
                ));
            }
            let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
            let verdict = self.execute_callable_value_with_this_and_env(
                &has_instance,
                std::slice::from_ref(left),
                &event,
                None,
                Some(right.clone()),
            )?;
            return Ok(verdict.truthy());
        }

        if !self.is_callable_value(right) {
            return Err(Error::ScriptRuntime(
                "right-hand side of instanceof is not callable".into(),
            ));
        }

        let prototype = self.object_property_from_value(right, "prototype")?;
        let Value::Object(expected_prototype) = prototype else {
            return Err(Error::ScriptRuntime(
                "instanceof prototype is not an object".into(),
            ));
        };

        Ok(Self::value_prototype_chain_contains(left, &expected_prototype))
    }

    fn is_instanceof_rhs_object_like(value: &Value) -> bool {
        matches!(
            value,
            Value::Object(_)
                | Value::Array(_)
                | Value::Function(_)
                | Value::Map(_)
                | Value::WeakMap(_)
                | Value::Set(_)
                | Value::WeakSet(_)
                | Value::Promise(_)
                | Value::TypedArray(_)
                | Value::Blob(_)
                | Value::ArrayBuffer(_)
                | Value::StringConstructor
                | Value::TypedArrayConstructor(_)
                | Value::BlobConstructor
                | Value::UrlConstructor
                | Value::ArrayBufferConstructor
                | Value::PromiseConstructor
                | Value::MapConstructor
                | Value::WeakMapConstructor
                | Value::SetConstructor
                | Value::WeakSetConstructor
                | Value::SymbolConstructor
                | Value::RegExpConstructor
                | Value::PromiseCapability(_)
                | Value::RegExp(_)
                | Value::Date(_)
                | Value::Node(_)
                | Value::NodeList(_)
                | Value::FormData(_)
        )
    }

    fn value_prototype_chain_contains(left: &Value, expected: &Rc<RefCell<ObjectValue>>) -> bool {
        let mut prototype = Self::value_internal_prototype(left);
        while let Some(Value::Object(current)) = prototype {
            if Rc::ptr_eq(&current, expected) {
                return true;
            }
            let current_ref = current.borrow();
            prototype = Self::object_get_entry(&current_ref, INTERNAL_OBJECT_PROTOTYPE_KEY);
        }
        false
    }

    fn value_internal_prototype(value: &Value) -> Option<Value> {
        match value {
            Value::Object(entries) => {
                Self::object_get_entry(&entries.borrow(), INTERNAL_OBJECT_PROTOTYPE_KEY)
            }
            Value::Array(values) => {
                Self::object_get_entry(&values.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
            }
            Value::Map(map) => {
                Self::object_get_entry(&map.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
            }
            Value::WeakMap(map) => {
                Self::object_get_entry(&map.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
            }
            Value::Set(set) => {
                Self::object_get_entry(&set.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
            }
            Value::WeakSet(set) => {
                Self::object_get_entry(&set.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
            }
            Value::RegExp(regex) => {
                Self::object_get_entry(&regex.borrow().properties, INTERNAL_OBJECT_PROTOTYPE_KEY)
            }
            Value::Node(_) | Value::NodeList(_) | Value::Date(_) | Value::Function(_) => None,
            _ => None,
        }
    }

    pub(crate) fn is_named_constructor_value(&self, value: &Value, name: &str) -> bool {
        self.script_runtime
            .env
            .get(name)
            .is_some_and(|expected| self.strict_equal(value, expected))
    }

    pub(crate) fn value_as_index(&self, value: &Value) -> Option<usize> {
        match value {
            Value::Number(v) => usize::try_from(*v).ok(),
            Value::Float(v) => {
                if !v.is_finite() || v.fract() != 0.0 || *v < 0.0 {
                    None
                } else {
                    usize::try_from(*v as i64).ok()
                }
            }
            Value::BigInt(v) => v.to_usize(),
            Value::String(s) => {
                if let Ok(int) = s.parse::<i64>() {
                    usize::try_from(int).ok()
                } else if let Ok(float) = s.parse::<f64>() {
                    if float.fract() == 0.0 && float >= 0.0 {
                        usize::try_from(float as i64).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn strict_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::Float(l), Value::Float(r)) => l == r,
            (Value::Number(l), Value::Float(r)) => (*l as f64) == *r,
            (Value::Float(l), Value::Number(r)) => *l == (*r as f64),
            (Value::BigInt(l), Value::BigInt(r)) => l == r,
            (Value::Symbol(l), Value::Symbol(r)) => l.id == r.id,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Node(l), Value::Node(r)) => l == r,
            (Value::Array(l), Value::Array(r)) => Rc::ptr_eq(l, r),
            (Value::Map(l), Value::Map(r)) => Rc::ptr_eq(l, r),
            (Value::WeakMap(l), Value::WeakMap(r)) => Rc::ptr_eq(l, r),
            (Value::Set(l), Value::Set(r)) => Rc::ptr_eq(l, r),
            (Value::WeakSet(l), Value::WeakSet(r)) => Rc::ptr_eq(l, r),
            (Value::Promise(l), Value::Promise(r)) => Rc::ptr_eq(l, r),
            (Value::TypedArray(l), Value::TypedArray(r)) => Rc::ptr_eq(l, r),
            (Value::Blob(l), Value::Blob(r)) => Rc::ptr_eq(l, r),
            (Value::ArrayBuffer(l), Value::ArrayBuffer(r)) => Rc::ptr_eq(l, r),
            (Value::StringConstructor, Value::StringConstructor) => true,
            (Value::TypedArrayConstructor(l), Value::TypedArrayConstructor(r)) => l == r,
            (Value::BlobConstructor, Value::BlobConstructor) => true,
            (Value::UrlConstructor, Value::UrlConstructor) => true,
            (Value::ArrayBufferConstructor, Value::ArrayBufferConstructor) => true,
            (Value::PromiseConstructor, Value::PromiseConstructor) => true,
            (Value::MapConstructor, Value::MapConstructor) => true,
            (Value::WeakMapConstructor, Value::WeakMapConstructor) => true,
            (Value::SetConstructor, Value::SetConstructor) => true,
            (Value::WeakSetConstructor, Value::WeakSetConstructor) => true,
            (Value::SymbolConstructor, Value::SymbolConstructor) => true,
            (Value::RegExpConstructor, Value::RegExpConstructor) => true,
            (Value::PromiseCapability(l), Value::PromiseCapability(r)) => Rc::ptr_eq(l, r),
            (Value::Object(l), Value::Object(r)) => Rc::ptr_eq(l, r),
            (Value::RegExp(l), Value::RegExp(r)) => Rc::ptr_eq(l, r),
            (Value::Date(l), Value::Date(r)) => Rc::ptr_eq(l, r),
            (Value::Function(l), Value::Function(r)) => Rc::ptr_eq(l, r),
            (Value::FormData(l), Value::FormData(r)) => l == r,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            _ => false,
        }
    }

    pub(crate) fn compare<F>(&self, left: &Value, right: &Value, op: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        let ordering_to_cmp = |ordering: std::cmp::Ordering| match ordering {
            std::cmp::Ordering::Less => -1.0,
            std::cmp::Ordering::Equal => 0.0,
            std::cmp::Ordering::Greater => 1.0,
        };
        match (left, right) {
            (Value::String(l), Value::String(r)) => {
                return op(ordering_to_cmp(l.cmp(r)), 0.0);
            }
            (Value::String(l), Value::BigInt(r)) => {
                let Ok(parsed) = Self::parse_js_bigint_from_string(l) else {
                    return false;
                };
                return op(ordering_to_cmp(parsed.cmp(r)), 0.0);
            }
            (Value::BigInt(l), Value::String(r)) => {
                let Ok(parsed) = Self::parse_js_bigint_from_string(r) else {
                    return false;
                };
                return op(ordering_to_cmp(l.cmp(&parsed)), 0.0);
            }
            (Value::BigInt(l), Value::BigInt(r)) => {
                return op(ordering_to_cmp(l.cmp(r)), 0.0);
            }
            (Value::BigInt(l), Value::Number(_) | Value::Float(_)) => {
                let r = Self::coerce_number_for_global(right);
                if r.is_nan() {
                    return false;
                }
                if let Some(rb) = Self::f64_to_bigint_if_integral(r) {
                    return op(
                        l.to_f64().unwrap_or_else(|| {
                            if l.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                        rb.to_f64().unwrap_or_else(|| {
                            if rb.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                    );
                }
                return op(
                    l.to_f64().unwrap_or_else(|| {
                        if l.sign() == Sign::Minus {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        }
                    }),
                    r,
                );
            }
            (Value::Number(_) | Value::Float(_), Value::BigInt(r)) => {
                let l = Self::coerce_number_for_global(left);
                if l.is_nan() {
                    return false;
                }
                if let Some(lb) = Self::f64_to_bigint_if_integral(l) {
                    return op(
                        lb.to_f64().unwrap_or_else(|| {
                            if lb.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                        r.to_f64().unwrap_or_else(|| {
                            if r.sign() == Sign::Minus {
                                f64::NEG_INFINITY
                            } else {
                                f64::INFINITY
                            }
                        }),
                    );
                }
                return op(
                    l,
                    r.to_f64().unwrap_or_else(|| {
                        if r.sign() == Sign::Minus {
                            f64::NEG_INFINITY
                        } else {
                            f64::INFINITY
                        }
                    }),
                );
            }
            _ => {}
        }
        let l = Self::coerce_number_for_global(left);
        let r = Self::coerce_number_for_global(right);
        op(l, r)
    }

    pub(crate) fn number_bigint_loose_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::BigInt(l), Value::Number(r)) => *l == JsBigInt::from(*r),
            (Value::BigInt(l), Value::Float(r)) => {
                Self::f64_to_bigint_if_integral(*r).is_some_and(|rb| rb == *l)
            }
            (Value::Number(l), Value::BigInt(r)) => JsBigInt::from(*l) == *r,
            (Value::Float(l), Value::BigInt(r)) => {
                Self::f64_to_bigint_if_integral(*l).is_some_and(|lb| lb == *r)
            }
            _ => false,
        }
    }

    pub(crate) fn f64_to_bigint_if_integral(value: f64) -> Option<JsBigInt> {
        if !value.is_finite() || value.fract() != 0.0 {
            return None;
        }
        if value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            return Some(JsBigInt::from(value as i64));
        }
        let rendered = format!("{value:.0}");
        JsBigInt::parse_bytes(rendered.as_bytes(), 10)
    }

    fn invoke_primitive_coercion_method(
        &mut self,
        receiver: &Value,
        method: &Value,
    ) -> Result<Value> {
        let event = EventState::new("script", self.dom.root, self.scheduler.now_ms);
        self.execute_callable_value_with_this_and_env(
            method,
            &[],
            &event,
            None,
            Some(receiver.clone()),
        )
    }

    fn to_primitive_for_addition(&mut self, value: &Value) -> Result<Value> {
        if Self::is_primitive_value(value) || matches!(value, Value::Date(_)) {
            return Ok(value.clone());
        }

        if let Value::Object(entries) = value {
            let entries = entries.borrow();
            if let Some(wrapped) = Self::string_wrapper_value_from_object(&entries) {
                return Ok(Value::String(wrapped));
            }
            if let Some(id) = Self::symbol_wrapper_id_from_object(&entries) {
                if let Some(symbol) = self.symbol_runtime.symbols_by_id.get(&id) {
                    return Ok(Value::Symbol(symbol.clone()));
                }
            }
        }

        for method_name in ["valueOf", "toString"] {
            let method = match self.object_property_from_value(value, method_name) {
                Ok(method) => method,
                Err(Error::ScriptRuntime(msg)) if msg == "value is not an object" => {
                    return Ok(value.clone());
                }
                Err(other) => return Err(other),
            };
            if !self.is_callable_value(&method) {
                continue;
            }
            let coerced = self.invoke_primitive_coercion_method(value, &method)?;
            if Self::is_primitive_value(&coerced) {
                return Ok(coerced);
            }
        }

        Ok(Value::String(value.as_string()))
    }

    pub(crate) fn add_values(&mut self, left: &Value, right: &Value) -> Result<Value> {
        let left = self.to_primitive_for_addition(left)?;
        let right = self.to_primitive_for_addition(right)?;

        if matches!(left, Value::Symbol(_)) || matches!(right, Value::Symbol(_)) {
            return Err(Error::ScriptRuntime(
                "Cannot convert a Symbol value to a string".into(),
            ));
        }
        if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) {
            return Ok(Value::String(format!(
                "{}{}",
                left.as_string(),
                right.as_string()
            )));
        }

        if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
            return match (&left, &right) {
                (Value::BigInt(l), Value::BigInt(r)) => Ok(Value::BigInt(l + r)),
                _ => Err(Error::ScriptRuntime(
                    "cannot mix BigInt and other types in addition".into(),
                )),
            };
        }

        match (&left, &right) {
            (Value::Number(l), Value::Number(r)) => {
                if let Some(sum) = l.checked_add(*r) {
                    Ok(Value::Number(sum))
                } else {
                    Ok(Value::Float((*l as f64) + (*r as f64)))
                }
            }
            _ => Ok(Value::Float(
                Self::coerce_number_for_global(&left) + Self::coerce_number_for_global(&right),
            )),
        }
    }
}
