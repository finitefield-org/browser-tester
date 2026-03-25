package runtime

import "testing"

func TestSessionProgressCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><progress id="first"></progress><div id="host"></div><div><progress name="second" value="42"></progress></div></main>`,
	})

	if got := s.ProgressCount(); got != 2 {
		t.Fatalf("ProgressCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<progress id="third"></progress>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.ProgressCount(); got != 3 {
		t.Fatalf("ProgressCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.ProgressCount(); got != 0 {
		t.Fatalf("nil ProgressCount() = %d, want 0", got)
	}
}
