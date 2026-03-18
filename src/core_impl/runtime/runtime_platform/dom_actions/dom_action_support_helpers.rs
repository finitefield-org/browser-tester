use super::*;
use crate::core_dom_utils::{decode_base64_to_binary_string, decode_uri_like};

/// Internal DOM-action helper routines shared by user actions and assertions.
impl Harness {
    pub(crate) fn input_supports_required(kind: &str) -> bool {
        !matches!(
            kind,
            "hidden" | "range" | "color" | "button" | "submit" | "reset" | "image"
        )
    }

    pub(crate) fn is_labelable_control(&self, node: NodeId) -> bool {
        let Some(tag) = self.dom.tag_name(node) else {
            return false;
        };

        if tag.eq_ignore_ascii_case("input") {
            let input_type = self
                .dom
                .attr(node, "type")
                .unwrap_or_else(|| "text".to_string())
                .to_ascii_lowercase();
            return input_type != "hidden";
        }

        tag.eq_ignore_ascii_case("button")
            || tag.eq_ignore_ascii_case("select")
            || tag.eq_ignore_ascii_case("textarea")
            || tag.eq_ignore_ascii_case("output")
    }

    pub(crate) fn resolve_label_control(&self, label: NodeId) -> Option<NodeId> {
        if !self
            .dom
            .tag_name(label)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("label"))
        {
            return None;
        }

        if let Some(target_id) = self.dom.attr(label, "for") {
            if let Some(target) = self.dom.by_id(&target_id) {
                if self.is_labelable_control(target) {
                    return Some(target);
                }
            }
        }

        let mut descendants = Vec::new();
        self.dom
            .collect_elements_descendants_dfs(label, &mut descendants);
        descendants
            .into_iter()
            .find(|candidate| self.is_labelable_control(*candidate))
    }

    pub(crate) fn resolve_label_activation_control(&self, target: NodeId) -> Option<NodeId> {
        if self.is_labelable_control(target) {
            return None;
        }

        let label = if self
            .dom
            .tag_name(target)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("label"))
        {
            target
        } else {
            self.dom.find_ancestor_by_tag(target, "label")?
        };

        let mut cursor = self.dom.parent(target);
        while let Some(node) = cursor {
            if node == label {
                break;
            }
            if self.is_labelable_control(node) {
                return None;
            }
            cursor = self.dom.parent(node);
        }

        self.resolve_label_control(label)
            .filter(|control| *control != target)
    }

    pub(crate) fn resolve_details_for_summary_click(&self, target: NodeId) -> Option<NodeId> {
        let summary = if self
            .dom
            .tag_name(target)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("summary"))
        {
            Some(target)
        } else {
            self.dom.find_ancestor_by_tag(target, "summary")
        }?;

        let details = self.dom.parent(summary)?;
        if !self
            .dom
            .tag_name(details)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("details"))
        {
            return None;
        }

        let first_summary_child = self.dom.nodes[details.0]
            .children
            .iter()
            .copied()
            .find(|node| {
                self.dom
                    .tag_name(*node)
                    .is_some_and(|tag| tag.eq_ignore_ascii_case("summary"))
            });
        if first_summary_child != Some(summary) {
            return None;
        }
        Some(details)
    }

    pub(crate) fn is_hyperlink_element(&self, node: NodeId) -> bool {
        self.dom
            .tag_name(node)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("area"))
    }

    fn parse_data_url_download_artifact(href: &str) -> Result<Option<(Option<String>, Vec<u8>)>> {
        let Some((scheme, rest)) = href.split_once(':') else {
            return Ok(None);
        };
        if !scheme.eq_ignore_ascii_case("data") {
            return Ok(None);
        }

        let Some((meta, payload)) = rest.split_once(',') else {
            return Ok(None);
        };
        let is_base64 = meta
            .split(';')
            .skip(1)
            .any(|part| part.trim().eq_ignore_ascii_case("base64"));
        let media_type = meta
            .split(';')
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_lowercase())
            .or_else(|| Some("text/plain".to_string()));
        let bytes = if is_base64 {
            decode_base64_to_binary_string(payload)?
                .chars()
                .map(|ch| ch as u32 as u8)
                .collect()
        } else {
            decode_uri_like(payload, true)?.into_bytes()
        };
        Ok(Some((media_type, bytes)))
    }

    pub(crate) fn maybe_capture_anchor_download(&mut self, target: NodeId) -> Result<bool> {
        if !self.is_hyperlink_element(target) {
            return Ok(false);
        }
        let Some(filename) = self.dom.attr(target, "download") else {
            return Ok(false);
        };

        let href = self.resolve_anchor_href(target);
        let (mime_type, bytes) = if let Some(blob) =
            self.browser_apis.blob_url_objects.get(&href).cloned()
        {
            let blob = blob.borrow();
            let mime_type = if blob.mime_type.is_empty() {
                None
            } else {
                Some(blob.mime_type.clone())
            };
            (mime_type, blob.bytes.clone())
        } else if let Some((mime_type, bytes)) = Self::parse_data_url_download_artifact(&href)? {
            (mime_type, bytes)
        } else {
            return Ok(false);
        };

        self.browser_apis.downloads.push(DownloadArtifact {
            filename: if filename.is_empty() {
                None
            } else {
                Some(filename)
            },
            mime_type,
            bytes,
        });
        Ok(true)
    }

    pub(crate) fn maybe_follow_anchor_hyperlink(&mut self, target: NodeId) -> Result<()> {
        if !self.is_hyperlink_element(target) {
            return Ok(());
        }
        if self
            .dom
            .tag_name(target)
            .is_some_and(|tag| tag.eq_ignore_ascii_case("area"))
            && self.dom.attr(target, "nohref").is_some()
        {
            return Ok(());
        }
        if self.dom.attr(target, "href").is_none() {
            return Ok(());
        }
        if self.dom.attr(target, "download").is_some() {
            return Ok(());
        }

        let target_attr = self
            .dom
            .attr(target, "target")
            .unwrap_or_else(|| self.default_hyperlink_target())
            .to_ascii_lowercase();
        if !matches!(
            target_attr.as_str(),
            "" | "_self" | "_parent" | "_top" | "_unfencedtop"
        ) {
            return Ok(());
        }

        let href = self.resolve_anchor_href(target);
        if href
            .split_once(':')
            .is_some_and(|(scheme, _)| scheme.eq_ignore_ascii_case("javascript"))
        {
            return Ok(());
        }
        if self.try_resolve_location_target_url(&href).is_err() {
            return Ok(());
        }
        self.navigate_location(&href, LocationNavigationKind::Assign)
    }

    pub(crate) fn is_within_first_legend_of_fieldset(
        &self,
        node: NodeId,
        fieldset: NodeId,
    ) -> bool {
        let first_legend = self.dom.child_elements(fieldset).into_iter().find(|child| {
            self.dom
                .tag_name(*child)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("legend"))
        });
        let Some(first_legend) = first_legend else {
            return false;
        };

        node == first_legend || self.dom.is_descendant_of(node, first_legend)
    }

    pub(crate) fn is_effectively_disabled(&self, node: NodeId) -> bool {
        if self.dom.disabled(node) {
            return true;
        }
        if !is_form_control(&self.dom, node) {
            return false;
        }

        let mut cursor = self.dom.parent(node);
        while let Some(parent) = cursor {
            if self
                .dom
                .tag_name(parent)
                .is_some_and(|tag| tag.eq_ignore_ascii_case("fieldset"))
                && self.dom.disabled(parent)
            {
                if self.is_within_first_legend_of_fieldset(node, parent) {
                    cursor = self.dom.parent(parent);
                    continue;
                }
                return true;
            }
            cursor = self.dom.parent(parent);
        }

        false
    }
}
