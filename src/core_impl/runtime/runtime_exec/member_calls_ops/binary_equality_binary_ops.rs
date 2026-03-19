use super::*;

impl Harness {
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
                Value::Float(
                    Self::coerce_number_for_global(left) - Self::coerce_number_for_global(right),
                )
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
                        return Err(Error::ScriptRuntime("division by zero".into()));
                    }
                    return Ok(Value::BigInt(l % r));
                }
                if matches!(left, Value::BigInt(_)) || matches!(right, Value::BigInt(_)) {
                    return Err(Error::ScriptRuntime(
                        "cannot mix BigInt and other types in arithmetic operations".into(),
                    ));
                }
                Value::Float(self.numeric_value(left) % self.numeric_value(right))
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

    pub(crate) fn to_primitive_for_addition(&mut self, value: &Value) -> Result<Value> {
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
            let left = left.as_string();
            let right = right.as_string();
            let mut out = String::with_capacity(left.len() + right.len());
            out.push_str(&left);
            out.push_str(&right);
            return Ok(Value::String(out));
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
