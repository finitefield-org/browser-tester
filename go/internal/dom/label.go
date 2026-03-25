package dom

// OptionLabelForNode returns the bounded label reflection for an option element.
// It prefers the label attribute when present and non-empty, otherwise it falls
// back to the option's text content.
func (s *Store) OptionLabelForNode(nodeID NodeID) string {
	if s == nil {
		return ""
	}
	node := s.Node(nodeID)
	if node == nil || node.Kind != NodeKindElement || node.TagName != "option" {
		return ""
	}
	if label, ok := attributeValue(node.Attrs, "label"); ok && label != "" {
		return label
	}
	return s.TextContentForNode(nodeID)
}
