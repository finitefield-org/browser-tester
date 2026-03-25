package runtime

import "testing"

func TestSessionAudioAndVideoCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><audio id="first"></audio><video id="movie"></video><div id="host"></div><section><audio name="second"></audio><video name="third"></video></section></main>`,
	})

	if got := s.AudioCount(); got != 2 {
		t.Fatalf("AudioCount() = %d, want 2", got)
	}
	if got := s.VideoCount(); got != 2 {
		t.Fatalf("VideoCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<audio id="fourth"></audio><video id="fifth"></video>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.AudioCount(); got != 3 {
		t.Fatalf("AudioCount() after SetInnerHTML = %d, want 3", got)
	}
	if got := s.VideoCount(); got != 3 {
		t.Fatalf("VideoCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.AudioCount(); got != 0 {
		t.Fatalf("nil AudioCount() = %d, want 0", got)
	}
	if got := nilSession.VideoCount(); got != 0 {
		t.Fatalf("nil VideoCount() = %d, want 0", got)
	}
}
