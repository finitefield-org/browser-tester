package runtime

import "testing"

func TestSessionOptgroupCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><select id="mode"><optgroup id="first" label="Group"><option>A</option></optgroup><optgroup id="second"><legend>Legend</legend><option>B</option></optgroup><div><optgroup id="ignored" label="Nope"></optgroup></div></select></main>`,
	})

	if got, want := s.OptgroupCount(), 3; got != want {
		t.Fatalf("OptgroupCount() = %d, want %d", got, want)
	}

	var nilSession *Session
	if got := nilSession.OptgroupCount(); got != 0 {
		t.Fatalf("nil OptgroupCount() = %d, want 0", got)
	}
}
