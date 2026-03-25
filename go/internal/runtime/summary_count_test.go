package runtime

import "testing"

func TestSessionSummaryCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><details id="first"><summary id="one">One</summary></details><div id="host"></div><section><details id="second"><summary id="two">Two</summary></details></section></main>`,
	})

	if got := s.SummaryCount(); got != 2 {
		t.Fatalf("SummaryCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<details id="third"><summary id="three">Three</summary></details>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.SummaryCount(); got != 3 {
		t.Fatalf("SummaryCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.SummaryCount(); got != 0 {
		t.Fatalf("nil SummaryCount() = %d, want 0", got)
	}
}
