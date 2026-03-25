package runtime

import "testing"

func TestSessionAddressCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<div id="root"><address id="first"></address><div id="host"></div><section><address id="second"></address></section></div>`,
	})

	if got := s.AddressCount(); got != 2 {
		t.Fatalf("AddressCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<address id="third"></address>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.AddressCount(); got != 3 {
		t.Fatalf("AddressCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.AddressCount(); got != 0 {
		t.Fatalf("nil AddressCount() = %d, want 0", got)
	}
}
