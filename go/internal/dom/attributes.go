package dom

import (
	"fmt"
	"strings"
)

func (s *Store) GetAttribute(nodeID NodeID, name string) (string, bool, error) {
	if s == nil {
		return "", false, fmt.Errorf("dom store is nil")
	}
	normalized, err := normalizeAttributeName(name)
	if err != nil {
		return "", false, err
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return "", false, err
	}
	value, ok := attributeValue(node.Attrs, normalized)
	return value, ok, nil
}

func (s *Store) HasAttribute(nodeID NodeID, name string) (bool, error) {
	if s == nil {
		return false, fmt.Errorf("dom store is nil")
	}
	normalized, err := normalizeAttributeName(name)
	if err != nil {
		return false, err
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return false, err
	}
	_, ok := attributeValue(node.Attrs, normalized)
	return ok, nil
}

// SetAttribute models a bounded DOM setAttribute(name, value) slice.
// It always stores an explicit string value.
func (s *Store) SetAttribute(nodeID NodeID, name, value string) error {
	if s == nil {
		return fmt.Errorf("dom store is nil")
	}
	normalized, err := normalizeAttributeName(name)
	if err != nil {
		return err
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return err
	}
	node.Attrs = setAttribute(node.Attrs, normalized, value, true)
	return nil
}

func (s *Store) RemoveAttribute(nodeID NodeID, name string) error {
	if s == nil {
		return fmt.Errorf("dom store is nil")
	}
	normalized, err := normalizeAttributeName(name)
	if err != nil {
		return err
	}
	node, err := s.elementNode(nodeID)
	if err != nil {
		return err
	}
	node.Attrs = removeAttribute(node.Attrs, normalized)
	return nil
}

func (s *Store) elementNode(nodeID NodeID) (*Node, error) {
	node := s.Node(nodeID)
	if node == nil {
		return nil, fmt.Errorf("invalid node id: %d", nodeID)
	}
	if node.Kind != NodeKindElement {
		return nil, fmt.Errorf("node %d is not an element", nodeID)
	}
	return node, nil
}

func normalizeAttributeName(name string) (string, error) {
	normalized := strings.ToLower(strings.TrimSpace(name))
	if normalized == "" {
		return "", fmt.Errorf("attribute name must not be empty")
	}
	return normalized, nil
}
