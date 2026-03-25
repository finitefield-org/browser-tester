package runtime

import "testing"

func TestSessionAbbrCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><abbr id="first"></abbr><div id="host"></div><section><abbr id="second"></abbr></section></div>`,
	})

	if got := s.AbbrCount(); got != 2 {
		t.Fatalf("AbbrCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<abbr id="third"></abbr>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.AbbrCount(); got != 3 {
		t.Fatalf("AbbrCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.AbbrCount(); got != 0 {
		t.Fatalf("nil AbbrCount() = %d, want 0", got)
	}
}
