pub(super) fn validate_math_arity(method: MathMethod, count: usize) -> Result<()> {
    let method_name = match method {
        MathMethod::Abs => "abs",
        MathMethod::Acos => "acos",
        MathMethod::Acosh => "acosh",
        MathMethod::Asin => "asin",
        MathMethod::Asinh => "asinh",
        MathMethod::Atan => "atan",
        MathMethod::Atan2 => "atan2",
        MathMethod::Atanh => "atanh",
        MathMethod::Cbrt => "cbrt",
        MathMethod::Ceil => "ceil",
        MathMethod::Clz32 => "clz32",
        MathMethod::Cos => "cos",
        MathMethod::Cosh => "cosh",
        MathMethod::Exp => "exp",
        MathMethod::Expm1 => "expm1",
        MathMethod::Floor => "floor",
        MathMethod::F16Round => "f16round",
        MathMethod::FRound => "fround",
        MathMethod::Hypot => "hypot",
        MathMethod::Imul => "imul",
        MathMethod::Log => "log",
        MathMethod::Log10 => "log10",
        MathMethod::Log1p => "log1p",
        MathMethod::Log2 => "log2",
        MathMethod::Max => "max",
        MathMethod::Min => "min",
        MathMethod::Pow => "pow",
        MathMethod::Random => "random",
        MathMethod::Round => "round",
        MathMethod::Sign => "sign",
        MathMethod::Sin => "sin",
        MathMethod::Sinh => "sinh",
        MathMethod::Sqrt => "sqrt",
        MathMethod::SumPrecise => "sumPrecise",
        MathMethod::Tan => "tan",
        MathMethod::Tanh => "tanh",
        MathMethod::Trunc => "trunc",
    };

    let valid = match method {
        MathMethod::Random => count == 0,
        MathMethod::Atan2 | MathMethod::Imul | MathMethod::Pow => count == 2,
        MathMethod::Hypot | MathMethod::Max | MathMethod::Min => true,
        MathMethod::SumPrecise => count == 1,
        _ => count == 1,
    };

    if valid {
        return Ok(());
    }

    let message = match method {
        MathMethod::Random => format!("Math.{method_name} does not take arguments"),
        MathMethod::Atan2 | MathMethod::Imul | MathMethod::Pow => {
            format!("Math.{method_name} requires exactly two arguments")
        }
        MathMethod::SumPrecise => format!("Math.{method_name} requires exactly one argument"),
        _ => format!("Math.{method_name} requires exactly one argument"),
    };
    Err(Error::ScriptParse(message))
}

pub(super) fn parse_string_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("String") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "String supports zero or one argument".into(),
            ));
        }
        let value = if let Some(arg) = args.first() {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse("String argument cannot be empty".into()));
            }
            Some(Box::new(parse_expr(arg)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::StringConstruct {
            value,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::StringConstruct {
                value: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let method = match member.as_str() {
            "fromCharCode" => StringStaticMethod::FromCharCode,
            "fromCodePoint" => StringStaticMethod::FromCodePoint,
            "raw" => StringStaticMethod::Raw,
            _ => return Ok(None),
        };

        if matches!(method, StringStaticMethod::Raw) && args.is_empty() {
            return Err(Error::ScriptParse(
                "String.raw requires at least one argument".into(),
            ));
        }

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "String.{member} argument cannot be empty"
                )));
            }
            parsed.push(parse_expr(arg)?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::StringStaticMethod {
            method,
            args: parsed,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::StringConstructor))
}

pub(super) fn parse_number_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut has_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        has_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Number") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if has_new && cursor.peek() != Some(b'(') {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::NumberConstruct { value: None }));
        }
        return Ok(None);
    }

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Number supports zero or one argument".into(),
            ));
        }
        let value = if let Some(arg) = args.first() {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse("Number argument cannot be empty".into()));
            }
            Some(Box::new(parse_expr(arg)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::NumberConstruct { value }));
    }

    if has_new {
        return Ok(None);
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let Some(method) = parse_number_method_name(&member) else {
            return Ok(None);
        };

        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let parsed = match method {
            NumberMethod::IsFinite
            | NumberMethod::IsInteger
            | NumberMethod::IsNaN
            | NumberMethod::IsSafeInteger
            | NumberMethod::ParseFloat => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(format!(
                        "Number.{member} requires exactly one argument"
                    )));
                }
                vec![parse_expr(args[0].trim())?]
            }
            NumberMethod::ParseInt => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Number.parseInt requires one or two arguments".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Number.parseInt radix argument cannot be empty".into(),
                    ));
                }
                let mut parsed = Vec::with_capacity(args.len());
                parsed.push(parse_expr(args[0].trim())?);
                if args.len() == 2 {
                    parsed.push(parse_expr(args[1].trim())?);
                }
                parsed
            }
        };

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::NumberMethod {
            method,
            args: parsed,
        }));
    }

    let Some(constant) = parse_number_const_name(&member) else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::NumberConst(constant)))
}

pub(super) fn parse_number_const_name(name: &str) -> Option<NumberConst> {
    match name {
        "EPSILON" => Some(NumberConst::Epsilon),
        "MAX_SAFE_INTEGER" => Some(NumberConst::MaxSafeInteger),
        "MAX_VALUE" => Some(NumberConst::MaxValue),
        "MIN_SAFE_INTEGER" => Some(NumberConst::MinSafeInteger),
        "MIN_VALUE" => Some(NumberConst::MinValue),
        "NaN" => Some(NumberConst::NaN),
        "NEGATIVE_INFINITY" => Some(NumberConst::NegativeInfinity),
        "POSITIVE_INFINITY" => Some(NumberConst::PositiveInfinity),
        _ => None,
    }
}

pub(super) fn parse_number_method_name(name: &str) -> Option<NumberMethod> {
    match name {
        "isFinite" => Some(NumberMethod::IsFinite),
        "isInteger" => Some(NumberMethod::IsInteger),
        "isNaN" => Some(NumberMethod::IsNaN),
        "isSafeInteger" => Some(NumberMethod::IsSafeInteger),
        "parseFloat" => Some(NumberMethod::ParseFloat),
        "parseInt" => Some(NumberMethod::ParseInt),
        _ => None,
    }
}

pub(super) fn parse_bigint_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("BigInt") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "BigInt supports zero or one argument".into(),
            ));
        }
        let value = if let Some(arg) = args.first() {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse("BigInt argument cannot be empty".into()));
            }
            Some(Box::new(parse_expr(arg)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::BigIntConstruct {
            value,
            called_with_new,
        }));
    }

    if called_with_new {
        return Ok(None);
    }

    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }

    let Some(method) = parse_bigint_method_name(&member) else {
        return Ok(None);
    };
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };
    if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
        return Err(Error::ScriptParse(format!(
            "BigInt.{member} requires exactly two arguments"
        )));
    }

    let parsed = vec![parse_expr(args[0].trim())?, parse_expr(args[1].trim())?];
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::BigIntMethod {
        method,
        args: parsed,
    }))
}

pub(super) fn parse_bigint_method_name(name: &str) -> Option<BigIntMethod> {
    match name {
        "asIntN" => Some(BigIntMethod::AsIntN),
        "asUintN" => Some(BigIntMethod::AsUintN),
        _ => None,
    }
}

pub(super) fn parse_typed_array_kind_name(name: &str) -> Option<TypedArrayKind> {
    match name {
        "Int8Array" => Some(TypedArrayKind::Int8),
        "Uint8Array" => Some(TypedArrayKind::Uint8),
        "Uint8ClampedArray" => Some(TypedArrayKind::Uint8Clamped),
        "Int16Array" => Some(TypedArrayKind::Int16),
        "Uint16Array" => Some(TypedArrayKind::Uint16),
        "Int32Array" => Some(TypedArrayKind::Int32),
        "Uint32Array" => Some(TypedArrayKind::Uint32),
        "Float16Array" => Some(TypedArrayKind::Float16),
        "Float32Array" => Some(TypedArrayKind::Float32),
        "Float64Array" => Some(TypedArrayKind::Float64),
        "BigInt64Array" => Some(TypedArrayKind::BigInt64),
        "BigUint64Array" => Some(TypedArrayKind::BigUint64),
        _ => None,
    }
}

pub(super) fn parse_typed_array_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    let Some(constructor_name) = cursor.parse_identifier() else {
        return Ok(None);
    };
    let Some(kind) = parse_typed_array_kind_name(&constructor_name) else {
        return Ok(None);
    };
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "{} argument cannot be empty",
                    constructor_name
                )));
            }
            parsed.push(parse_expr(arg)?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::TypedArrayConstruct {
            kind,
            args: parsed,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::TypedArrayConstruct {
                kind,
                args: Vec::new(),
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();

        if member == "BYTES_PER_ELEMENT" {
            if !cursor.eof() {
                return Ok(None);
            }
            return Ok(Some(Expr::TypedArrayStaticBytesPerElement(kind)));
        }

        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        let method = match member.as_str() {
            "from" => TypedArrayStaticMethod::From,
            "of" => TypedArrayStaticMethod::Of,
            _ => return Ok(None),
        };
        if matches!(method, TypedArrayStaticMethod::From)
            && (args.len() != 1 || args[0].trim().is_empty())
        {
            return Err(Error::ScriptParse(format!(
                "{}.from requires exactly one argument",
                constructor_name
            )));
        }
        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "{}.{} argument cannot be empty",
                    constructor_name, member
                )));
            }
            parsed.push(parse_expr(arg)?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::TypedArrayStaticMethod {
            kind,
            method,
            args: parsed,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }

    Ok(Some(Expr::TypedArrayConstructorRef(
        TypedArrayConstructorKind::Concrete(kind),
    )))
}

pub(super) fn parse_promise_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Promise") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Promise supports exactly one executor argument".into(),
            ));
        }
        let executor = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse(
                    "Promise executor argument cannot be empty".into(),
                ));
            }
            Some(Box::new(parse_expr(first)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::PromiseConstruct {
            executor,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::PromiseConstruct {
                executor: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let method = match member.as_str() {
            "resolve" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.resolve supports zero or one argument".into(),
                    ));
                }
                PromiseStaticMethod::Resolve
            }
            "reject" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.reject supports zero or one argument".into(),
                    ));
                }
                PromiseStaticMethod::Reject
            }
            "all" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.all requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::All
            }
            "allSettled" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.allSettled requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::AllSettled
            }
            "any" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.any requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::Any
            }
            "race" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.race requires exactly one argument".into(),
                    ));
                }
                PromiseStaticMethod::Race
            }
            "try" => {
                if args.is_empty() || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.try requires at least one argument".into(),
                    ));
                }
                PromiseStaticMethod::Try
            }
            "withResolvers" => {
                if !args.is_empty() {
                    return Err(Error::ScriptParse(
                        "Promise.withResolvers does not take arguments".into(),
                    ));
                }
                PromiseStaticMethod::WithResolvers
            }
            _ => return Ok(None),
        };

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Promise.{} argument cannot be empty",
                    member
                )));
            }
            parsed.push(parse_expr(arg)?);
        }

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::PromiseStaticMethod {
            method,
            args: parsed,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::PromiseConstructor))
}

pub(super) fn parse_promise_method_expr(src: &str) -> Result<Option<Expr>> {
    let src = src.trim();
    let dots = collect_top_level_char_positions(src, b'.');
    for dot in dots.into_iter().rev() {
        let Some(base_src) = src.get(..dot) else {
            continue;
        };
        let base_src = base_src.trim();
        if base_src.is_empty() {
            continue;
        }

        let Some(tail_src) = src.get(dot + 1..) else {
            continue;
        };
        let tail_src = tail_src.trim();
        let mut cursor = Cursor::new(tail_src);
        let Some(member) = cursor.parse_identifier() else {
            continue;
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            continue;
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        cursor.skip_ws();
        if !cursor.eof() {
            continue;
        }

        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let method = match member.as_str() {
            "then" => {
                if args.len() > 2 {
                    return Err(Error::ScriptParse(
                        "Promise.then supports up to two arguments".into(),
                    ));
                }
                PromiseInstanceMethod::Then
            }
            "catch" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.catch supports at most one argument".into(),
                    ));
                }
                PromiseInstanceMethod::Catch
            }
            "finally" => {
                if args.len() > 1 {
                    return Err(Error::ScriptParse(
                        "Promise.finally supports at most one argument".into(),
                    ));
                }
                PromiseInstanceMethod::Finally
            }
            _ => continue,
        };

        let mut parsed_args = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Promise.{} argument cannot be empty",
                    member
                )));
            }
            parsed_args.push(parse_expr(arg)?);
        }

        return Ok(Some(Expr::PromiseMethod {
            target: Box::new(parse_expr(base_src)?),
            method,
            args: parsed_args,
        }));
    }
    Ok(None)
}

pub(super) fn parse_blob_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Blob") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "Blob supports zero, one, or two arguments".into(),
            ));
        }
        if args.iter().any(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse("Blob argument cannot be empty".into()));
        }

        let parts = args
            .first()
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);
        let options = args
            .get(1)
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::BlobConstruct {
            parts,
            options,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::BlobConstruct {
                parts: None,
                options: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::BlobConstructor))
}

pub(super) fn parse_map_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Map") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Map supports zero or one argument".into(),
            ));
        }
        let iterable = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse("Map argument cannot be empty".into()));
            }
            Some(Box::new(parse_expr(first)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MapConstruct {
            iterable,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::MapConstruct {
                iterable: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if member != "groupBy" {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
            return Err(Error::ScriptParse(
                "Map.groupBy requires exactly two arguments".into(),
            ));
        }
        let parsed = vec![parse_expr(args[0].trim())?, parse_expr(args[1].trim())?];
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MapStaticMethod {
            method: MapStaticMethod::GroupBy,
            args: parsed,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::MapConstructor))
}

pub(super) fn parse_url_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("URL") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "URL supports one or two constructor arguments".into(),
            ));
        }
        if args.first().is_none_or(|arg| arg.trim().is_empty()) {
            return Err(Error::ScriptParse(
                "URL constructor requires a URL argument".into(),
            ));
        }
        if args.len() == 2 && args[1].trim().is_empty() {
            return Err(Error::ScriptParse(
                "URL base argument cannot be empty".into(),
            ));
        }
        let input = args
            .first()
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);
        let base = args
            .get(1)
            .map(|arg| parse_expr(arg.trim()))
            .transpose()?
            .map(Box::new);
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::UrlConstruct {
            input,
            base,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::UrlConstruct {
                input: None,
                base: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        let method = match member.as_str() {
            "canParse" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.canParse requires a URL argument and optional base".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.canParse base argument cannot be empty".into(),
                    ));
                }
                UrlStaticMethod::CanParse
            }
            "parse" => {
                if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.parse requires a URL argument and optional base".into(),
                    ));
                }
                if args.len() == 2 && args[1].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.parse base argument cannot be empty".into(),
                    ));
                }
                UrlStaticMethod::Parse
            }
            "createObjectURL" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.createObjectURL requires exactly one argument".into(),
                    ));
                }
                UrlStaticMethod::CreateObjectUrl
            }
            "revokeObjectURL" => {
                if args.len() != 1 || args[0].trim().is_empty() {
                    return Err(Error::ScriptParse(
                        "URL.revokeObjectURL requires exactly one argument".into(),
                    ));
                }
                UrlStaticMethod::RevokeObjectUrl
            }
            _ => return Ok(None),
        };
        let mut parsed_args = Vec::with_capacity(args.len());
        for arg in args {
            parsed_args.push(parse_expr(arg.trim())?);
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::UrlStaticMethod {
            method,
            args: parsed_args,
        }));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::UrlConstructor))
}

pub(super) fn parse_url_search_params_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("URLSearchParams") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "URLSearchParams supports zero or one argument".into(),
            ));
        }
        let init = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse(
                    "URLSearchParams argument cannot be empty".into(),
                ));
            }
            Some(Box::new(parse_expr(first)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::UrlSearchParamsConstruct {
            init,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::UrlSearchParamsConstruct {
                init: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    Ok(None)
}

pub(super) fn parse_set_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Set") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Set supports zero or one argument".into(),
            ));
        }
        let iterable = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse("Set argument cannot be empty".into()));
            }
            Some(Box::new(parse_expr(first)?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::SetConstruct {
            iterable,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::SetConstruct {
                iterable: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::SetConstructor))
}

pub(super) fn parse_symbol_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("Symbol") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 1 {
            return Err(Error::ScriptParse(
                "Symbol supports zero or one argument".into(),
            ));
        }
        if args.len() == 1 && args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "Symbol description argument cannot be empty".into(),
            ));
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::SymbolConstruct {
            description: if args.is_empty() {
                None
            } else {
                Some(Box::new(parse_expr(args[0].trim())?))
            },
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::SymbolConstruct {
                description: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();

        if cursor.peek() == Some(b'(') {
            let args_src = cursor.read_balanced_block(b'(', b')')?;
            let raw_args = split_top_level_by_char(&args_src, b',');
            let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
                Vec::new()
            } else {
                raw_args
            };
            let method = match member.as_str() {
                "for" => {
                    if args.len() != 1 || args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "Symbol.for requires exactly one argument".into(),
                        ));
                    }
                    SymbolStaticMethod::For
                }
                "keyFor" => {
                    if args.len() != 1 || args[0].trim().is_empty() {
                        return Err(Error::ScriptParse(
                            "Symbol.keyFor requires exactly one argument".into(),
                        ));
                    }
                    SymbolStaticMethod::KeyFor
                }
                _ => return Ok(None),
            };

            let mut parsed = Vec::with_capacity(args.len());
            for arg in args {
                parsed.push(parse_expr(arg.trim())?);
            }
            cursor.skip_ws();
            if !cursor.eof() {
                return Ok(None);
            }
            return Ok(Some(Expr::SymbolStaticMethod {
                method,
                args: parsed,
            }));
        }

        let property = match member.as_str() {
            "asyncDispose" => SymbolStaticProperty::AsyncDispose,
            "asyncIterator" => SymbolStaticProperty::AsyncIterator,
            "dispose" => SymbolStaticProperty::Dispose,
            "hasInstance" => SymbolStaticProperty::HasInstance,
            "isConcatSpreadable" => SymbolStaticProperty::IsConcatSpreadable,
            "iterator" => SymbolStaticProperty::Iterator,
            "match" => SymbolStaticProperty::Match,
            "matchAll" => SymbolStaticProperty::MatchAll,
            "replace" => SymbolStaticProperty::Replace,
            "search" => SymbolStaticProperty::Search,
            "species" => SymbolStaticProperty::Species,
            "split" => SymbolStaticProperty::Split,
            "toPrimitive" => SymbolStaticProperty::ToPrimitive,
            "toStringTag" => SymbolStaticProperty::ToStringTag,
            "unscopables" => SymbolStaticProperty::Unscopables,
            _ => return Ok(None),
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::SymbolStaticProperty(property)));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::SymbolConstructor))
}

pub(super) fn parse_array_buffer_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    let mut called_with_new = false;
    if cursor.consume_ascii("new") {
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        called_with_new = true;
        cursor.skip_ws();
    }

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii("ArrayBuffer") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.peek() == Some(b'(') {
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() > 2 {
            return Err(Error::ScriptParse(
                "ArrayBuffer supports up to two arguments".into(),
            ));
        }
        if args.len() >= 1 && args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "ArrayBuffer byteLength argument cannot be empty".into(),
            ));
        }
        if args.len() == 2 && args[1].trim().is_empty() {
            return Err(Error::ScriptParse(
                "ArrayBuffer options argument cannot be empty".into(),
            ));
        }
        let byte_length = if let Some(first) = args.first() {
            Some(Box::new(parse_expr(first.trim())?))
        } else {
            None
        };
        let options = if args.len() == 2 {
            Some(Box::new(parse_expr(args[1].trim())?))
        } else {
            None
        };
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ArrayBufferConstruct {
            byte_length,
            options,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::ArrayBufferConstruct {
                byte_length: None,
                options: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if cursor.consume_byte(b'.') {
        cursor.skip_ws();
        let Some(member) = cursor.parse_identifier() else {
            return Ok(None);
        };
        cursor.skip_ws();
        if member != "isView" {
            return Ok(None);
        }
        if cursor.peek() != Some(b'(') {
            return Ok(None);
        }
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };
        if args.len() != 1 || args[0].trim().is_empty() {
            return Err(Error::ScriptParse(
                "ArrayBuffer.isView requires exactly one argument".into(),
            ));
        }
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ArrayBufferIsView(Box::new(parse_expr(
            args[0].trim(),
        )?))));
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::ArrayBufferConstructor))
}

