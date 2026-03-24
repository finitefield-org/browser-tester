package dom

import (
	"fmt"
	"strings"
)

type simpleSelector struct {
	tag     string
	anyTag  bool
	id      string
	classes []string
}

func (s *Store) Select(selector string) ([]NodeID, error) {
	if s == nil {
		return nil, fmt.Errorf("dom store is nil")
	}
	parsed, err := parseSimpleSelector(selector)
	if err != nil {
		return nil, err
	}

	matches := make([]NodeID, 0, 4)
	for _, rootID := range s.documentChildren() {
		s.walkElementPreOrder(rootID, func(node *Node) {
			if parsed.matches(node) {
				matches = append(matches, node.ID)
			}
		})
	}
	return matches, nil
}

func (s *Store) walkElementPreOrder(id NodeID, visit func(*Node)) {
	node := s.nodes[id]
	if node == nil {
		return
	}
	if node.Kind == NodeKindElement {
		visit(node)
	}
	for _, childID := range node.Children {
		s.walkElementPreOrder(childID, visit)
	}
}

func parseSimpleSelector(input string) (simpleSelector, error) {
	text := strings.TrimSpace(input)
	if text == "" {
		return simpleSelector{}, fmt.Errorf("selector must not be empty")
	}
	if strings.ContainsAny(text, " >+~[]:,") {
		return simpleSelector{}, fmt.Errorf("unsupported selector `%s`: only simple tag/#id/.class selectors are supported", input)
	}

	out := simpleSelector{}
	i := 0
	if text[0] == '*' {
		out.anyTag = true
		i++
	} else if isSelectorNameStart(text[0]) {
		start := i
		i++
		for i < len(text) && isSelectorNameChar(text[i]) {
			i++
		}
		out.tag = strings.ToLower(text[start:i])
	}

	for i < len(text) {
		switch text[i] {
		case '#':
			i++
			start := i
			for i < len(text) && isSelectorNameChar(text[i]) {
				i++
			}
			if start == i {
				return simpleSelector{}, fmt.Errorf("invalid id selector `%s`", input)
			}
			if out.id != "" {
				return simpleSelector{}, fmt.Errorf("multiple id selectors are not supported: `%s`", input)
			}
			out.id = text[start:i]
		case '.':
			i++
			start := i
			for i < len(text) && isSelectorNameChar(text[i]) {
				i++
			}
			if start == i {
				return simpleSelector{}, fmt.Errorf("invalid class selector `%s`", input)
			}
			out.classes = append(out.classes, text[start:i])
		default:
			return simpleSelector{}, fmt.Errorf("unsupported selector `%s`", input)
		}
	}

	if !out.anyTag && out.tag == "" && out.id == "" && len(out.classes) == 0 {
		return simpleSelector{}, fmt.Errorf("selector must include tag, id, or class")
	}
	return out, nil
}

func (s simpleSelector) matches(node *Node) bool {
	if node == nil || node.Kind != NodeKindElement {
		return false
	}
	if !s.anyTag && s.tag != "" && node.TagName != s.tag {
		return false
	}
	if s.id != "" {
		id, ok := attributeValue(node.Attrs, "id")
		if !ok || id != s.id {
			return false
		}
	}
	if len(s.classes) > 0 {
		classValue, ok := attributeValue(node.Attrs, "class")
		if !ok {
			return false
		}
		classList := strings.Fields(classValue)
		for _, expected := range s.classes {
			if !containsToken(classList, expected) {
				return false
			}
		}
	}
	return true
}

func attributeValue(attrs []Attribute, name string) (string, bool) {
	for _, attr := range attrs {
		if attr.Name == name {
			return attr.Value, true
		}
	}
	return "", false
}

func containsToken(tokens []string, token string) bool {
	for _, current := range tokens {
		if current == token {
			return true
		}
	}
	return false
}

func isSelectorNameStart(ch byte) bool {
	return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z') || ch == '_'
}

func isSelectorNameChar(ch byte) bool {
	return isSelectorNameStart(ch) ||
		(ch >= '0' && ch <= '9') ||
		ch == '-' || ch == ':'
}
