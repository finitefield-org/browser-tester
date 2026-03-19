use super::*;

impl Harness {
    pub(crate) fn try_eval_string_basic_member_call(
        &mut self,
        text: &str,
        member: &str,
        evaluated_args: &[Value],
    ) -> Result<Option<Value>> {
        let value = match member {
            "charAt" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "charAt supports zero or one argument".into(),
                    ));
                }
                let len = Self::string_char_len(text);
                let index = evaluated_args.first().map(Self::value_to_i64).unwrap_or(0);
                if index < 0 || (index as usize) >= len {
                    Value::String(String::new())
                } else {
                    Self::string_char_at(text, index as usize)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or_else(|| Value::String(String::new()))
                }
            }
            "charCodeAt" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "charCodeAt supports zero or one argument".into(),
                    ));
                }
                let chars = text.chars().collect::<Vec<_>>();
                let len = chars.len();
                let index = evaluated_args.first().map(Self::value_to_i64).unwrap_or(0);
                if index < 0 || (index as usize) >= len {
                    Value::Float(f64::NAN)
                } else {
                    let ch = chars[index as usize];
                    let code_unit = crate::js_regex::deinternalize_surrogate_marker(ch)
                        .map(|value| value as i64)
                        .unwrap_or(ch as i64);
                    Value::Number(code_unit)
                }
            }
            "codePointAt" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "codePointAt supports zero or one argument".into(),
                    ));
                }
                let chars = text.chars().collect::<Vec<_>>();
                let len = chars.len();
                let index = evaluated_args.first().map(Self::value_to_i64).unwrap_or(0);
                if index < 0 || (index as usize) >= len {
                    Value::Undefined
                } else {
                    let i = index as usize;
                    let ch = chars[i];
                    if let Some(first_unit) = crate::js_regex::deinternalize_surrogate_marker(ch) {
                        if (0xD800..=0xDBFF).contains(&first_unit) {
                            if let Some(next_ch) = chars.get(i + 1).copied() {
                                if let Some(second_unit) =
                                    crate::js_regex::deinternalize_surrogate_marker(next_ch)
                                {
                                    if (0xDC00..=0xDFFF).contains(&second_unit) {
                                        let cp = 0x10000
                                            + (((first_unit - 0xD800) as u32) << 10)
                                            + ((second_unit - 0xDC00) as u32);
                                        Value::Number(cp as i64)
                                    } else {
                                        Value::Number(first_unit as i64)
                                    }
                                } else {
                                    Value::Number(first_unit as i64)
                                }
                            } else {
                                Value::Number(first_unit as i64)
                            }
                        } else {
                            Value::Number(first_unit as i64)
                        }
                    } else {
                        Value::Number(ch as i64)
                    }
                }
            }
            "at" => {
                if evaluated_args.len() > 1 {
                    return Err(Error::ScriptRuntime(
                        "at supports zero or one argument".into(),
                    ));
                }
                let len = Self::string_char_len(text) as i64;
                let index = evaluated_args.first().map(Self::value_to_i64).unwrap_or(0);
                let index = if index < 0 { len + index } else { index };
                if index < 0 || index >= len {
                    Value::Undefined
                } else {
                    Self::string_char_at(text, index as usize)
                        .map(|ch| Value::String(ch.to_string()))
                        .unwrap_or(Value::Undefined)
                }
            }
            "concat" => {
                let mut out = text.to_string();
                for arg in evaluated_args {
                    out.push_str(&self.coerce_to_string_for_tostring(arg)?);
                }
                Value::String(out)
            }
            "trim" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime("trim does not take arguments".into()));
                }
                Value::String(text.trim().to_string())
            }
            "trimStart" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "trimStart does not take arguments".into(),
                    ));
                }
                Value::String(text.trim_start().to_string())
            }
            "trimEnd" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(
                        "trimEnd does not take arguments".into(),
                    ));
                }
                Value::String(text.trim_end().to_string())
            }
            "toUpperCase" | "toLocaleUpperCase" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} does not take arguments"
                    )));
                }
                Value::String(text.to_uppercase())
            }
            "toLowerCase" | "toLocaleLowerCase" => {
                if !evaluated_args.is_empty() {
                    return Err(Error::ScriptRuntime(format!(
                        "{member} does not take arguments"
                    )));
                }
                Value::String(text.to_lowercase())
            }
            _ => return Ok(None),
        };
        Ok(Some(value))
    }
}
