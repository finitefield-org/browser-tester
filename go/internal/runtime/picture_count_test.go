package runtime

import "testing"

func TestSessionPictureCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><picture id="first"></picture><div id="host"></div><section><picture name="second"></picture></section></main>`,
	})

	if got := s.PictureCount(); got != 2 {
		t.Fatalf("PictureCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<picture id="third"></picture>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.PictureCount(); got != 3 {
		t.Fatalf("PictureCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.PictureCount(); got != 0 {
		t.Fatalf("nil PictureCount() = %d, want 0", got)
	}
}
