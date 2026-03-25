package runtime

import "testing"

func TestSessionSourceCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><picture><source id="first"><source id="second"></picture><audio><source id="third"></audio><div id="host"></div></main>`,
	})

	if got := s.SourceCount(); got != 3 {
		t.Fatalf("SourceCount() = %d, want 3", got)
	}

	if err := s.SetInnerHTML("#host", `<video><source id="fourth"></video>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.SourceCount(); got != 4 {
		t.Fatalf("SourceCount() after SetInnerHTML = %d, want 4", got)
	}

	var nilSession *Session
	if got := nilSession.SourceCount(); got != 0 {
		t.Fatalf("nil SourceCount() = %d, want 0", got)
	}
}
