package runtime

import (
	"fmt"
	"strings"

	"browsertester/internal/dom"
)

// SelectorError marks a failure caused by selector parsing/resolution.
// It is returned by assertion helpers so the public facade can classify errors.
type SelectorError struct {
	Message string
}

func (e SelectorError) Error() string {
	if strings.TrimSpace(e.Message) == "" {
		return "selector error"
	}
	return e.Message
}

// AssertionError marks a failure caused by an unmet expectation.
// It is returned by assertion helpers so the public facade can classify errors.
type AssertionError struct {
	Message string
}

func (e AssertionError) Error() string {
	if strings.TrimSpace(e.Message) == "" {
		return "assertion failed"
	}
	return e.Message
}

func (s *Session) AssertExists(selector string) error {
	store, normalized, matches, err := s.selectForAssertion(selector)
	if err != nil {
		return err
	}
	if len(matches) == 0 {
		return AssertionError{Message: fmt.Sprintf(
			"expected selector `%s` to match at least one node\nDOM:\n%s",
			normalized,
			store.DumpDOM(),
		)}
	}
	return nil
}

func (s *Session) AssertText(selector, expected string) error {
	store, normalized, matches, err := s.selectForAssertion(selector)
	if err != nil {
		return err
	}
	if len(matches) == 0 {
		return AssertionError{Message: fmt.Sprintf(
			"expected selector `%s` to match at least one node\nDOM:\n%s",
			normalized,
			store.DumpDOM(),
		)}
	}
	nodeID := matches[0]
	actual := store.TextContentForNode(nodeID)
	if actual != expected {
		return AssertionError{Message: fmt.Sprintf(
			"expected selector `%s` to have text `%s`, got `%s`\nDOM:\n%s",
			normalized,
			expected,
			actual,
			store.DumpDOM(),
		)}
	}
	return nil
}

func (s *Session) AssertValue(selector, expected string) error {
	store, normalized, matches, err := s.selectForAssertion(selector)
	if err != nil {
		return err
	}
	if len(matches) == 0 {
		return AssertionError{Message: fmt.Sprintf(
			"expected selector `%s` to match at least one node\nDOM:\n%s",
			normalized,
			store.DumpDOM(),
		)}
	}
	nodeID := matches[0]

	actual := store.ValueForNode(nodeID)
	if node := store.Node(nodeID); node != nil && node.Kind == dom.NodeKindElement && node.TagName == "input" && inputType(node) == "file" {
		if value, ok := s.fileInputValueForSelector(normalized); ok {
			actual = value
		} else {
			actual = ""
		}
	}

	if actual != expected {
		return AssertionError{Message: fmt.Sprintf(
			"expected selector `%s` to have value `%s`, got `%s`\nDOM:\n%s",
			normalized,
			expected,
			actual,
			store.DumpDOM(),
		)}
	}
	return nil
}

func (s *Session) AssertChecked(selector string, expected bool) error {
	store, normalized, matches, err := s.selectForAssertion(selector)
	if err != nil {
		return err
	}
	if len(matches) == 0 {
		return AssertionError{Message: fmt.Sprintf(
			"expected selector `%s` to match at least one node\nDOM:\n%s",
			normalized,
			store.DumpDOM(),
		)}
	}
	nodeID := matches[0]
	actual, ok := store.CheckedForNode(nodeID)
	if !ok {
		return AssertionError{Message: fmt.Sprintf(
			"expected selector `%s` to refer to a checkable control\nDOM:\n%s",
			normalized,
			store.DumpDOM(),
		)}
	}
	if actual != expected {
		return AssertionError{Message: fmt.Sprintf(
			"expected selector `%s` to be checked `%v`, got `%v`\nDOM:\n%s",
			normalized,
			expected,
			actual,
			store.DumpDOM(),
		)}
	}
	return nil
}

func (s *Session) selectForAssertion(selector string) (*dom.Store, string, []dom.NodeID, error) {
	if s == nil {
		return nil, "", nil, fmt.Errorf("session is unavailable")
	}
	normalized := strings.TrimSpace(selector)
	if normalized == "" {
		return nil, "", nil, SelectorError{Message: "selector must not be empty"}
	}
	store, err := s.ensureDOM()
	if err != nil {
		return nil, normalized, nil, err
	}
	matches, err := store.Select(normalized)
	if err != nil {
		return store, normalized, nil, SelectorError{Message: err.Error()}
	}
	return store, normalized, matches, nil
}

func (s *Session) fileInputValueForSelector(selector string) (string, bool) {
	if s == nil {
		return "", false
	}
	normalized := strings.TrimSpace(selector)
	if normalized == "" {
		return "", false
	}
	registry := s.Registry()
	if registry == nil || registry.FileInput() == nil {
		return "", false
	}
	selections := registry.FileInput().Selections()
	for i := len(selections) - 1; i >= 0; i-- {
		item := selections[i]
		if strings.TrimSpace(item.Selector) != normalized {
			continue
		}
		return strings.Join(item.Files, ", "), true
	}
	return "", false
}
