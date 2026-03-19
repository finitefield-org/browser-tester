use super::*;

impl Harness {
    pub(crate) fn collect_capture_names_from_data_collection_expr(
        expr: &Expr,
        out: &mut HashSet<String>,
    ) -> bool {
        match expr {
            Expr::ArrayLiteral(args) | Expr::ArrayConstruct { args, .. } => {
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::StringConstruct { value, .. }
            | Expr::BooleanConstruct { value, .. }
            | Expr::NumberConstruct { value, .. }
            | Expr::BigIntConstruct { value, .. }
            | Expr::ObjectConstruct { value } => {
                if let Some(value) = value.as_ref() {
                    Self::collect_capture_names_from_expr(value, out);
                }
            }
            Expr::NumberInstanceMethod { value, args, .. }
            | Expr::BigIntInstanceMethod { value, args, .. } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::BlobConstruct { parts, options, .. } => {
                if let Some(parts) = parts.as_ref() {
                    Self::collect_capture_names_from_expr(parts, out);
                }
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::UrlConstruct { input, base, .. } => {
                if let Some(input) = input.as_ref() {
                    Self::collect_capture_names_from_expr(input, out);
                }
                if let Some(base) = base.as_ref() {
                    Self::collect_capture_names_from_expr(base, out);
                }
            }
            Expr::ArrayBufferConstruct {
                byte_length,
                options,
                ..
            } => {
                if let Some(byte_length) = byte_length.as_ref() {
                    Self::collect_capture_names_from_expr(byte_length, out);
                }
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::ArrayBufferResize {
                target,
                new_byte_length,
            } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_expr(new_byte_length, out);
            }
            Expr::ArrayBufferSlice { target, start, end } => {
                Self::collect_capture_name(target, out);
                if let Some(start) = start.as_ref() {
                    Self::collect_capture_names_from_expr(start, out);
                }
                if let Some(end) = end.as_ref() {
                    Self::collect_capture_names_from_expr(end, out);
                }
            }
            Expr::ArrayBufferTransfer { target, .. } => {
                Self::collect_capture_name(target, out);
            }
            Expr::TypedArrayConstruct { args, .. } => {
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::TypedArrayConstructWithCallee { callee, args, .. } => {
                Self::collect_capture_names_from_expr(callee, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::PromiseConstruct { executor, .. }
            | Expr::MapConstruct {
                iterable: executor, ..
            }
            | Expr::WeakMapConstruct {
                iterable: executor, ..
            }
            | Expr::UrlSearchParamsConstruct { init: executor, .. }
            | Expr::SetConstruct {
                iterable: executor, ..
            }
            | Expr::WeakSetConstruct {
                iterable: executor, ..
            }
            | Expr::SymbolConstruct {
                description: executor,
                ..
            } => {
                if let Some(executor) = executor.as_ref() {
                    Self::collect_capture_names_from_expr(executor, out);
                }
            }
            Expr::PromiseMethod { target, args, .. } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::TypedArrayMethod { target, args, .. } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::ParseInt { value, radix } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(radix) = radix.as_ref() {
                    Self::collect_capture_names_from_expr(radix, out);
                }
            }
            Expr::JsonStringify {
                value,
                replacer,
                space,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(replacer) = replacer.as_ref() {
                    Self::collect_capture_names_from_expr(replacer, out);
                }
                if let Some(space) = space.as_ref() {
                    Self::collect_capture_names_from_expr(space, out);
                }
            }
            Expr::ObjectLiteral(entries) => {
                for entry in entries {
                    Self::collect_capture_names_from_object_literal_entry(entry, out);
                }
            }
            Expr::ObjectGetOwnPropertyDescriptor { object, key }
            | Expr::ObjectHasOwn { object, key } => {
                Self::collect_capture_names_from_expr(object, out);
                Self::collect_capture_names_from_expr(key, out);
            }
            Expr::ObjectDefineProperty {
                object,
                key,
                descriptor,
            } => {
                Self::collect_capture_names_from_expr(object, out);
                Self::collect_capture_names_from_expr(key, out);
                Self::collect_capture_names_from_expr(descriptor, out);
            }
            Expr::ReflectSet {
                target,
                key,
                value,
                receiver,
            } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_expr(key, out);
                Self::collect_capture_names_from_expr(value, out);
                if let Some(receiver) = receiver.as_ref() {
                    Self::collect_capture_names_from_expr(receiver, out);
                }
            }
            Expr::ArrayFrom { source, map_fn } => {
                Self::collect_capture_names_from_expr(source, out);
                if let Some(map_fn) = map_fn.as_ref() {
                    Self::collect_capture_names_from_expr(map_fn, out);
                }
            }
            Expr::ArrayIndex { target, index } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_expr(index, out);
            }
            Expr::ArrayPush { target, args } | Expr::ArrayUnshift { target, args } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::ArrayMap { target, callback }
            | Expr::ArrayFilter { target, callback }
            | Expr::ArrayForEach { target, callback }
            | Expr::ArrayFind { target, callback }
            | Expr::ArrayFindIndex { target, callback }
            | Expr::ArraySome { target, callback }
            | Expr::ArrayEvery { target, callback } => {
                Self::collect_capture_name(target, out);
                Self::collect_nested_handler_capture_names(callback, out);
            }
            Expr::ArrayReduce {
                target,
                callback,
                initial,
            } => {
                Self::collect_capture_name(target, out);
                Self::collect_nested_handler_capture_names(callback, out);
                if let Some(initial) = initial.as_ref() {
                    Self::collect_capture_names_from_expr(initial, out);
                }
            }
            Expr::ArraySlice { target, start, end } => {
                Self::collect_capture_name(target, out);
                if let Some(start) = start.as_ref() {
                    Self::collect_capture_names_from_expr(start, out);
                }
                if let Some(end) = end.as_ref() {
                    Self::collect_capture_names_from_expr(end, out);
                }
            }
            Expr::ArraySplice {
                target,
                start,
                delete_count,
                items,
            } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_expr(start, out);
                if let Some(delete_count) = delete_count.as_ref() {
                    Self::collect_capture_names_from_expr(delete_count, out);
                }
                Self::collect_capture_names_from_exprs(items, out);
            }
            Expr::ArrayJoin { target, separator } => {
                Self::collect_capture_name(target, out);
                if let Some(separator) = separator.as_ref() {
                    Self::collect_capture_names_from_expr(separator, out);
                }
            }
            Expr::ArraySort { target, comparator } => {
                Self::collect_capture_name(target, out);
                if let Some(comparator) = comparator.as_ref() {
                    Self::collect_capture_names_from_expr(comparator, out);
                }
            }
            Expr::StringTrim { value, .. } => {
                Self::collect_capture_names_from_expr(value, out);
            }
            Expr::StringIncludes {
                value,
                search,
                position,
            }
            | Expr::StringStartsWith {
                value,
                search,
                position,
            }
            | Expr::StringIndexOf {
                value,
                search,
                position,
            }
            | Expr::StringLastIndexOf {
                value,
                search,
                position,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(search, out);
                if let Some(position) = position.as_ref() {
                    Self::collect_capture_names_from_expr(position, out);
                }
            }
            Expr::StringEndsWith {
                value,
                search,
                length,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(search, out);
                if let Some(length) = length.as_ref() {
                    Self::collect_capture_names_from_expr(length, out);
                }
            }
            Expr::StringSlice { value, start, end }
            | Expr::StringSubstring { value, start, end } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(start) = start.as_ref() {
                    Self::collect_capture_names_from_expr(start, out);
                }
                if let Some(end) = end.as_ref() {
                    Self::collect_capture_names_from_expr(end, out);
                }
            }
            Expr::StringMatch { value, pattern } | Expr::StringSearch { value, pattern } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(pattern, out);
            }
            Expr::StringSplit {
                value,
                separator,
                limit,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(separator) = separator.as_ref() {
                    Self::collect_capture_names_from_expr(separator, out);
                }
                if let Some(limit) = limit.as_ref() {
                    Self::collect_capture_names_from_expr(limit, out);
                }
            }
            Expr::StringReplace { value, from, to }
            | Expr::StringReplaceAll { value, from, to } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(from, out);
                Self::collect_capture_names_from_expr(to, out);
            }
            Expr::StringCharAt { value, index }
            | Expr::StringCharCodeAt { value, index }
            | Expr::StringCodePointAt { value, index }
            | Expr::StringAt { value, index } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(index) = index.as_ref() {
                    Self::collect_capture_names_from_expr(index, out);
                }
            }
            Expr::StringConcat { value, args } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::StringRepeat { value, count } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(count, out);
            }
            Expr::StringPadStart {
                value,
                target_length,
                pad,
            }
            | Expr::StringPadEnd {
                value,
                target_length,
                pad,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(target_length, out);
                if let Some(pad) = pad.as_ref() {
                    Self::collect_capture_names_from_expr(pad, out);
                }
            }
            Expr::StringLocaleCompare {
                value,
                compare,
                locales,
                options,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(compare, out);
                if let Some(locales) = locales.as_ref() {
                    Self::collect_capture_names_from_expr(locales, out);
                }
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            _ => return false,
        }
        true
    }
}
