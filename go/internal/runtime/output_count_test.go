package runtime

import "testing"

func TestSessionOutputCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><output id="first"></output><div id="host"></div><div><output name="second"></output></div></main>`,
	})

	if got := s.OutputCount(); got != 2 {
		t.Fatalf("OutputCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<output id="third"></output>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.OutputCount(); got != 3 {
		t.Fatalf("OutputCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.OutputCount(); got != 0 {
		t.Fatalf("nil OutputCount() = %d, want 0", got)
	}
}
