package runtime

import "testing"

func TestSessionDOMReadinessInspection(t *testing.T) {
	t.Run("initially pending", func(t *testing.T) {
		s := NewSession(DefaultSessionConfig())
		if got := s.DOMReady(); got {
			t.Fatalf("DOMReady() = %v, want false before first DOM access", got)
		}
		if got := s.DOMError(); got != "" {
			t.Fatalf("DOMError() = %q, want empty before first DOM access", got)
		}
	})

	t.Run("successful bootstrap", func(t *testing.T) {
		s := NewSession(SessionConfig{HTML: `<main><button id="btn">Go</button></main>`})
		if _, err := s.ensureDOM(); err != nil {
			t.Fatalf("ensureDOM() error = %v", err)
		}
		if got := s.DOMReady(); !got {
			t.Fatalf("DOMReady() = %v, want true after successful bootstrap", got)
		}
		if got := s.DOMError(); got != "" {
			t.Fatalf("DOMError() = %q, want empty after successful bootstrap", got)
		}
	})

	t.Run("failed bootstrap", func(t *testing.T) {
		s := NewSession(SessionConfig{HTML: `<main><div id="broken"></main>`})
		if _, err := s.ensureDOM(); err == nil {
			t.Fatalf("ensureDOM() error = nil, want parse failure")
		}
		if got := s.DOMReady(); got {
			t.Fatalf("DOMReady() = %v, want false after failed bootstrap", got)
		}
		if got := s.DOMError(); got == "" {
			t.Fatalf("DOMError() = %q, want parse error text", got)
		}
	})
}
