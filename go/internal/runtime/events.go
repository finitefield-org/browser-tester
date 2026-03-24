package runtime

import (
	"fmt"
	"strings"

	"browsertester/internal/dom"
)

type eventListenerRecord struct {
	nodeID dom.NodeID
	event  string
	source string
}

func normalizeEventType(event string) string {
	return strings.ToLower(strings.TrimSpace(event))
}

func (s *Session) registerEventListener(nodeID dom.NodeID, event, source string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	normalized := normalizeEventType(event)
	if normalized == "" {
		return fmt.Errorf("event type must not be empty")
	}
	source = strings.TrimSpace(source)
	if source == "" {
		return fmt.Errorf("event listener source must not be empty")
	}

	s.eventListeners = append(s.eventListeners, eventListenerRecord{
		nodeID: nodeID,
		event:  normalized,
		source: source,
	})
	return nil
}

func (s *Session) dispatchEventListeners(store *dom.Store, nodeID dom.NodeID, event string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if store == nil {
		return fmt.Errorf("dom store is unavailable")
	}

	normalized := normalizeEventType(event)
	if normalized == "" {
		return nil
	}

	listeners := s.listenersForEvent(nodeID, normalized)
	for _, listener := range listeners {
		if _, err := s.runScriptOnStore(store, listener.source); err != nil {
			return err
		}
	}
	return nil
}

func (s *Session) listenersForEvent(nodeID dom.NodeID, event string) []eventListenerRecord {
	if s == nil || len(s.eventListeners) == 0 {
		return nil
	}

	out := make([]eventListenerRecord, 0, len(s.eventListeners))
	for _, listener := range s.eventListeners {
		if listener.nodeID == nodeID && listener.event == event {
			out = append(out, listener)
		}
	}
	return out
}
