package runtime

import "testing"

func TestSessionInputCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><input id="first"><div id="host"></div><div><input name="second"></div></main>`,
	})

	if got := s.InputCount(); got != 2 {
		t.Fatalf("InputCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<input id="third">`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.InputCount(); got != 3 {
		t.Fatalf("InputCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.InputCount(); got != 0 {
		t.Fatalf("nil InputCount() = %d, want 0", got)
	}
}
