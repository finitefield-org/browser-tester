package runtime

import "testing"

func TestSessionFormCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><form id="first"></form><div id="host"></div><form name="second"></form></main>`,
	})

	if got := s.FormCount(); got != 2 {
		t.Fatalf("FormCount() = %d, want 2", got)
	}

	after := s.FormCount()
	if after != s.FormCount() {
		t.Fatalf("FormCount() reread = %d, want stable result", s.FormCount())
	}

	if err := s.SetInnerHTML("#host", "<form></form>"); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.FormCount(); got != 3 {
		t.Fatalf("FormCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.FormCount(); got != 0 {
		t.Fatalf("nil FormCount() = %d, want 0", got)
	}
}
