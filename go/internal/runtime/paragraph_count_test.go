package runtime

import "testing"

func TestSessionParagraphCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><p id="first"></p><div id="host"></div><section><p id="second"></p></section></div>`,
	})

	if got := s.ParagraphCount(); got != 2 {
		t.Fatalf("ParagraphCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<p id="third"></p>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.ParagraphCount(); got != 3 {
		t.Fatalf("ParagraphCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.ParagraphCount(); got != 0 {
		t.Fatalf("nil ParagraphCount() = %d, want 0", got)
	}
}
