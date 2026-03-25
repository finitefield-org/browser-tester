package runtime

import "testing"

func TestSessionLinkCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><a href="/one">One</a><a name="anchor">Anchor</a><area href="/area" alt="Area"><div><a href="/two">Two</a><a name="inner">Inner</a></div></main>`,
	})

	if got, want := s.LinkCount(), 3; got != want {
		t.Fatalf("LinkCount() = %d, want %d (DOMReady=%v, DOMError=%q)", got, want, s.DOMReady(), s.DOMError())
	}
	if got, want := s.AnchorCount(), 2; got != want {
		t.Fatalf("AnchorCount() = %d, want %d (DOMReady=%v, DOMError=%q)", got, want, s.DOMReady(), s.DOMError())
	}
	if !s.DOMReady() {
		t.Fatalf("DOMReady() = false, want true after link count bootstrap")
	}
	if got := s.DOMError(); got != "" {
		t.Fatalf("DOMError() = %q, want empty after link count bootstrap", got)
	}

	var nilSession *Session
	if got := nilSession.LinkCount(); got != 0 {
		t.Fatalf("nil LinkCount() = %d, want 0", got)
	}
	if got := nilSession.AnchorCount(); got != 0 {
		t.Fatalf("nil AnchorCount() = %d, want 0", got)
	}
}
