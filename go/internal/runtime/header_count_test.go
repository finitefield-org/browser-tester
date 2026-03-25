package runtime

import "testing"

func TestSessionHeaderCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><header id="first"></header><div id="host"></div><section><header id="second"></header></section></div>`,
	})

	if got := s.HeaderCount(); got != 2 {
		t.Fatalf("HeaderCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<header id="third"></header>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.HeaderCount(); got != 3 {
		t.Fatalf("HeaderCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.HeaderCount(); got != 0 {
		t.Fatalf("nil HeaderCount() = %d, want 0", got)
	}
}
