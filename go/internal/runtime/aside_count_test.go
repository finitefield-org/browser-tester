package runtime

import "testing"

func TestSessionAsideCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><aside id="first"></aside><div id="host"></div><section><aside id="second"></aside></section></div>`,
	})

	if got := s.AsideCount(); got != 2 {
		t.Fatalf("AsideCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<aside id="third"></aside>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.AsideCount(); got != 3 {
		t.Fatalf("AsideCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.AsideCount(); got != 0 {
		t.Fatalf("nil AsideCount() = %d, want 0", got)
	}
}
