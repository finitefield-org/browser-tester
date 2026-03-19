use super::*;

pub(crate) fn parse_array_constructor_expr(src: &str) -> Result<Option<Expr>> {
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

    if !cursor.consume_ascii("Array") {
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

        let mut parsed = Vec::with_capacity(args.len());
        for arg in args {
            let arg = arg.trim();
            if arg.is_empty() {
                return Err(Error::ScriptParse(
                    "Array constructor arguments cannot be empty".into(),
                ));
            }
            parsed.push(parse_expr(arg)?);
        }

        cursor.skip_ws();
        if !cursor.eof() {
            return Ok(None);
        }
        return Ok(Some(Expr::ArrayConstruct {
            args: parsed,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::ArrayConstruct {
                args: Vec::new(),
                called_with_new: true,
            }));
        }
    }

    Ok(None)
}

pub(crate) fn parse_map_expr(src: &str) -> Result<Option<Expr>> {
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

pub(crate) fn parse_weak_map_expr(src: &str) -> Result<Option<Expr>> {
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

    if !cursor.consume_ascii("WeakMap") {
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
                "WeakMap supports zero or one argument".into(),
            ));
        }
        let iterable = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse(
                    "WeakMap argument cannot be empty".into(),
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
        return Ok(Some(Expr::WeakMapConstruct {
            iterable,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::WeakMapConstruct {
                iterable: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::WeakMapConstructor))
}

pub(crate) fn parse_set_expr(src: &str) -> Result<Option<Expr>> {
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

pub(crate) fn parse_weak_set_expr(src: &str) -> Result<Option<Expr>> {
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

    if !cursor.consume_ascii("WeakSet") {
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
                "WeakSet supports zero or one argument".into(),
            ));
        }
        let iterable = if let Some(first) = args.first() {
            let first = first.trim();
            if first.is_empty() {
                return Err(Error::ScriptParse(
                    "WeakSet argument cannot be empty".into(),
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
        return Ok(Some(Expr::WeakSetConstruct {
            iterable,
            called_with_new,
        }));
    }

    if called_with_new {
        cursor.skip_ws();
        if cursor.eof() {
            return Ok(Some(Expr::WeakSetConstruct {
                iterable: None,
                called_with_new: true,
            }));
        }
        return Ok(None);
    }

    if !cursor.eof() {
        return Ok(None);
    }
    Ok(Some(Expr::WeakSetConstructor))
}

pub(crate) fn parse_symbol_expr(src: &str) -> Result<Option<Expr>> {
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
