use super::*;

impl Harness {
    pub(crate) fn collect_capture_names_from_exprs(exprs: &[Expr], out: &mut HashSet<String>) {
        for expr in exprs {
            Self::collect_capture_names_from_expr(expr, out);
        }
    }

    pub(crate) fn collect_capture_names_from_expr(expr: &Expr, out: &mut HashSet<String>) {
        match expr {
            Expr::String(_)
            | Expr::Bool(_)
            | Expr::Null
            | Expr::Undefined
            | Expr::Number(_)
            | Expr::Float(_)
            | Expr::BigInt(_)
            | Expr::DateNow
            | Expr::PerformanceNow
            | Expr::RegexLiteral { .. }
            | Expr::RegExpConstructor
            | Expr::MathConst(_)
            | Expr::StringConstructor
            | Expr::NumberConst(_)
            | Expr::BlobConstructor
            | Expr::UrlConstructor
            | Expr::ArrayBufferConstructor
            | Expr::TypedArrayConstructorRef(_)
            | Expr::PromiseConstructor
            | Expr::MapConstructor
            | Expr::WeakMapConstructor
            | Expr::UrlSearchParamsConstructor
            | Expr::SetConstructor
            | Expr::WeakSetConstructor
            | Expr::SymbolConstructor
            | Expr::SymbolStaticProperty(_)
            | Expr::TypedArrayStaticBytesPerElement(_)
            | Expr::ImportMeta
            | Expr::NewTarget
            | Expr::CreateElement(_)
            | Expr::CreateTextNode(_)
            | Expr::DocumentHasFocus => {}
            Expr::DateNew { args }
            | Expr::DateUtc { args }
            | Expr::IntlStaticMethod { args, .. }
            | Expr::IntlConstruct { args }
            | Expr::RegExpStaticMethod { args, .. }
            | Expr::MathMethod { args, .. }
            | Expr::StringStaticMethod { args, .. }
            | Expr::NumberMethod { args, .. }
            | Expr::BigIntMethod { args, .. }
            | Expr::UrlStaticMethod { args, .. }
            | Expr::PromiseStaticMethod { args, .. }
            | Expr::MapStaticMethod { args, .. }
            | Expr::SymbolStaticMethod { args, .. }
            | Expr::TypedArrayStaticMethod { args, .. }
            | Expr::FunctionConstructor { args }
            | Expr::HistoryMethodCall { args, .. }
            | Expr::ClipboardMethodCall { args, .. }
            | Expr::ArrayLiteral(args)
            | Expr::ArrayConstruct { args, .. }
            | Expr::Comma(args)
            | Expr::Add(args) => {
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::DateParse(inner)
            | Expr::ArrayBufferIsView(inner)
            | Expr::EncodeUri(inner)
            | Expr::EncodeUriComponent(inner)
            | Expr::DecodeUri(inner)
            | Expr::DecodeUriComponent(inner)
            | Expr::Escape(inner)
            | Expr::Unescape(inner)
            | Expr::IsNaN(inner)
            | Expr::IsFinite(inner)
            | Expr::Atob(inner)
            | Expr::Btoa(inner)
            | Expr::ParseFloat(inner)
            | Expr::JsonParse(inner)
            | Expr::ObjectGetOwnPropertyNames(inner)
            | Expr::ObjectGetOwnPropertySymbols(inner)
            | Expr::ObjectKeys(inner)
            | Expr::ObjectValues(inner)
            | Expr::ObjectEntries(inner)
            | Expr::ObjectGetPrototypeOf(inner)
            | Expr::ObjectFreeze(inner)
            | Expr::ReflectOwnKeys(inner)
            | Expr::ArrayIsArray(inner)
            | Expr::StringToUpperCase(inner)
            | Expr::StringToLowerCase(inner)
            | Expr::StringIsWellFormed(inner)
            | Expr::StringToWellFormed(inner)
            | Expr::StringValueOf(inner)
            | Expr::StringToString(inner)
            | Expr::MatchMedia(inner)
            | Expr::Alert(inner)
            | Expr::Confirm(inner)
            | Expr::Neg(inner)
            | Expr::Pos(inner)
            | Expr::BitNot(inner)
            | Expr::Not(inner)
            | Expr::Void(inner)
            | Expr::Delete(inner)
            | Expr::TypeOf(inner)
            | Expr::Await(inner)
            | Expr::Yield(inner)
            | Expr::YieldStar(inner)
            | Expr::Spread(inner) => {
                Self::collect_capture_names_from_expr(inner, out);
            }
            Expr::DateGetTime(target)
            | Expr::DateToIsoString(target)
            | Expr::DateGetUTCFullYear(target)
            | Expr::DateGetFullYear(target)
            | Expr::DateGetMonth(target)
            | Expr::DateGetDate(target)
            | Expr::DateGetHours(target)
            | Expr::DateGetMinutes(target)
            | Expr::DateGetSeconds(target)
            | Expr::ArrayBufferDetached(target)
            | Expr::ArrayBufferMaxByteLength(target)
            | Expr::ArrayBufferResizable(target)
            | Expr::TypedArrayByteLength(target)
            | Expr::TypedArrayByteOffset(target)
            | Expr::TypedArrayBuffer(target)
            | Expr::TypedArrayBytesPerElement(target)
            | Expr::ArrayLength(target)
            | Expr::ArrayPop(target)
            | Expr::ArrayShift(target)
            | Expr::ObjectGet { target, .. }
            | Expr::ObjectPathGet { target, .. }
            | Expr::ObjectHasOwnProperty { target, .. }
            | Expr::EventProp {
                event_var: target, ..
            }
            | Expr::Var(target) => {
                Self::collect_capture_name(target, out);
            }
            Expr::FunctionCall { target, args } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::MapMethod { target, args, .. }
            | Expr::UrlSearchParamsMethod { target, args, .. }
            | Expr::SetMethod { target, args, .. } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::DateSetTime { target, value } => {
                Self::collect_capture_name(target, out);
                Self::collect_capture_names_from_expr(value, out);
            }
            Expr::IntlFormatterConstruct {
                locales, options, ..
            } => {
                if let Some(locales) = locales.as_ref() {
                    Self::collect_capture_names_from_expr(locales, out);
                }
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::IntlFormat { formatter, value }
            | Expr::IntlDateTimeFormatToParts { formatter, value } => {
                Self::collect_capture_names_from_expr(formatter, out);
                if let Some(value) = value.as_ref() {
                    Self::collect_capture_names_from_expr(value, out);
                }
            }
            Expr::IntlFormatGetter { formatter }
            | Expr::IntlCollatorCompareGetter {
                collator: formatter,
            }
            | Expr::IntlDateTimeResolvedOptions { formatter }
            | Expr::RegexToString { regex: formatter } => {
                Self::collect_capture_names_from_expr(formatter, out);
            }
            Expr::IntlCollatorCompare {
                collator,
                left,
                right,
            } => {
                Self::collect_capture_names_from_expr(collator, out);
                Self::collect_capture_names_from_expr(left, out);
                Self::collect_capture_names_from_expr(right, out);
            }
            Expr::IntlDateTimeFormatRange {
                formatter,
                start,
                end,
            }
            | Expr::IntlDateTimeFormatRangeToParts {
                formatter,
                start,
                end,
            } => {
                Self::collect_capture_names_from_expr(formatter, out);
                Self::collect_capture_names_from_expr(start, out);
                Self::collect_capture_names_from_expr(end, out);
            }
            Expr::IntlDisplayNamesOf {
                display_names,
                code,
            } => {
                Self::collect_capture_names_from_expr(display_names, out);
                Self::collect_capture_names_from_expr(code, out);
            }
            Expr::IntlPluralRulesSelect {
                plural_rules,
                value,
            } => {
                Self::collect_capture_names_from_expr(plural_rules, out);
                Self::collect_capture_names_from_expr(value, out);
            }
            Expr::IntlPluralRulesSelectRange {
                plural_rules,
                start,
                end,
            } => {
                Self::collect_capture_names_from_expr(plural_rules, out);
                Self::collect_capture_names_from_expr(start, out);
                Self::collect_capture_names_from_expr(end, out);
            }
            Expr::IntlRelativeTimeFormat {
                formatter,
                value,
                unit,
            }
            | Expr::IntlRelativeTimeFormatToParts {
                formatter,
                value,
                unit,
            } => {
                Self::collect_capture_names_from_expr(formatter, out);
                Self::collect_capture_names_from_expr(value, out);
                Self::collect_capture_names_from_expr(unit, out);
            }
            Expr::IntlSegmenterSegment { segmenter, value } => {
                Self::collect_capture_names_from_expr(segmenter, out);
                Self::collect_capture_names_from_expr(value, out);
            }
            Expr::IntlLocaleConstruct { tag, options, .. } => {
                Self::collect_capture_names_from_expr(tag, out);
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::IntlLocaleMethod { locale, .. } => {
                Self::collect_capture_names_from_expr(locale, out);
            }
            Expr::RegexNew { pattern, flags } => {
                Self::collect_capture_names_from_expr(pattern, out);
                if let Some(flags) = flags.as_ref() {
                    Self::collect_capture_names_from_expr(flags, out);
                }
            }
            Expr::RegexTest { regex, input } | Expr::RegexExec { regex, input } => {
                Self::collect_capture_names_from_expr(regex, out);
                Self::collect_capture_names_from_expr(input, out);
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
            Expr::StructuredClone { value, options }
            | Expr::Fetch {
                request: value,
                options,
            } => {
                Self::collect_capture_names_from_expr(value, out);
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::MatchMediaProp { query, .. } => {
                Self::collect_capture_names_from_expr(query, out);
            }
            Expr::Prompt { message, default } => {
                Self::collect_capture_names_from_expr(message, out);
                if let Some(default) = default.as_ref() {
                    Self::collect_capture_names_from_expr(default, out);
                }
            }
            Expr::ImportCall { module, options } => {
                Self::collect_capture_names_from_expr(module, out);
                if let Some(options) = options.as_ref() {
                    Self::collect_capture_names_from_expr(options, out);
                }
            }
            Expr::Call { target, args, .. } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::MemberCall { target, args, .. }
            | Expr::PrivateMemberCall { target, args, .. } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_exprs(args, out);
            }
            Expr::MemberGet { target, .. }
            | Expr::PrivateMemberGet { target, .. }
            | Expr::PrivateIn { target, .. } => {
                Self::collect_capture_names_from_expr(target, out);
            }
            Expr::IndexGet { target, index, .. } => {
                Self::collect_capture_names_from_expr(target, out);
                Self::collect_capture_names_from_expr(index, out);
            }
            Expr::DomRef(query)
            | Expr::QuerySelectorAllLength { target: query }
            | Expr::FormElementsLength { form: query }
            | Expr::DomGetAttribute { target: query, .. }
            | Expr::DomHasAttribute { target: query, .. } => {
                Self::collect_capture_names_from_dom_query(query, out);
            }
            Expr::SetTimeout { handler, delay_ms } | Expr::SetInterval { handler, delay_ms } => {
                Self::collect_capture_names_from_timer_invocation(handler, out);
                Self::collect_capture_names_from_expr(delay_ms, out);
            }
            Expr::RequestAnimationFrame { callback } => {
                Self::collect_capture_names_from_timer_callback(callback, out);
            }
            Expr::Function { handler, .. } | Expr::QueueMicrotask { handler } => {
                Self::collect_nested_handler_capture_names(handler, out);
            }
            Expr::Binary { left, right, .. } => {
                Self::collect_capture_names_from_expr(left, out);
                Self::collect_capture_names_from_expr(right, out);
            }
            Expr::DomRead { target, .. }
            | Expr::DomMatches { target, .. }
            | Expr::DomClosest { target, .. }
            | Expr::DomComputedStyleProperty { target, .. }
            | Expr::ClassListContains { target, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
            }
            Expr::LocationMethodCall { url, .. } => {
                if let Some(url) = url.as_ref() {
                    Self::collect_capture_names_from_expr(url, out);
                }
            }
            Expr::FormDataNew { form, submitter } => {
                if let Some(form) = form.as_ref() {
                    Self::collect_capture_names_from_dom_query(form, out);
                }
                if let Some(submitter) = submitter.as_ref() {
                    Self::collect_capture_names_from_dom_query(submitter, out);
                }
            }
            Expr::FormDataGet { source, .. }
            | Expr::FormDataHas { source, .. }
            | Expr::FormDataGetAll { source, .. }
            | Expr::FormDataGetAllLength { source, .. } => {
                Self::collect_capture_names_from_form_data_source(source, out);
            }
            Expr::Ternary {
                cond,
                on_true,
                on_false,
            } => {
                Self::collect_capture_names_from_expr(cond, out);
                Self::collect_capture_names_from_expr(on_true, out);
                Self::collect_capture_names_from_expr(on_false, out);
            }
        }
    }

    fn collect_capture_names_from_object_literal_key(
        key: &ObjectLiteralKey,
        out: &mut HashSet<String>,
    ) {
        if let ObjectLiteralKey::Computed(expr) = key {
            Self::collect_capture_names_from_expr(expr, out);
        }
    }

    fn collect_capture_names_from_object_literal_entry(
        entry: &ObjectLiteralEntry,
        out: &mut HashSet<String>,
    ) {
        match entry {
            ObjectLiteralEntry::Pair(key, value) => {
                Self::collect_capture_names_from_object_literal_key(key, out);
                Self::collect_capture_names_from_expr(value, out);
            }
            ObjectLiteralEntry::ProtoSetter(value) | ObjectLiteralEntry::Spread(value) => {
                Self::collect_capture_names_from_expr(value, out);
            }
            ObjectLiteralEntry::Getter(key, handler) | ObjectLiteralEntry::Setter(key, handler) => {
                Self::collect_capture_names_from_object_literal_key(key, out);
                Self::collect_nested_handler_capture_names(handler, out);
            }
        }
    }

    pub(crate) fn collect_capture_names_from_dom_query(
        query: &DomQuery,
        out: &mut HashSet<String>,
    ) {
        match query {
            DomQuery::DocumentRoot
            | DomQuery::DocumentBody
            | DomQuery::DocumentHead
            | DomQuery::DocumentElement
            | DomQuery::ActiveElement
            | DomQuery::ById(_)
            | DomQuery::BySelector(_)
            | DomQuery::BySelectorAll { .. } => {}
            DomQuery::BySelectorAllIndex { index, .. } => {
                Self::collect_capture_names_from_dom_index(index, out);
            }
            DomQuery::QuerySelector { target, .. } | DomQuery::QuerySelectorAll { target, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
            }
            DomQuery::Index { target, index }
            | DomQuery::QuerySelectorAllIndex { target, index, .. } => {
                Self::collect_capture_names_from_dom_query(target, out);
                Self::collect_capture_names_from_dom_index(index, out);
            }
            DomQuery::FormElementsIndex { form, index } => {
                Self::collect_capture_names_from_dom_query(form, out);
                Self::collect_capture_names_from_dom_index(index, out);
            }
            DomQuery::Var(name) => {
                Self::collect_capture_name(name, out);
            }
            DomQuery::VarPath { base, .. } => {
                Self::collect_capture_name(base, out);
            }
        }
    }

    fn collect_capture_names_from_dom_index(index: &DomIndex, out: &mut HashSet<String>) {
        let DomIndex::Dynamic(raw) = index else {
            return;
        };
        if let Ok(expr) = crate::core_impl::parser::api::parse_expr(raw) {
            Self::collect_capture_names_from_expr(&expr, out);
            return;
        }
        Self::collect_capture_name(raw, out);
    }

    fn collect_capture_names_from_form_data_source(
        source: &FormDataSource,
        out: &mut HashSet<String>,
    ) {
        match source {
            FormDataSource::New { form, submitter } => {
                if let Some(form) = form.as_ref() {
                    Self::collect_capture_names_from_dom_query(form, out);
                }
                if let Some(submitter) = submitter.as_ref() {
                    Self::collect_capture_names_from_dom_query(submitter, out);
                }
            }
            FormDataSource::Var(name) => {
                Self::collect_capture_name(name, out);
            }
        }
    }

    pub(crate) fn collect_capture_names_from_timer_invocation(
        invocation: &TimerInvocation,
        out: &mut HashSet<String>,
    ) {
        Self::collect_capture_names_from_timer_callback(&invocation.callback, out);
        Self::collect_capture_names_from_exprs(&invocation.args, out);
    }

    fn collect_capture_names_from_timer_callback(
        callback: &TimerCallback,
        out: &mut HashSet<String>,
    ) {
        match callback {
            TimerCallback::Inline(handler) => {
                Self::collect_nested_handler_capture_names(handler, out);
            }
            TimerCallback::Reference(name) => {
                Self::collect_capture_name(name, out);
            }
        }
    }
}
