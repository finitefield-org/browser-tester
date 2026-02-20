use super::*;

pub(crate) fn parse_new_error_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if !cursor.consume_ascii("Error") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    if cursor.eof() {
        return Ok(Some(Expr::String("Error".to_string())));
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
    if args.len() > 2 {
        return Err(Error::ScriptParse(
            "Error constructor supports up to two arguments".into(),
        ));
    }
    if args.first().is_some_and(|arg| arg.trim().is_empty()) {
        return Err(Error::ScriptParse(
            "Error message argument cannot be empty".into(),
        ));
    }
    if args.len() == 2 && args[1].trim().is_empty() {
        return Err(Error::ScriptParse(
            "Error options argument cannot be empty".into(),
        ));
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    if let Some(message) = args.first() {
        return Ok(Some(parse_expr(message.trim())?));
    }
    Ok(Some(Expr::String("Error".to_string())))
}

pub(crate) fn parse_new_callee_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    if !cursor.consume_ascii("new") {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();
    if cursor.peek() != Some(b'(') {
        return Ok(None);
    }
    let callee_src = cursor.read_balanced_block(b'(', b')')?;
    cursor.skip_ws();
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
            return Err(Error::ScriptParse(
                "constructor argument cannot be empty".into(),
            ));
        }
        parsed.push(parse_expr(arg)?);
    }
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    let callee = parse_expr(callee_src.trim())?;
    Ok(Some(Expr::TypedArrayConstructWithCallee {
        callee: Box::new(callee),
        args: parsed,
        called_with_new: true,
    }))
}

pub(crate) fn parse_array_buffer_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    let property_expr = match member.as_str() {
        "detached" => Some(Expr::ArrayBufferDetached(target.clone())),
        "maxByteLength" => Some(Expr::ArrayBufferMaxByteLength(target.clone())),
        "resizable" => Some(Expr::ArrayBufferResizable(target.clone())),
        _ => None,
    };
    if let Some(expr) = property_expr {
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(expr));
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
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    match member.as_str() {
        "resize" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.resize requires exactly one argument".into(),
                ));
            }
            Ok(Some(Expr::ArrayBufferResize {
                target,
                new_byte_length: Box::new(parse_expr(args[0].trim())?),
            }))
        }
        "slice" => {
            if args.len() > 2 {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.slice supports up to two arguments".into(),
                ));
            }
            if args.len() >= 1 && args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.slice start cannot be empty".into(),
                ));
            }
            if args.len() == 2 && args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.slice end cannot be empty".into(),
                ));
            }
            let start = if let Some(first) = args.first() {
                Some(Box::new(parse_expr(first.trim())?))
            } else {
                None
            };
            let end = if args.len() == 2 {
                Some(Box::new(parse_expr(args[1].trim())?))
            } else {
                None
            };
            Ok(Some(Expr::ArrayBufferSlice { target, start, end }))
        }
        "transfer" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.transfer does not take arguments".into(),
                ));
            }
            Ok(Some(Expr::ArrayBufferTransfer {
                target,
                to_fixed_length: false,
            }))
        }
        "transferToFixedLength" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "ArrayBuffer.transferToFixedLength does not take arguments".into(),
                ));
            }
            Ok(Some(Expr::ArrayBufferTransfer {
                target,
                to_fixed_length: true,
            }))
        }
        _ => Ok(None),
    }
}

pub(crate) fn parse_typed_array_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();

    let property_expr = match member.as_str() {
        "byteLength" => Some(Expr::TypedArrayByteLength(target.clone())),
        "byteOffset" => Some(Expr::TypedArrayByteOffset(target.clone())),
        "buffer" => Some(Expr::TypedArrayBuffer(target.clone())),
        "BYTES_PER_ELEMENT" => Some(Expr::TypedArrayBytesPerElement(target.clone())),
        _ => None,
    };
    if let Some(expr) = property_expr {
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(expr));
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
        "at" => TypedArrayInstanceMethod::At,
        "copyWithin" => TypedArrayInstanceMethod::CopyWithin,
        "entries" => TypedArrayInstanceMethod::Entries,
        "fill" => TypedArrayInstanceMethod::Fill,
        "findIndex" => TypedArrayInstanceMethod::FindIndex,
        "findLast" => TypedArrayInstanceMethod::FindLast,
        "findLastIndex" => TypedArrayInstanceMethod::FindLastIndex,
        "indexOf" => TypedArrayInstanceMethod::IndexOf,
        "keys" => TypedArrayInstanceMethod::Keys,
        "lastIndexOf" => TypedArrayInstanceMethod::LastIndexOf,
        "reduceRight" => TypedArrayInstanceMethod::ReduceRight,
        "reverse" => TypedArrayInstanceMethod::Reverse,
        "set" => TypedArrayInstanceMethod::Set,
        "sort" => TypedArrayInstanceMethod::Sort,
        "subarray" => TypedArrayInstanceMethod::Subarray,
        "toReversed" => TypedArrayInstanceMethod::ToReversed,
        "toSorted" => TypedArrayInstanceMethod::ToSorted,
        "values" => TypedArrayInstanceMethod::Values,
        "with" => TypedArrayInstanceMethod::With,
        _ => return Ok(None),
    };

    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(format!(
                "{} argument cannot be empty",
                member
            )));
        }
        parsed.push(parse_expr(arg)?);
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::TypedArrayMethod {
        target,
        method,
        args: parsed,
    }))
}

pub(crate) fn parse_url_search_params_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
    if !cursor.consume_byte(b'.') {
        return Ok(None);
    }
    cursor.skip_ws();
    let Some(member) = cursor.parse_identifier() else {
        return Ok(None);
    };
    if !matches!(member.as_str(), "append" | "getAll" | "has" | "delete") {
        return Ok(None);
    }
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
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }

    let method = match member.as_str() {
        "append" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "URLSearchParams.append requires exactly two arguments".into(),
                ));
            }
            UrlSearchParamsInstanceMethod::Append
        }
        "getAll" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "URLSearchParams.getAll requires exactly one argument".into(),
                ));
            }
            UrlSearchParamsInstanceMethod::GetAll
        }
        "has" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Ok(None);
            }
            UrlSearchParamsInstanceMethod::Has
        }
        "delete" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Ok(None);
            }
            UrlSearchParamsInstanceMethod::Delete
        }
        _ => unreachable!(),
    };

    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        parsed.push(parse_expr(arg.trim())?);
    }

    Ok(Some(Expr::UrlSearchParamsMethod {
        target,
        method,
        args: parsed,
    }))
}

pub(crate) fn parse_map_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
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
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    let method = match member.as_str() {
        "get" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.get requires exactly one argument".into(),
                ));
            }
            MapInstanceMethod::Get
        }
        "has" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.has requires exactly one argument".into(),
                ));
            }
            MapInstanceMethod::Has
        }
        "delete" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.delete requires exactly one argument".into(),
                ));
            }
            MapInstanceMethod::Delete
        }
        "clear" => {
            if !args.is_empty() {
                return Err(Error::ScriptParse(
                    "Map.clear does not take arguments".into(),
                ));
            }
            MapInstanceMethod::Clear
        }
        "forEach" => {
            if args.is_empty() || args.len() > 2 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.forEach requires a callback and optional thisArg".into(),
                ));
            }
            if args.len() == 2 && args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.forEach thisArg cannot be empty".into(),
                ));
            }
            MapInstanceMethod::ForEach
        }
        "getOrInsert" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.getOrInsert requires exactly two arguments".into(),
                ));
            }
            MapInstanceMethod::GetOrInsert
        }
        "getOrInsertComputed" => {
            if args.len() != 2 || args[0].trim().is_empty() || args[1].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Map.getOrInsertComputed requires exactly two arguments".into(),
                ));
            }
            MapInstanceMethod::GetOrInsertComputed
        }
        _ => return Ok(None),
    };

    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(format!(
                "Map.{} argument cannot be empty",
                member
            )));
        }
        parsed.push(parse_expr(arg)?);
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::MapMethod {
        target,
        method,
        args: parsed,
    }))
}

pub(crate) fn parse_set_access_expr(src: &str) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();
    let Some(target) = cursor.parse_identifier() else {
        return Ok(None);
    };
    cursor.skip_ws();
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
    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let raw_args = split_top_level_by_char(&args_src, b',');
    let args = if raw_args.len() == 1 && raw_args[0].trim().is_empty() {
        Vec::new()
    } else {
        raw_args
    };

    let method = match member.as_str() {
        "add" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.add requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::Add
        }
        "union" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.union requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::Union
        }
        "intersection" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.intersection requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::Intersection
        }
        "difference" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.difference requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::Difference
        }
        "symmetricDifference" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.symmetricDifference requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::SymmetricDifference
        }
        "isDisjointFrom" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.isDisjointFrom requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::IsDisjointFrom
        }
        "isSubsetOf" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.isSubsetOf requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::IsSubsetOf
        }
        "isSupersetOf" => {
            if args.len() != 1 || args[0].trim().is_empty() {
                return Err(Error::ScriptParse(
                    "Set.isSupersetOf requires exactly one argument".into(),
                ));
            }
            SetInstanceMethod::IsSupersetOf
        }
        _ => return Ok(None),
    };

    let mut parsed = Vec::with_capacity(args.len());
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            return Err(Error::ScriptParse(format!(
                "Set.{} argument cannot be empty",
                member
            )));
        }
        parsed.push(parse_expr(arg)?);
    }

    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::SetMethod {
        target,
        method,
        args: parsed,
    }))
}

pub(crate) fn parse_is_nan_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "isNaN", "isNaN requires exactly one argument")
}

pub(crate) fn parse_encode_uri_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "encodeURI", "encodeURI requires exactly one argument")
}

pub(crate) fn parse_encode_uri_component_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "encodeURIComponent",
        "encodeURIComponent requires exactly one argument",
    )
}

pub(crate) fn parse_decode_uri_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "decodeURI", "decodeURI requires exactly one argument")
}

pub(crate) fn parse_decode_uri_component_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(
        src,
        "decodeURIComponent",
        "decodeURIComponent requires exactly one argument",
    )
}

pub(crate) fn parse_escape_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "escape", "escape requires exactly one argument")
}

pub(crate) fn parse_unescape_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "unescape", "unescape requires exactly one argument")
}

pub(crate) fn parse_is_finite_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "isFinite", "isFinite requires exactly one argument")
}

pub(crate) fn parse_atob_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "atob", "atob requires exactly one argument")
}

pub(crate) fn parse_btoa_expr(src: &str) -> Result<Option<Expr>> {
    parse_global_single_arg_expr(src, "btoa", "btoa requires exactly one argument")
}

pub(crate) fn parse_global_single_arg_expr(
    src: &str,
    function_name: &str,
    arg_error: &str,
) -> Result<Option<Expr>> {
    let mut cursor = Cursor::new(src);
    cursor.skip_ws();

    if cursor.consume_ascii("window") {
        cursor.skip_ws();
        if !cursor.consume_byte(b'.') {
            return Ok(None);
        }
        cursor.skip_ws();
    }

    if !cursor.consume_ascii(function_name) {
        return Ok(None);
    }
    if let Some(next) = cursor.peek() {
        if is_ident_char(next) {
            return Ok(None);
        }
    }
    cursor.skip_ws();

    let args_src = cursor.read_balanced_block(b'(', b')')?;
    let args = split_top_level_by_char(&args_src, b',');
    if args.len() != 1 || args[0].trim().is_empty() {
        return Err(Error::ScriptParse(arg_error.into()));
    }

    let value = parse_expr(args[0].trim())?;
    cursor.skip_ws();
    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(value))
}
