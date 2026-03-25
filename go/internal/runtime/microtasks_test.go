package runtime

import "testing"

func TestSessionPendingMicrotasksInspection(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	s.enqueueMicrotask(`host:setTextContent("#out", "micro")`)
	s.enqueueMicrotask(`host:insertAdjacentHTML("#log", "beforeend", "<span>micro</span>")`)

	microtasks := s.PendingMicrotasks()
	if len(microtasks) != 2 {
		t.Fatalf("PendingMicrotasks() len = %d, want 2", len(microtasks))
	}
	if microtasks[0] != `host:setTextContent("#out", "micro")` || microtasks[1] != `host:insertAdjacentHTML("#log", "beforeend", "<span>micro</span>")` {
		t.Fatalf("PendingMicrotasks() = %#v, want original microtasks", microtasks)
	}

	microtasks[0] = "mutated"
	fresh := s.PendingMicrotasks()
	if fresh[0] != `host:setTextContent("#out", "micro")` {
		t.Fatalf("PendingMicrotasks() reread = %#v, want original queue", fresh)
	}

	s.discardMicrotasks()
	if got := s.PendingMicrotasks(); got != nil {
		t.Fatalf("PendingMicrotasks() after discard = %#v, want nil", got)
	}

	var nilSession *Session
	if got := nilSession.PendingMicrotasks(); got != nil {
		t.Fatalf("nil PendingMicrotasks() = %#v, want nil", got)
	}
}
