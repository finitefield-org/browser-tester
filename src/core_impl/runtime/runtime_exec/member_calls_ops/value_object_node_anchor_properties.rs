use super::*;

impl Harness {
    pub(crate) fn node_anchor_property_value(
        &mut self,
        node: NodeId,
        key: &str,
    ) -> Result<Value> {
        match key {
            "target" => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| {
                        tag.eq_ignore_ascii_case("form")
                            || tag.eq_ignore_ascii_case("a")
                            || tag.eq_ignore_ascii_case("area")
                            || tag.eq_ignore_ascii_case("base")
                    })
                {
                    Ok(Value::String(self.dom.attr(node, "target").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "href" => Ok(Value::String(self.resolve_anchor_href(node))),
            "download" => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
                {
                    Ok(Value::String(self.dom.attr(node, "download").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "hreflang" => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("link")
                }) {
                    Ok(Value::String(self.dom.attr(node, "hreflang").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "ping" => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
                {
                    Ok(Value::String(self.dom.attr(node, "ping").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "referrerPolicy" | "referrerpolicy" => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("img")
                        || tag.eq_ignore_ascii_case("link")
                        || tag.eq_ignore_ascii_case("script")
                        || tag.eq_ignore_ascii_case("iframe")
                }) {
                    Ok(Value::String(
                        self.dom.attr(node, "referrerpolicy").unwrap_or_default(),
                    ))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "rel" => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("link")
                }) {
                    Ok(Value::String(self.dom.attr(node, "rel").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "alt" => {
                if self.dom.tag_name(node).is_some_and(|tag| {
                    tag.eq_ignore_ascii_case("a")
                        || tag.eq_ignore_ascii_case("area")
                        || tag.eq_ignore_ascii_case("img")
                }) {
                    Ok(Value::String(self.dom.attr(node, "alt").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "charset" => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
                {
                    Ok(Value::String(self.dom.attr(node, "charset").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "coords" => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
                {
                    Ok(Value::String(self.dom.attr(node, "coords").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "rev" => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
                {
                    Ok(Value::String(self.dom.attr(node, "rev").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "shape" => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
                {
                    Ok(Value::String(self.dom.attr(node, "shape").unwrap_or_default()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            "noHref" | "nohref" => {
                if self
                    .dom
                    .tag_name(node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
                {
                    Ok(Value::Bool(self.dom.attr(node, "nohref").is_some()))
                } else {
                    Ok(Value::Undefined)
                }
            }
            _ => Ok(Value::Undefined),
        }
    }
}
