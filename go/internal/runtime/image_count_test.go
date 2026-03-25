package runtime

import "testing"

func TestSessionImageCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><img id="first" src="/a"><div id="host"></div><img name="second" src="/b"></main>`,
	})

	if got := s.ImageCount(); got != 2 {
		t.Fatalf("ImageCount() = %d, want 2", got)
	}

	after := s.ImageCount()
	if after != s.ImageCount() {
		t.Fatalf("ImageCount() reread = %d, want stable result", s.ImageCount())
	}

	if err := s.SetInnerHTML("#host", "<img>"); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.ImageCount(); got != 3 {
		t.Fatalf("ImageCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.ImageCount(); got != 0 {
		t.Fatalf("nil ImageCount() = %d, want 0", got)
	}
}
