package runtime

import "testing"

func TestSessionLabelCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><label id="first">A</label><div id="host"></div><div><label name="second">B</label></div></main>`,
	})

	if got := s.LabelCount(); got != 2 {
		t.Fatalf("LabelCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<label id="third">C</label>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.LabelCount(); got != 3 {
		t.Fatalf("LabelCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.LabelCount(); got != 0 {
		t.Fatalf("nil LabelCount() = %d, want 0", got)
	}
}
