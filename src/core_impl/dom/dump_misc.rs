use super::*;

const SVG_NAMESPACE_URI: &str = "http://www.w3.org/2000/svg";

fn adjusted_svg_attribute_name(name: &str) -> Option<&'static str> {
    match name {
        "attributename" => Some("attributeName"),
        "attributetype" => Some("attributeType"),
        "basefrequency" => Some("baseFrequency"),
        "baseprofile" => Some("baseProfile"),
        "calcmode" => Some("calcMode"),
        "clippathunits" => Some("clipPathUnits"),
        "diffuseconstant" => Some("diffuseConstant"),
        "edgemode" => Some("edgeMode"),
        "filterunits" => Some("filterUnits"),
        "glyphref" => Some("glyphRef"),
        "gradienttransform" => Some("gradientTransform"),
        "gradientunits" => Some("gradientUnits"),
        "kernelmatrix" => Some("kernelMatrix"),
        "kernelunitlength" => Some("kernelUnitLength"),
        "keypoints" => Some("keyPoints"),
        "keysplines" => Some("keySplines"),
        "keytimes" => Some("keyTimes"),
        "lengthadjust" => Some("lengthAdjust"),
        "limitingconeangle" => Some("limitingConeAngle"),
        "markerheight" => Some("markerHeight"),
        "markerunits" => Some("markerUnits"),
        "markerwidth" => Some("markerWidth"),
        "maskcontentunits" => Some("maskContentUnits"),
        "maskunits" => Some("maskUnits"),
        "numoctaves" => Some("numOctaves"),
        "pathlength" => Some("pathLength"),
        "patterncontentunits" => Some("patternContentUnits"),
        "patterntransform" => Some("patternTransform"),
        "patternunits" => Some("patternUnits"),
        "pointsatx" => Some("pointsAtX"),
        "pointsaty" => Some("pointsAtY"),
        "pointsatz" => Some("pointsAtZ"),
        "preservealpha" => Some("preserveAlpha"),
        "preserveaspectratio" => Some("preserveAspectRatio"),
        "primitiveunits" => Some("primitiveUnits"),
        "refx" => Some("refX"),
        "refy" => Some("refY"),
        "repeatcount" => Some("repeatCount"),
        "repeatdur" => Some("repeatDur"),
        "requiredextensions" => Some("requiredExtensions"),
        "requiredfeatures" => Some("requiredFeatures"),
        "specularconstant" => Some("specularConstant"),
        "specularexponent" => Some("specularExponent"),
        "spreadmethod" => Some("spreadMethod"),
        "startoffset" => Some("startOffset"),
        "stddeviation" => Some("stdDeviation"),
        "stitchtiles" => Some("stitchTiles"),
        "surfacescale" => Some("surfaceScale"),
        "systemlanguage" => Some("systemLanguage"),
        "tablevalues" => Some("tableValues"),
        "targetx" => Some("targetX"),
        "targety" => Some("targetY"),
        "textlength" => Some("textLength"),
        "viewbox" => Some("viewBox"),
        "viewtarget" => Some("viewTarget"),
        "xchannelselector" => Some("xChannelSelector"),
        "ychannelselector" => Some("yChannelSelector"),
        "zoomandpan" => Some("zoomAndPan"),
        _ => None,
    }
}

impl Dom {
    fn is_svg_serialization_context(&self, node_id: NodeId) -> bool {
        let mut current = Some(node_id);
        let mut is_start = true;

        while let Some(node) = current {
            let parent = self.nodes[node.0].parent;
            let Some(element) = self.element(node) else {
                current = parent;
                is_start = false;
                continue;
            };

            if element.namespace_uri.as_deref() == Some(SVG_NAMESPACE_URI) {
                return true;
            }

            if !is_start && element.tag_name.eq_ignore_ascii_case("foreignobject") {
                return false;
            }

            if element.tag_name.eq_ignore_ascii_case("svg") {
                return true;
            }

            current = parent;
            is_start = false;
        }

        false
    }

    fn serialized_attribute_name<'a>(&self, node_id: NodeId, name: &'a str) -> &'a str {
        if self.is_svg_serialization_context(node_id)
            && let Some(adjusted) = adjusted_svg_attribute_name(name)
        {
            return adjusted;
        }
        name
    }

    pub(crate) fn dump_node(&self, node_id: NodeId) -> String {
        match &self.nodes[node_id.0].node_type {
            NodeType::Document => {
                let mut out = String::new();
                for child in &self.nodes[node_id.0].children {
                    out.push_str(&self.dump_node(*child));
                }
                out
            }
            NodeType::Text(text) => escape_html_text_for_serialization(text),
            NodeType::Element(element) => {
                let mut out = String::new();
                out.push('<');
                out.push_str(&element.tag_name);
                let mut attrs = element.attrs.iter().collect::<Vec<_>>();
                attrs.sort_by(|(left, _), (right, _)| left.cmp(right));
                for (k, v) in attrs {
                    out.push(' ');
                    out.push_str(self.serialized_attribute_name(node_id, k));
                    out.push_str("=\"");
                    out.push_str(&escape_html_attr_for_serialization(v));
                    out.push('"');
                }
                out.push('>');
                if is_void_tag(&element.tag_name) {
                    return out;
                }
                let raw_text_container = element.tag_name.eq_ignore_ascii_case("script")
                    || element.tag_name.eq_ignore_ascii_case("style");
                for child in &self.nodes[node_id.0].children {
                    if raw_text_container {
                        match &self.nodes[child.0].node_type {
                            NodeType::Text(text) => out.push_str(text),
                            _ => out.push_str(&self.dump_node(*child)),
                        }
                    } else {
                        out.push_str(&self.dump_node(*child));
                    }
                }
                out.push_str("</");
                out.push_str(&element.tag_name);
                out.push('>');
                out
            }
        }
    }
}
