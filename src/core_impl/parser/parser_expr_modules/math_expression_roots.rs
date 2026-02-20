pub(super) fn parse_math_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }
    if !cursor.consume_ascii("Math") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.consume_byte(b'[') {
        cursor.skip_ws();
        if !cursor.consume_ascii("Symbol") {
            return Ok(None);
        }
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
        if !cursor.consume_ascii("toStringTag") {
            return Ok(None);
        }
        if let Some(next) = cursor.peek() {
            if is_ident_char(next) {
                return Ok(None);
            }
        }
        cursor.skip_ws();
        cursor.expect_byte(b']')?;
        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MathConst(MathConst::ToStringTag)));
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
        let args_src = cursor.read_balanced_block(b'(', b')')?;
        let raw_args = split_top_level_by_char(&args_src, b',');
        let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
            Vec::new()
        } else {
            raw_args
        };

        let Some(method) = parse_math_method_name(&member) else {
            return Ok(None);
        };
        validate_math_arity(method, args.len())?;

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(format!(
                    "Math.{} argument cannot be empty",
                    member
                )));
            }
            parsed.push(parse_expr(arg)?);
        }

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::MathMethod {
            method,
            args: parsed,
        }));
    }

    let Some(constant) = parse_math_const_name(&member) else {
        return Ok(None);
    };

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::MathConst(constant)))
}

pub(super) fn parse_math_const_name(name: &str) -> Option<MathConst> {
    match name {
        "E" => Some(MathConst::E),
        "LN10" => Some(MathConst::Ln10),
        "LN2" => Some(MathConst::Ln2),
        "LOG10E" => Some(MathConst::Log10E),
        "LOG2E" => Some(MathConst::Log2E),
        "PI" => Some(MathConst::Pi),
        "SQRT1_2" => Some(MathConst::Sqrt1_2),
        "SQRT2" => Some(MathConst::Sqrt2),
        _ => None,
    }
}

pub(super) fn parse_math_method_name(name: &str) -> Option<MathMethod> {
    match name {
        "abs" => Some(MathMethod::Abs),
        "acos" => Some(MathMethod::Acos),
        "acosh" => Some(MathMethod::Acosh),
        "asin" => Some(MathMethod::Asin),
        "asinh" => Some(MathMethod::Asinh),
        "atan" => Some(MathMethod::Atan),
        "atan2" => Some(MathMethod::Atan2),
        "atanh" => Some(MathMethod::Atanh),
        "cbrt" => Some(MathMethod::Cbrt),
        "ceil" => Some(MathMethod::Ceil),
        "clz32" => Some(MathMethod::Clz32),
        "cos" => Some(MathMethod::Cos),
        "cosh" => Some(MathMethod::Cosh),
        "exp" => Some(MathMethod::Exp),
        "expm1" => Some(MathMethod::Expm1),
        "floor" => Some(MathMethod::Floor),
        "f16round" => Some(MathMethod::F16Round),
        "fround" => Some(MathMethod::FRound),
        "hypot" => Some(MathMethod::Hypot),
        "imul" => Some(MathMethod::Imul),
        "log" => Some(MathMethod::Log),
        "log10" => Some(MathMethod::Log10),
        "log1p" => Some(MathMethod::Log1p),
        "log2" => Some(MathMethod::Log2),
        "max" => Some(MathMethod::Max),
        "min" => Some(MathMethod::Min),
        "pow" => Some(MathMethod::Pow),
        "random" => Some(MathMethod::Random),
        "round" => Some(MathMethod::Round),
        "sign" => Some(MathMethod::Sign),
        "sin" => Some(MathMethod::Sin),
        "sinh" => Some(MathMethod::Sinh),
        "sqrt" => Some(MathMethod::Sqrt),
        "sumPrecise" => Some(MathMethod::SumPrecise),
        "tan" => Some(MathMethod::Tan),
        "tanh" => Some(MathMethod::Tanh),
        "trunc" => Some(MathMethod::Trunc),
        _ => None,
    }
}

