use super::*;

pub(crate) fn validate_math_arity(method: MathMethod, count: usize) -> Result<()> {
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

pub(crate) fn parse_string_expr(src: &str) -> Result<Option<Expr>> {
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
            parsed.push(parse_call_arg_expr(arg)?);
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

pub(crate) fn parse_number_expr(src: &str) -> Result<Option<Expr>> {
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
            return Ok(Some(Expr::NumberConstruct {
                value: None,
                called_with_new: true,
            }));
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
        return Ok(Some(Expr::NumberConstruct {
            value,
            called_with_new: has_new,
        }));
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

pub(crate) fn parse_boolean_expr(src: &str) -> Result<Option<Expr>> {
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

    if !cursor.consume_ascii("Boolean") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if called_with_new && cursor.peek() != Some(b'(') {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::BooleanConstruct {
                value: None,
                called_with_new: true,
            }));
        }
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
    if args.len() > 1 {
        return Err(Error::ScriptParse(
            "Boolean supports zero or one argument".into(),
        ));
    }
    let value = if let Some(arg) = args.first() {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(
                "Boolean argument cannot be empty".into(),
            ));
        }
        Some(Box::new(parse_expr(arg)?))
    } else {
        None
    };
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::BooleanConstruct {
        value,
        called_with_new,
    }))
}

pub(crate) fn parse_number_const_name(name: &str) -> Option<NumberConst> {
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

pub(crate) fn parse_number_method_name(name: &str) -> Option<NumberMethod> {
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

pub(crate) fn parse_bigint_expr(src: &str) -> Result<Option<Expr>> {
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

pub(crate) fn parse_bigint_method_name(name: &str) -> Option<BigIntMethod> {
    match name {
        "asIntN" => Some(BigIntMethod::AsIntN),
        "asUintN" => Some(BigIntMethod::AsUintN),
        _ => None,
    }
}

pub(crate) fn parse_typed_array_kind_name(name: &str) -> Option<TypedArrayKind> {
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

pub(crate) fn parse_typed_array_expr(src: &str) -> Result<Option<Expr>> {
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
        if matches!(method, TypedArrayStaticMethod::From) {
            if args.is_empty() || args.len() > 3 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(format!(
                    "{}.from requires a source argument and supports at most mapFn/thisArg",
                    constructor_name
                )));
            }
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
