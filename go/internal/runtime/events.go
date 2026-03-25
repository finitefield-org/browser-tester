package runtime

import (
	"fmt"
	"strings"

	"browsertester/internal/dom"
)

type eventPhase string

const (
	eventPhaseTarget  eventPhase = "target"
	eventPhaseCapture eventPhase = "capture"
	eventPhaseBubble  eventPhase = "bubble"
)

type eventListenerRecord struct {
	id     int64
	nodeID dom.NodeID
	event  string
	phase  eventPhase
	source string
	once   bool
}

type eventDispatchContext struct {
	defaultPrevented   bool
	propagationStopped bool
}

type EventListenerRegistration struct {
	NodeID int64
	Event  string
	Phase  string
	Source string
	Once   bool
}

func normalizeEventType(event string) string {
	return strings.ToLower(strings.TrimSpace(event))
}

func parseEventPhase(phase string) (eventPhase, error) {
	normalized := strings.ToLower(strings.TrimSpace(phase))
	switch normalized {
	case "", string(eventPhaseTarget):
		return eventPhaseTarget, nil
	case string(eventPhaseCapture):
		return eventPhaseCapture, nil
	case string(eventPhaseBubble):
		return eventPhaseBubble, nil
	default:
		return "", fmt.Errorf("unsupported event listener phase %q", phase)
	}
}

func (s *Session) registerEventListener(nodeID dom.NodeID, event, source, phase string, once bool) error {
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
	normalizedPhase, err := parseEventPhase(phase)
	if err != nil {
		return err
	}

	s.nextEventListenerID++
	id := s.nextEventListenerID
	s.eventListeners = append(s.eventListeners, eventListenerRecord{
		id:     id,
		nodeID: nodeID,
		event:  normalized,
		phase:  normalizedPhase,
		source: source,
		once:   once,
	})
	return nil
}

func (s *Session) EventListeners() []EventListenerRegistration {
	if s == nil {
		return nil
	}
	if _, err := s.ensureDOM(); err != nil {
		return nil
	}
	if len(s.eventListeners) == 0 {
		return nil
	}
	out := make([]EventListenerRegistration, len(s.eventListeners))
	for i := range s.eventListeners {
		out[i] = EventListenerRegistration{
			NodeID: int64(s.eventListeners[i].nodeID),
			Event:  s.eventListeners[i].event,
			Phase:  string(s.eventListeners[i].phase),
			Source: s.eventListeners[i].source,
			Once:   s.eventListeners[i].once,
		}
	}
	return out
}

func (s *Session) dispatchEventListeners(store *dom.Store, nodeID dom.NodeID, event string) (bool, error) {
	return s.dispatchEventListenersWithPropagation(store, nodeID, event, true, true)
}

func (s *Session) dispatchTargetEventListeners(store *dom.Store, nodeID dom.NodeID, event string) (bool, error) {
	return s.dispatchEventListenersWithPropagation(store, nodeID, event, false, false)
}

func (s *Session) dispatchEventListenersWithPropagation(store *dom.Store, nodeID dom.NodeID, event string, capture, bubble bool) (bool, error) {
	if s == nil {
		return false, fmt.Errorf("session is unavailable")
	}
	if store == nil {
		return false, fmt.Errorf("dom store is unavailable")
	}

	normalized := normalizeEventType(event)
	if normalized == "" {
		return false, nil
	}

	prev := s.eventDispatch
	ctx := &eventDispatchContext{}
	s.eventDispatch = ctx
	defer func() {
		s.eventDispatch = prev
	}()

	path := s.eventPath(store, nodeID)
	if capture {
		for i := len(path) - 1; i >= 0; i-- {
			if err := s.dispatchListenersOnNode(store, path[i], normalized, eventPhaseCapture); err != nil {
				return ctx.defaultPrevented, err
			}
			if s.domStore != nil && s.domStore != store {
				return ctx.defaultPrevented, nil
			}
			if ctx.propagationStopped {
				break
			}
		}
	}
	if err := s.dispatchListenersOnNode(store, nodeID, normalized, eventPhaseTarget); err != nil {
		return ctx.defaultPrevented, err
	}
	if s.domStore != nil && s.domStore != store {
		return ctx.defaultPrevented, nil
	}
	if bubble && !ctx.propagationStopped {
		for i := 0; i < len(path); i++ {
			if err := s.dispatchListenersOnNode(store, path[i], normalized, eventPhaseBubble); err != nil {
				return ctx.defaultPrevented, err
			}
			if s.domStore != nil && s.domStore != store {
				return ctx.defaultPrevented, nil
			}
			if ctx.propagationStopped {
				break
			}
		}
	}
	return ctx.defaultPrevented, nil
}

func (s *Session) dispatchListenersOnNode(store *dom.Store, nodeID dom.NodeID, event string, phase eventPhase) error {
	listeners := s.listenersForEvent(nodeID, event, phase)
	for _, listener := range listeners {
		if s.domStore != nil && s.domStore != store {
			return nil
		}
		if !s.eventListenerExists(listener.id) {
			continue
		}
		if _, err := s.runScriptOnStore(store, listener.source); err != nil {
			if listener.once {
				s.removeEventListenerByID(listener.id)
			}
			return err
		}
		if s.domStore != nil && s.domStore != store {
			return nil
		}
		if listener.once {
			s.removeEventListenerByID(listener.id)
		}
	}
	return nil
}

func (s *Session) listenersForEvent(nodeID dom.NodeID, event string, phase eventPhase) []eventListenerRecord {
	if s == nil || len(s.eventListeners) == 0 {
		return nil
	}

	out := make([]eventListenerRecord, 0, len(s.eventListeners))
	for _, listener := range s.eventListeners {
		if listener.nodeID == nodeID && listener.event == event && listener.phase == phase {
			out = append(out, listener)
		}
	}
	return out
}

func (s *Session) eventPath(store *dom.Store, nodeID dom.NodeID) []dom.NodeID {
	if s == nil || store == nil || nodeID == 0 {
		return nil
	}

	path := make([]dom.NodeID, 0, 4)
	current := nodeID
	for current != 0 {
		node := store.Node(current)
		if node == nil {
			break
		}
		path = append(path, current)
		current = node.Parent
	}
	return path
}

func (s *Session) eventListenerExists(id int64) bool {
	if s == nil || len(s.eventListeners) == 0 {
		return false
	}
	for _, listener := range s.eventListeners {
		if listener.id == id {
			return true
		}
	}
	return false
}

func (s *Session) removeEventListenerByID(id int64) {
	if s == nil || len(s.eventListeners) == 0 {
		return
	}
	for i, listener := range s.eventListeners {
		if listener.id != id {
			continue
		}
		copy(s.eventListeners[i:], s.eventListeners[i+1:])
		s.eventListeners = s.eventListeners[:len(s.eventListeners)-1]
		return
	}
}

func (s *Session) removeEventListener(nodeID dom.NodeID, event, source, phase string) (bool, error) {
	if s == nil {
		return false, fmt.Errorf("session is unavailable")
	}
	normalized := normalizeEventType(event)
	if normalized == "" {
		return false, fmt.Errorf("event type must not be empty")
	}
	source = strings.TrimSpace(source)
	if source == "" {
		return false, fmt.Errorf("event listener source must not be empty")
	}
	normalizedPhase, err := parseEventPhase(phase)
	if err != nil {
		return false, err
	}

	for i, listener := range s.eventListeners {
		if listener.nodeID != nodeID || listener.event != normalized || listener.phase != normalizedPhase || listener.source != source {
			continue
		}
		copy(s.eventListeners[i:], s.eventListeners[i+1:])
		s.eventListeners = s.eventListeners[:len(s.eventListeners)-1]
		return true, nil
	}
	return false, nil
}

func (s *Session) preventDefault() error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if s.eventDispatch == nil {
		return fmt.Errorf("preventDefault() requires an active event dispatch")
	}
	s.eventDispatch.defaultPrevented = true
	return nil
}

func (s *Session) stopPropagation() error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if s.eventDispatch == nil {
		return fmt.Errorf("stopPropagation() requires an active event dispatch")
	}
	s.eventDispatch.propagationStopped = true
	return nil
}
