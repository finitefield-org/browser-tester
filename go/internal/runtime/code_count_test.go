package runtime

import "testing"

func TestSessionCodeCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><code id="first"></code><div id="host"></div><section><code id="second"></code></section></div>`,
	})

	if got := s.CodeCount(); got != 2 {
		t.Fatalf("CodeCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<code id="third"></code>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.CodeCount(); got != 3 {
		t.Fatalf("CodeCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.CodeCount(); got != 0 {
		t.Fatalf("nil CodeCount() = %d, want 0", got)
	}
}
