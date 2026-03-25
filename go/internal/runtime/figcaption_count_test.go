package runtime

import "testing"

func TestSessionFigcaptionCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><figure id="first"><figcaption id="caption-one">One</figcaption></figure><div id="host"></div><section><figure id="second"><figcaption id="caption-two">Two</figcaption></figure></section></div>`,
	})

	if got := s.FigcaptionCount(); got != 2 {
		t.Fatalf("FigcaptionCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<figure id="third"><figcaption id="caption-three">Three</figcaption></figure>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.FigcaptionCount(); got != 3 {
		t.Fatalf("FigcaptionCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.FigcaptionCount(); got != 0 {
		t.Fatalf("nil FigcaptionCount() = %d, want 0", got)
	}
}
