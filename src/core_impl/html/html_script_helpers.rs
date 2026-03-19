use super::*;

pub(super) fn is_executable_script_type(raw_type: Option<&str>) -> bool {
    let Some(raw_type) = raw_type else {
        return true;
    };

    let media_type = raw_type
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase();

    if media_type.is_empty() {
        return true;
    }

    matches!(
        media_type.as_str(),
        "text/javascript"
            | "application/javascript"
            | "application/ecmascript"
            | "text/ecmascript"
            | "module"
    )
}

pub(super) fn is_module_script_type(raw_type: Option<&str>) -> bool {
    let Some(raw_type) = raw_type else {
        return false;
    };

    let media_type = raw_type
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase();
    media_type == "module"
}

pub(super) fn decode_data_script_source(src: &str) -> Result<Option<String>> {
    let src = src.trim();
    let Some(rest) = src.strip_prefix("data:") else {
        return Ok(None);
    };
    let Some((meta, payload)) = rest.split_once(',') else {
        return Err(Error::HtmlParse(format!(
            "invalid script src data URL: {src}"
        )));
    };
    let is_base64 = meta
        .split(';')
        .skip(1)
        .any(|part| part.trim().eq_ignore_ascii_case("base64"));
    if is_base64 {
        let decoded = decode_base64_to_binary_string(payload)?;
        return Ok(Some(decoded));
    }
    Ok(Some(decode_uri_like(payload, true)?))
}
