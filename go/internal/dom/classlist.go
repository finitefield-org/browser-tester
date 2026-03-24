package dom

import (
	"fmt"
	"strings"
)

type ClassList struct {
	store  *Store
	nodeID NodeID
}

func (s *Store) ClassList(nodeID NodeID) (ClassList, error) {
	if s == nil {
		return ClassList{}, fmt.Errorf("dom store is nil")
	}
	if _, err := s.elementNode(nodeID); err != nil {
		return ClassList{}, err
	}
	return ClassList{store: s, nodeID: nodeID}, nil
}

func (c ClassList) Values() []string {
	tokens, _, err := c.current()
	if err != nil || len(tokens) == 0 {
		return []string{}
	}
	out := make([]string, len(tokens))
	copy(out, tokens)
	return out
}

func (c ClassList) Contains(token string) bool {
	normalized, ok := normalizeToken(token)
	if !ok {
		return false
	}
	tokens, _, err := c.current()
	if err != nil {
		return false
	}
	for _, current := range tokens {
		if current == normalized {
			return true
		}
	}
	return false
}

func (c ClassList) Add(tokens ...string) error {
	current, hasAttr, node, err := c.currentWithNode()
	if err != nil {
		return err
	}

	seen := make(map[string]bool, len(current))
	for _, token := range current {
		seen[token] = true
	}

	changed := false
	for _, raw := range tokens {
		normalized, ok := normalizeToken(raw)
		if !ok {
			return fmt.Errorf("invalid class token: %q", raw)
		}
		if !seen[normalized] {
			current = append(current, normalized)
			seen[normalized] = true
			changed = true
		}
	}

	if !changed {
		return nil
	}

	node.Attrs = setAttribute(node.Attrs, "class", strings.Join(current, " "), true)
	if !hasAttr {
		// The attribute might have been absent; setAttribute will append a new entry.
	}
	return nil
}

func (c ClassList) Remove(tokens ...string) error {
	current, hasAttr, node, err := c.currentWithNode()
	if err != nil {
		return err
	}
	if len(current) == 0 && !hasAttr {
		return nil
	}

	removeSet := make(map[string]bool, len(tokens))
	for _, raw := range tokens {
		normalized, ok := normalizeToken(raw)
		if !ok {
			return fmt.Errorf("invalid class token: %q", raw)
		}
		removeSet[normalized] = true
	}
	if len(removeSet) == 0 {
		return nil
	}

	next := current[:0]
	changed := false
	for _, token := range current {
		if removeSet[token] {
			changed = true
			continue
		}
		next = append(next, token)
	}

	if !changed {
		return nil
	}
	current = next

	if !hasAttr && len(current) == 0 {
		return nil
	}

	// If the class attribute existed, keep it present but empty when all tokens are removed.
	node.Attrs = setAttribute(node.Attrs, "class", strings.Join(current, " "), true)
	return nil
}

func (c ClassList) current() ([]string, bool, error) {
	tokens, hasAttr, _, err := c.currentWithNode()
	return tokens, hasAttr, err
}

func (c ClassList) currentWithNode() ([]string, bool, *Node, error) {
	if c.store == nil {
		return nil, false, nil, fmt.Errorf("dom store is nil")
	}
	node, err := c.store.elementNode(c.nodeID)
	if err != nil {
		return nil, false, nil, err
	}
	value, ok := attributeValue(node.Attrs, "class")
	if !ok {
		return []string{}, false, node, nil
	}
	return strings.Fields(value), true, node, nil
}

func normalizeToken(token string) (string, bool) {
	normalized := strings.TrimSpace(token)
	if normalized == "" {
		return "", false
	}
	// DOMTokenList tokens must not contain ASCII whitespace.
	if strings.ContainsAny(normalized, " \n\r\t\f") {
		return "", false
	}
	return normalized, true
}
