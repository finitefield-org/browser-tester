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

type selectorCombinator uint8

const (
	selectorCombinatorNone selectorCombinator = iota
	selectorCombinatorDescendant
	selectorCombinatorChild
)

type selectorSequence struct {
	parts []selectorSequencePart
}

type selectorSequencePart struct {
	compound   simpleSelector
	combinator selectorCombinator
}

func (s *Store) Select(selector string) ([]NodeID, error) {
	if s == nil {
		return nil, fmt.Errorf("dom store is nil")
	}
	parsed, err := parseSelectorSequence(selector)
	if err != nil {
		return nil, err
	}

	matches := make([]NodeID, 0, 4)
	for _, rootID := range s.documentChildren() {
		s.walkElementPreOrder(rootID, func(node *Node) {
			if parsed.matches(s, node) {
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

func parseSelectorSequence(input string) (selectorSequence, error) {
	text := strings.TrimSpace(input)
	if text == "" {
		return selectorSequence{}, fmt.Errorf("selector must not be empty")
	}

	parts := make([]selectorSequencePart, 0, 4)
	i := 0
	for {
		i = skipSpaces(text, i)
		if i >= len(text) {
			break
		}
		if text[i] == '>' {
			return selectorSequence{}, fmt.Errorf("unsupported selector `%s`: combinators must separate selector compounds", input)
		}

		start := i
		for i < len(text) && !isSelectorCompoundTerminator(text[i]) {
			i++
		}
		if start == i {
			return selectorSequence{}, fmt.Errorf("unsupported selector `%s`", input)
		}

		compound, err := parseSimpleSelector(text[start:i])
		if err != nil {
			return selectorSequence{}, err
		}
		parts = append(parts, selectorSequencePart{compound: compound})

		j := i
		hadSpace := false
		for j < len(text) && isSpace(text[j]) {
			hadSpace = true
			j++
		}
		if j >= len(text) {
			break
		}

		switch text[j] {
		case '>':
			parts[len(parts)-1].combinator = selectorCombinatorChild
			i = j + 1
		default:
			if hadSpace {
				parts[len(parts)-1].combinator = selectorCombinatorDescendant
				i = j
			} else {
				return selectorSequence{}, fmt.Errorf("unsupported selector `%s`", input)
			}
		}
	}

	if len(parts) == 0 {
		return selectorSequence{}, fmt.Errorf("selector must not be empty")
	}
	if parts[len(parts)-1].combinator != selectorCombinatorNone {
		return selectorSequence{}, fmt.Errorf("unsupported selector `%s`: trailing combinator", input)
	}
	return selectorSequence{parts: parts}, nil
}

func (s selectorSequence) matches(store *Store, node *Node) bool {
	if store == nil || node == nil || node.Kind != NodeKindElement {
		return false
	}
	if len(s.parts) == 0 {
		return false
	}

	last := len(s.parts) - 1
	if !s.parts[last].compound.matches(node) {
		return false
	}

	current := node
	for i := last - 1; i >= 0; i-- {
		switch s.parts[i].combinator {
		case selectorCombinatorChild:
			parentID := current.Parent
			if parentID == 0 {
				return false
			}
			parent := store.Node(parentID)
			if parent == nil || !s.parts[i].compound.matches(parent) {
				return false
			}
			current = parent
		case selectorCombinatorDescendant:
			found := false
			parentID := current.Parent
			for parentID != 0 {
				parent := store.Node(parentID)
				if parent == nil {
					return false
				}
				if s.parts[i].compound.matches(parent) {
					current = parent
					found = true
					break
				}
				parentID = parent.Parent
			}
			if !found {
				return false
			}
		default:
			return false
		}
	}

	return true
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

func isSelectorCompoundTerminator(ch byte) bool {
	return isSpace(ch) || ch == '>'
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
