package runtime

import "testing"

func TestSessionTextAreaCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><textarea id="first"></textarea><div id="host"></div><div><textarea name="second"></textarea></div></main>`,
	})

	if got := s.TextAreaCount(); got != 2 {
		t.Fatalf("TextAreaCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<textarea id="third"></textarea>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.TextAreaCount(); got != 3 {
		t.Fatalf("TextAreaCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.TextAreaCount(); got != 0 {
		t.Fatalf("nil TextAreaCount() = %d, want 0", got)
	}
}
