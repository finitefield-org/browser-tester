package runtime

import "testing"

func TestSessionDialogCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><dialog id="first"></dialog><div id="host"></div><section><dialog name="second"></dialog></section></main>`,
	})

	if got := s.DialogCount(); got != 2 {
		t.Fatalf("DialogCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<dialog id="third"></dialog>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.DialogCount(); got != 3 {
		t.Fatalf("DialogCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.DialogCount(); got != 0 {
		t.Fatalf("nil DialogCount() = %d, want 0", got)
	}
}
