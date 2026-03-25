package dom

import "fmt"

func (s *Store) QuerySelector(selector string) (NodeID, bool, error) {
	if s == nil {
		return 0, false, fmt.Errorf("dom store is nil")
	}
	nodes, err := s.Select(selector)
	if err != nil {
		return 0, false, err
	}
	if len(nodes) == 0 {
		return 0, false, nil
	}
	return nodes[0], true, nil
}

func (s *Store) QuerySelectorAll(selector string) (NodeList, error) {
	if s == nil {
		return NodeList{}, fmt.Errorf("dom store is nil")
	}
	nodes, err := s.Select(selector)
	if err != nil {
		return NodeList{}, err
	}
	return newNodeList(nodes), nil
}

func (s *Store) Matches(nodeID NodeID, selector string) (bool, error) {
	if s == nil {
		return false, fmt.Errorf("dom store is nil")
	}
	node := s.Node(nodeID)
	if node == nil {
		return false, fmt.Errorf("invalid node id: %d", nodeID)
	}
	parsed, err := parseSelectorSequence(selector)
	if err != nil {
		return false, err
	}
	return parsed.matchesWithScope(s, node, nodeID), nil
}

func (s *Store) Closest(nodeID NodeID, selector string) (NodeID, bool, error) {
	if s == nil {
		return 0, false, fmt.Errorf("dom store is nil")
	}
	parsed, err := parseSelectorSequence(selector)
	if err != nil {
		return 0, false, err
	}

	current := nodeID
	for current != 0 {
		node := s.Node(current)
		if node == nil {
			return 0, false, fmt.Errorf("invalid node id: %d", nodeID)
		}
		if parsed.matchesWithScope(s, node, nodeID) {
			return current, true, nil
		}
		current = node.Parent
	}

	return 0, false, nil
}
