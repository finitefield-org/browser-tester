package runtime

import "testing"

func TestSessionTrackCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><track id="first"><div id="host"></div><section><track name="second"></section></main>`,
	})

	if got := s.TrackCount(); got != 2 {
		t.Fatalf("TrackCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<track id="third">`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.TrackCount(); got != 3 {
		t.Fatalf("TrackCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.TrackCount(); got != 0 {
		t.Fatalf("nil TrackCount() = %d, want 0", got)
	}
}
