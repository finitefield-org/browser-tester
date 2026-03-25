package runtime

import "testing"

func TestSessionFigureCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><figure id="first"></figure><div id="host"></div><section><figure id="second"></figure></section></div>`,
	})

	if got := s.FigureCount(); got != 2 {
		t.Fatalf("FigureCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<figure id="third"></figure>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.FigureCount(); got != 3 {
		t.Fatalf("FigureCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.FigureCount(); got != 0 {
		t.Fatalf("nil FigureCount() = %d, want 0", got)
	}
}
