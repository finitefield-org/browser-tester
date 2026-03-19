use super::*;

#[path = "capture_name_expr_data_collection_analysis.rs"]
mod capture_name_expr_data_collection_analysis;
#[path = "capture_name_expr_platform_analysis.rs"]
mod capture_name_expr_platform_analysis;

impl Harness {
    pub(crate) fn collect_capture_names_from_exprs(exprs: &[Expr], out: &mut HashSet<String>) {
        for expr in exprs {
            Self::collect_capture_names_from_expr(expr, out);
        }
    }

    pub(crate) fn collect_capture_names_from_expr(expr: &Expr, out: &mut HashSet<String>) {
        if Self::collect_capture_names_from_data_collection_expr(expr, out)
            || Self::collect_capture_names_from_platform_expr(expr, out)
        {
            return;
        }

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
            Expr::Binary { left, right, .. } => {
                Self::collect_capture_names_from_expr(left, out);
                Self::collect_capture_names_from_expr(right, out);
            }
            _ => {}
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

    pub(crate) fn collect_capture_names_from_object_literal_entry(
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

    pub(crate) fn collect_capture_names_from_form_data_source(
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

    pub(crate) fn collect_capture_names_from_timer_callback(
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
