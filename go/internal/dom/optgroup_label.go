package dom

import "strings"

// OptgroupLabelForNode returns the bounded label reflection for an optgroup element.
// It prefers the first child legend's text content when present, otherwise it
// falls back to the label attribute.
func (s *Store) OptgroupLabelForNode(nodeID NodeID) string {
	if s == nil {
		return ""
	}
	node := s.Node(nodeID)
	if node == nil || node.Kind != NodeKindElement || node.TagName != "optgroup" {
		return ""
	}
	for _, childID := range node.Children {
		child := s.Node(childID)
		if child == nil || child.Kind != NodeKindElement || child.TagName != "legend" {
			continue
		}
		return strings.Join(strings.Fields(s.TextContentForNode(childID)), " ")
	}
	if label, ok := attributeValue(node.Attrs, "label"); ok {
		return label
	}
	return ""
}
