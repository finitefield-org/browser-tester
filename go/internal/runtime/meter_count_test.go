package runtime

import "testing"

func TestSessionMeterCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><meter id="first"></meter><div id="host"></div><div><meter name="second" value="3"></meter></div></main>`,
	})

	if got := s.MeterCount(); got != 2 {
		t.Fatalf("MeterCount() = %d, want 2", got)
	}

	if err := s.SetInnerHTML("#host", `<meter id="third"></meter>`); err != nil {
		t.Fatalf("SetInnerHTML(#host) error = %v", err)
	}
	if got := s.MeterCount(); got != 3 {
		t.Fatalf("MeterCount() after SetInnerHTML = %d, want 3", got)
	}

	var nilSession *Session
	if got := nilSession.MeterCount(); got != 0 {
		t.Fatalf("nil MeterCount() = %d, want 0", got)
	}
}
