package runtime

import "testing"

func TestSessionLegendCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><fieldset><legend id="first"></legend></fieldset><div id="host"></div><div><legend name="second"></legend></div></main>`,
	})

	if got := s.LegendCount(); got != 2 {
		t.Fatalf("LegendCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<legend id="third"></legend>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.LegendCount(); got != 3 {
		t.Fatalf("LegendCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.LegendCount(); got != 0 {
		t.Fatalf("nil LegendCount() = %d, want 0", got)
	}
}
