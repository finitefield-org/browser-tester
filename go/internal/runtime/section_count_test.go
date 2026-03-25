package runtime

import "testing"

func TestSessionSectionCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><section id="first"></section><div id="host"></div><article><section id="second"></section></article></main>`,
	})

	if got := s.SectionCount(); got != 2 {
		t.Fatalf("SectionCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<section id="third"></section>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.SectionCount(); got != 3 {
		t.Fatalf("SectionCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.SectionCount(); got != 0 {
		t.Fatalf("nil SectionCount() = %d, want 0", got)
	}
}
