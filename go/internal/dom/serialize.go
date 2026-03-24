package dom

import (
	"fmt"
	"strings"
)

func (s *Store) DumpDOM() string {
	if s == nil {
		return ""
	}
	var b strings.Builder
	for _, childID := range s.documentChildren() {
		s.serializeNode(&b, childID)
	}
	return b.String()
}

func (s *Store) OuterHTMLForNode(id NodeID) (string, error) {
	if s == nil {
		return "", fmt.Errorf("dom store is nil")
	}
	if _, ok := s.nodes[id]; !ok {
		return "", fmt.Errorf("invalid node id: %d", id)
	}
	var b strings.Builder
	s.serializeNode(&b, id)
	return b.String(), nil
}

func (s *Store) TextContentForNode(id NodeID) string {
	if s == nil {
		return ""
	}
	node := s.nodes[id]
	if node == nil {
		return ""
	}
	if node.Kind == NodeKindText {
		return node.Text
	}
	var b strings.Builder
	for _, childID := range node.Children {
		b.WriteString(s.TextContentForNode(childID))
	}
	return b.String()
}

func (s *Store) serializeNode(b *strings.Builder, id NodeID) {
	node := s.nodes[id]
	if node == nil {
		return
	}

	switch node.Kind {
	case NodeKindText:
		if s.shouldSerializeTextRaw(id) {
			b.WriteString(node.Text)
			return
		}
		b.WriteString(escapeTextContent(node.Text))
	case NodeKindElement:
		b.WriteByte('<')
		b.WriteString(node.TagName)
		for _, attr := range node.Attrs {
			b.WriteByte(' ')
			b.WriteString(attr.Name)
			if attr.HasValue {
				b.WriteString(`="`)
				b.WriteString(escapeAttributeValue(attr.Value))
				b.WriteByte('"')
			}
		}
		b.WriteByte('>')

		if isVoidElement(node.TagName) {
			return
		}

		for _, childID := range node.Children {
			s.serializeNode(b, childID)
		}
		b.WriteString("</")
		b.WriteString(node.TagName)
		b.WriteByte('>')
	}
}

func (s *Store) shouldSerializeTextRaw(id NodeID) bool {
	node := s.nodes[id]
	if node == nil || node.Kind != NodeKindText {
		return false
	}
	parent := s.nodes[node.Parent]
	if parent == nil || parent.Kind != NodeKindElement {
		return false
	}
	return parent.TagName == "script"
}

func escapeAttributeValue(value string) string {
	if value == "" {
		return value
	}
	value = strings.ReplaceAll(value, "&", "&amp;")
	value = strings.ReplaceAll(value, "<", "&lt;")
	value = strings.ReplaceAll(value, ">", "&gt;")
	value = strings.ReplaceAll(value, `"`, "&quot;")
	return value
}

func escapeTextContent(value string) string {
	if value == "" {
		return value
	}
	value = strings.ReplaceAll(value, "&", "&amp;")
	value = strings.ReplaceAll(value, "<", "&lt;")
	value = strings.ReplaceAll(value, ">", "&gt;")
	return value
}
