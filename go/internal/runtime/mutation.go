package runtime

import (
	"fmt"

	"browsertester/internal/dom"
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
	if err := store.SetInnerHTML(nodeID, markup); err != nil {
		return err
	}
	if store.FocusedNodeID() == 0 {
		s.focusedSelector = ""
	}
	return nil
}

func (s *Session) SetOuterHTML(selector, markup string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	store, nodeID, _, _, err := s.resolveActionTarget(selector)
	if err != nil {
		return err
	}
	if err := store.SetOuterHTML(nodeID, markup); err != nil {
		return err
	}
	if store.FocusedNodeID() == 0 {
		s.focusedSelector = ""
	}
	return nil
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
	if store.FocusedNodeID() == 0 || normalized == s.focusedSelector {
		s.focusedSelector = ""
	}
	return nil
}

func (s *Session) WriteHTML(markup string) (err error) {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if s.writingHTML {
		return fmt.Errorf("document write is already in progress")
	}

	store := dom.NewStore()
	if err := store.BootstrapHTML(markup); err != nil {
		return err
	}

	prevStore := s.domStore
	prevReady := s.domReady
	prevErr := s.domErr
	prevFocused := s.focusedSelector
	prevListeners := append([]eventListenerRecord(nil), s.eventListeners...)
	prevNextEventListenerID := s.nextEventListenerID
	prevDispatch := s.eventDispatch
	prevMicrotasks := append([]string(nil), s.microtasks...)
	prevTimers := cloneTimerMap(s.timers)
	prevFrames := cloneAnimationFrameMap(s.animationFrames)
	prevNextTimerID := s.nextTimerID
	prevNextAnimationFrameID := s.nextAnimationFrameID
	prevRunningTimerID := s.runningTimerID
	prevRunningTimerCancelled := s.runningTimerCancelled
	prevScrollX := s.scrollX
	prevScrollY := s.scrollY
	prevWindowName := s.windowName

	s.writingHTML = true
	defer func() {
		s.writingHTML = false
	}()
	defer func() {
		if err != nil {
			s.domStore = prevStore
			s.domReady = prevReady
			s.domErr = prevErr
			s.focusedSelector = prevFocused
			s.eventListeners = prevListeners
			s.nextEventListenerID = prevNextEventListenerID
			s.eventDispatch = prevDispatch
			s.microtasks = prevMicrotasks
			s.timers = prevTimers
			s.animationFrames = prevFrames
			s.nextTimerID = prevNextTimerID
			s.nextAnimationFrameID = prevNextAnimationFrameID
			s.runningTimerID = prevRunningTimerID
			s.runningTimerCancelled = prevRunningTimerCancelled
			s.scrollX = prevScrollX
			s.scrollY = prevScrollY
			s.windowName = prevWindowName
		}
	}()

	s.discardMicrotasks()
	s.domStore = store
	s.domReady = true
	s.domErr = nil
	s.focusedSelector = ""
	s.eventListeners = nil
	s.nextEventListenerID = 0
	s.eventDispatch = nil
	s.scrollX = 0
	s.scrollY = 0
	s.syncDocumentState(s.URL())

	if err = s.executeInlineScripts(store); err != nil {
		return err
	}
	if err = s.drainMicrotasks(store); err != nil {
		return err
	}
	return nil
}
