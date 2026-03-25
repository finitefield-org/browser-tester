package runtime

import "testing"

func TestSessionEmbedCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><embed id="first"><div id="host"></div><section><embed name="second"></section></main>`,
	})

	if got := s.EmbedCount(); got != 2 {
		t.Fatalf("EmbedCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<embed id="third">`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.EmbedCount(); got != 3 {
		t.Fatalf("EmbedCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.EmbedCount(); got != 0 {
		t.Fatalf("nil EmbedCount() = %d, want 0", got)
	}
}
