package runtime

import (
	"fmt"
)

func (s *Session) InnerHTML(selector string) (string, error) {
	if s == nil {
		return "", fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return "", err
	}
	return store.InnerHTMLForNode(nodeID)
}

func (s *Session) OuterHTML(selector string) (string, error) {
	if s == nil {
		return "", fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return "", err
	}
	return store.OuterHTMLForNode(nodeID)
}

func (s *Session) SetInnerHTML(selector, markup string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	return store.SetInnerHTML(nodeID, markup)
}

func (s *Session) SetOuterHTML(selector, markup string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	return store.SetOuterHTML(nodeID, markup)
}

func (s *Session) InsertAdjacentHTML(selector, position, markup string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	return store.InsertAdjacentHTML(nodeID, position, markup)
}

func (s *Session) RemoveNode(selector string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, normalized, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	if err := store.RemoveNode(nodeID); err != nil {
		return err
	}
	if normalized == s.focusedSelector {
		s.focusedSelector = ""
	}
	return nil
}
