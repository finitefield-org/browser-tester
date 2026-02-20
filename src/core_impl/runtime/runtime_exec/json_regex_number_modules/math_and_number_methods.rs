impl Harness {
    pub(crate) fn eval_math_method(
        &mut self,
        method: MathMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }

        let single = |values: &[Value]| Self::coerce_number_for_global(&values[0]);

        match method {
            MathMethod::Abs => Ok(Value::Float(single(&values).abs())),
            MathMethod::Acos => Ok(Value::Float(single(&values).acos())),
            MathMethod::Acosh => Ok(Value::Float(single(&values).acosh())),
            MathMethod::Asin => Ok(Value::Float(single(&values).asin())),
            MathMethod::Asinh => Ok(Value::Float(single(&values).asinh())),
            MathMethod::Atan => Ok(Value::Float(single(&values).atan())),
            MathMethod::Atan2 => Ok(Value::Float(
                Self::coerce_number_for_global(&values[0])
                    .atan2(Self::coerce_number_for_global(&values[1])),
            )),
            MathMethod::Atanh => Ok(Value::Float(single(&values).atanh())),
            MathMethod::Cbrt => Ok(Value::Float(single(&values).cbrt())),
            MathMethod::Ceil => Ok(Value::Float(single(&values).ceil())),
            MathMethod::Clz32 => Ok(Value::Number(i64::from(
                Self::to_u32_for_math(&values[0]).leading_zeros(),
            ))),
            MathMethod::Cos => Ok(Value::Float(single(&values).cos())),
            MathMethod::Cosh => Ok(Value::Float(single(&values).cosh())),
            MathMethod::Exp => Ok(Value::Float(single(&values).exp())),
            MathMethod::Expm1 => Ok(Value::Float(single(&values).exp_m1())),
            MathMethod::Floor => Ok(Value::Float(single(&values).floor())),
            MathMethod::F16Round => Ok(Value::Float(Self::math_f16round(single(&values)))),
            MathMethod::FRound => Ok(Value::Float((single(&values) as f32) as f64)),
            MathMethod::Hypot => {
                let mut sum = 0.0f64;
                for value in values {
                    let value = Self::coerce_number_for_global(&value);
                    sum += value * value;
                }
                Ok(Value::Float(sum.sqrt()))
            }
            MathMethod::Imul => {
                let left = Self::to_i32_for_math(&values[0]);
                let right = Self::to_i32_for_math(&values[1]);
                Ok(Value::Number(i64::from(left.wrapping_mul(right))))
            }
            MathMethod::Log => Ok(Value::Float(single(&values).ln())),
            MathMethod::Log10 => Ok(Value::Float(single(&values).log10())),
            MathMethod::Log1p => Ok(Value::Float(single(&values).ln_1p())),
            MathMethod::Log2 => Ok(Value::Float(single(&values).log2())),
            MathMethod::Max => {
                let mut out = f64::NEG_INFINITY;
                for value in values {
                    out = out.max(Self::coerce_number_for_global(&value));
                }
                Ok(Value::Float(out))
            }
            MathMethod::Min => {
                let mut out = f64::INFINITY;
                for value in values {
                    out = out.min(Self::coerce_number_for_global(&value));
                }
                Ok(Value::Float(out))
            }
            MathMethod::Pow => Ok(Value::Float(
                Self::coerce_number_for_global(&values[0])
                    .powf(Self::coerce_number_for_global(&values[1])),
            )),
            MathMethod::Random => Ok(Value::Float(self.next_random_f64())),
            MathMethod::Round => Ok(Value::Float(Self::js_math_round(single(&values)))),
            MathMethod::Sign => Ok(Value::Float(Self::js_math_sign(single(&values)))),
            MathMethod::Sin => Ok(Value::Float(single(&values).sin())),
            MathMethod::Sinh => Ok(Value::Float(single(&values).sinh())),
            MathMethod::Sqrt => Ok(Value::Float(single(&values).sqrt())),
            MathMethod::SumPrecise => match &values[0] {
                Value::Array(values) => Ok(Value::Float(Self::sum_precise(&values.borrow()))),
                _ => Err(Error::ScriptRuntime(
                    "Math.sumPrecise argument must be an array".into(),
                )),
            },
            MathMethod::Tan => Ok(Value::Float(single(&values).tan())),
            MathMethod::Tanh => Ok(Value::Float(single(&values).tanh())),
            MathMethod::Trunc => Ok(Value::Float(single(&values).trunc())),
        }
    }

    pub(crate) fn eval_number_method(
        &mut self,
        method: NumberMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }

        match method {
            NumberMethod::IsFinite => Ok(Value::Bool(
                Self::number_primitive_value(&values[0]).is_some_and(f64::is_finite),
            )),
            NumberMethod::IsInteger => Ok(Value::Bool(
                Self::number_primitive_value(&values[0])
                    .is_some_and(|value| value.is_finite() && value.fract() == 0.0),
            )),
            NumberMethod::IsNaN => Ok(Value::Bool(matches!(
                values[0],
                Value::Float(value) if value.is_nan()
            ))),
            NumberMethod::IsSafeInteger => Ok(Value::Bool(
                Self::number_primitive_value(&values[0]).is_some_and(|value| {
                    value.is_finite()
                        && value.fract() == 0.0
                        && value.abs() <= 9_007_199_254_740_991.0
                }),
            )),
            NumberMethod::ParseFloat => {
                Ok(Value::Float(parse_js_parse_float(&values[0].as_string())))
            }
            NumberMethod::ParseInt => {
                let radix = if values.len() == 2 {
                    Some(Self::value_to_i64(&values[1]))
                } else {
                    None
                };
                Ok(Value::Float(parse_js_parse_int(
                    &values[0].as_string(),
                    radix,
                )))
            }
        }
    }

    pub(crate) fn eval_number_instance_method(
        &mut self,
        method: NumberInstanceMethod,
        value: &Expr,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let value = self.eval_expr(value, env, event_param, event)?;
        let mut args_value = Vec::with_capacity(args.len());
        for arg in args {
            args_value.push(self.eval_expr(arg, env, event_param, event)?);
        }

        if let Value::BigInt(bigint) = &value {
            return match method {
                NumberInstanceMethod::ToLocaleString => Ok(Value::String(bigint.to_string())),
                NumberInstanceMethod::ToString => {
                    let radix = if let Some(arg) = args_value.first() {
                        let radix = Self::value_to_i64(arg);
                        if !(2..=36).contains(&radix) {
                            return Err(Error::ScriptRuntime(
                                "toString radix must be between 2 and 36".into(),
                            ));
                        }
                        radix as u32
                    } else {
                        10
                    };
                    Ok(Value::String(bigint.to_str_radix(radix)))
                }
                NumberInstanceMethod::ValueOf => Ok(Value::BigInt(bigint.clone())),
                NumberInstanceMethod::ToExponential
                | NumberInstanceMethod::ToFixed
                | NumberInstanceMethod::ToPrecision => Err(Error::ScriptRuntime(
                    "number formatting methods are not supported for BigInt values".into(),
                )),
            };
        }

        if let Value::Symbol(symbol) = &value {
            return match method {
                NumberInstanceMethod::ValueOf => Ok(Value::Symbol(symbol.clone())),
                NumberInstanceMethod::ToString | NumberInstanceMethod::ToLocaleString => {
                    if !args_value.is_empty() {
                        return Err(Error::ScriptRuntime(
                            "Symbol.toString does not take arguments".into(),
                        ));
                    }
                    Ok(Value::String(Value::Symbol(symbol.clone()).as_string()))
                }
                NumberInstanceMethod::ToExponential
                | NumberInstanceMethod::ToFixed
                | NumberInstanceMethod::ToPrecision => Err(Error::ScriptRuntime(
                    "Cannot convert a Symbol value to a number".into(),
                )),
            };
        }

        let numeric = Self::coerce_number_for_number_constructor(&value);

        match method {
            NumberInstanceMethod::ToExponential => {
                let fraction_digits = if let Some(arg) = args_value.first() {
                    let fraction_digits = Self::value_to_i64(arg);
                    if !(0..=100).contains(&fraction_digits) {
                        return Err(Error::ScriptRuntime(
                            "toExponential fractionDigits must be between 0 and 100".into(),
                        ));
                    }
                    Some(fraction_digits as usize)
                } else {
                    None
                };
                Ok(Value::String(Self::number_to_exponential(
                    numeric,
                    fraction_digits,
                )))
            }
            NumberInstanceMethod::ToFixed => {
                let fraction_digits = if let Some(arg) = args_value.first() {
                    let fraction_digits = Self::value_to_i64(arg);
                    if !(0..=100).contains(&fraction_digits) {
                        return Err(Error::ScriptRuntime(
                            "toFixed fractionDigits must be between 0 and 100".into(),
                        ));
                    }
                    fraction_digits as usize
                } else {
                    0
                };
                Ok(Value::String(Self::number_to_fixed(
                    numeric,
                    fraction_digits,
                )))
            }
            NumberInstanceMethod::ToLocaleString => {
                Ok(Value::String(Self::format_number_default(numeric)))
            }
            NumberInstanceMethod::ToPrecision => {
                if let Some(arg) = args_value.first() {
                    let precision = Self::value_to_i64(arg);
                    if !(1..=100).contains(&precision) {
                        return Err(Error::ScriptRuntime(
                            "toPrecision precision must be between 1 and 100".into(),
                        ));
                    }
                    Ok(Value::String(Self::number_to_precision(
                        numeric,
                        precision as usize,
                    )))
                } else {
                    Ok(Value::String(Self::format_number_default(numeric)))
                }
            }
            NumberInstanceMethod::ToString => {
                let radix = if let Some(arg) = args_value.first() {
                    let radix = Self::value_to_i64(arg);
                    if !(2..=36).contains(&radix) {
                        return Err(Error::ScriptRuntime(
                            "toString radix must be between 2 and 36".into(),
                        ));
                    }
                    radix as u32
                } else {
                    10
                };
                Ok(Value::String(Self::number_to_string_radix(numeric, radix)))
            }
            NumberInstanceMethod::ValueOf => Ok(Self::number_value(numeric)),
        }
    }

    pub(crate) fn eval_bigint_method(
        &mut self,
        method: BigIntMethod,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.eval_expr(arg, env, event_param, event)?);
        }
        let bits_i64 = Self::value_to_i64(&values[0]);
        if bits_i64 < 0 {
            return Err(Error::ScriptRuntime(
                "BigInt bit width must be a non-negative integer".into(),
            ));
        }
        let bits = usize::try_from(bits_i64)
            .map_err(|_| Error::ScriptRuntime("BigInt bit width is too large".into()))?;
        let value = Self::coerce_bigint_for_builtin_op(&values[1])?;
        let out = match method {
            BigIntMethod::AsIntN => Self::bigint_as_int_n(bits, &value),
            BigIntMethod::AsUintN => Self::bigint_as_uint_n(bits, &value),
        };
        Ok(Value::BigInt(out))
    }

    pub(crate) fn eval_bigint_instance_method(
        &mut self,
        method: BigIntInstanceMethod,
        value: &Expr,
        args: &[Expr],
        env: &HashMap<String, Value>,
        event_param: &Option<String>,
        event: &EventState,
    ) -> Result<Value> {
        let value = self.eval_expr(value, env, event_param, event)?;
        let value = Self::coerce_bigint_for_builtin_op(&value)?;
        let mut args_value = Vec::with_capacity(args.len());
        for arg in args {
            args_value.push(self.eval_expr(arg, env, event_param, event)?);
        }

        match method {
            BigIntInstanceMethod::ToLocaleString => Ok(Value::String(value.to_string())),
            BigIntInstanceMethod::ToString => {
                let radix = if let Some(arg) = args_value.first() {
                    let radix = Self::value_to_i64(arg);
                    if !(2..=36).contains(&radix) {
                        return Err(Error::ScriptRuntime(
                            "toString radix must be between 2 and 36".into(),
                        ));
                    }
                    radix as u32
                } else {
                    10
                };
                Ok(Value::String(value.to_str_radix(radix)))
            }
            BigIntInstanceMethod::ValueOf => Ok(Value::BigInt(value)),
        }
    }

}
