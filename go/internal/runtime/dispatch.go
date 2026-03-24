package runtime

import (
	"fmt"
)

func (s *Session) Dispatch(selector, event string) (err error) {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	normalizedEvent := normalizeEventType(event)
	if normalizedEvent == "" {
		return fmt.Errorf("event type must not be empty")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	defer func() {
		if err != nil {
			s.discardMicrotasks()
		}
	}()
	if _, err := s.dispatchEventListeners(store, nodeID, normalizedEvent); err != nil {
		return err
	}
	return s.drainMicrotasks(store)
}

func (s *Session) DispatchKeyboard(selector string) (err error) {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	defer func() {
		if err != nil {
			s.discardMicrotasks()
		}
	}()
	for _, event := range []string{"keydown", "keypress", "keyup"} {
		if _, err := s.dispatchEventListeners(store, nodeID, event); err != nil {
			return err
		}
	}
	return s.drainMicrotasks(store)
}
