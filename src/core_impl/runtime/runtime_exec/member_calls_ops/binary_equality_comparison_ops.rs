use super::*;

impl Harness {
    pub(crate) fn value_instance_of(&mut self, left: &Value, right: &Value) -> Result<bool> {
        if let (Value::Object(left_obj), Value::Object(right_obj)) = (left, right) {
            if Self::is_iterator_constructor_object(&right_obj.borrow()) {
                return Ok(Self::is_iterator_object(&left_obj.borrow()));
            }
        }

        if let Value::Node(node) = left {
            if self.is_named_constructor_value(right, "Element") {
                return Ok(self.dom.element(*node).is_some());
            }
            if self.is_named_constructor_value(right, "HTMLElement") {
                return Ok(self.dom.element(*node).is_some());
            }
            if self.is_named_constructor_value(right, "HTMLAnchorElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("a"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLAreaElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("area"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLBodyElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("body"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLBRElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("br"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLBaseElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("base"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLAudioElement")
                || self.is_named_constructor_value(right, "Audio")
            {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("audio"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLButtonElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("button"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLCanvasElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("canvas"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLDataElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("data"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLDataListElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("datalist"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLInputElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("input"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLSelectElement") {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("select"))
                    .unwrap_or(false));
            }
            if self.is_named_constructor_value(right, "HTMLOptionElement")
                || self.is_named_constructor_value(right, "Option")
            {
                return Ok(self
                    .dom
                    .tag_name(*node)
                    .map(|tag| tag.eq_ignore_ascii_case("option"))
                    .unwrap_or(false));
            }
        }

        if matches!(right, Value::BlobConstructor) {
            return Ok(matches!(left, Value::Blob(_)));
        }
        if matches!(right, Value::UrlConstructor) {
            return Ok(
                matches!(left, Value::Object(left_obj) if Self::is_url_object(&left_obj.borrow())),
            );
        }
        if matches!(right, Value::StringConstructor) {
            return Ok(
                matches!(left, Value::Object(left_obj) if Self::string_wrapper_value_from_object(&left_obj.borrow()).is_some()),
            );
        }
        if matches!(
            Self::callable_kind_from_value(right),
            Some("bound_function")
        ) {
            let (target, _bound_this, _bound_args) = Self::bound_callable_components(right)?;
            return self.value_instance_of(left, &target);
        }

        if !Self::is_instanceof_rhs_object_like(right) {
            return Err(Error::ScriptRuntime(
                "right-hand side of instanceof is not an object".into(),
            ));
        }

        let has_instance_symbol =
            self.eval_symbol_static_property(SymbolStaticProperty::HasInstance);
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

        Ok(self.value_prototype_chain_contains(left, &expected_prototype))
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
                | Value::UrlSearchParamsConstructor
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

    fn value_prototype_chain_contains(
        &mut self,
        left: &Value,
        expected: &Rc<RefCell<ObjectValue>>,
    ) -> bool {
        let mut prototype = self.value_internal_prototype_value(left);
        while let Some(current) = prototype {
            if matches!(current, Value::Null | Value::Undefined) {
                break;
            }
            if let Value::Object(object) = &current
                && Rc::ptr_eq(object, expected)
            {
                return true;
            }
            prototype = self.value_internal_prototype_value(&current);
        }
        false
    }

    pub(crate) fn is_named_constructor_value(&self, value: &Value, name: &str) -> bool {
        self.script_runtime
            .env
            .get(name)
            .is_some_and(|expected| self.strict_equal(value, expected))
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
            (Value::NodeList(l), Value::NodeList(r)) => Rc::ptr_eq(l, r),
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
            (Value::UrlSearchParamsConstructor, Value::UrlSearchParamsConstructor) => true,
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
}
