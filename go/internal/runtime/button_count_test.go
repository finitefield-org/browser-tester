package runtime

import "testing"

func TestSessionButtonCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><button id="first"></button><div id="host"></div><div><button name="second"></button></div></main>`,
	})

	if got := s.ButtonCount(); got != 2 {
		t.Fatalf("ButtonCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<button id="third"></button>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.ButtonCount(); got != 3 {
		t.Fatalf("ButtonCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.ButtonCount(); got != 0 {
		t.Fatalf("nil ButtonCount() = %d, want 0", got)
	}
}
