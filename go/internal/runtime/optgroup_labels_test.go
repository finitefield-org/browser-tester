package runtime

import "testing"

func TestSessionOptgroupLabelsInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><select><optgroup id="plain" label="Group"><option>A</option></optgroup><optgroup id="legend"><legend> Legend  Name </legend><option>B</option></optgroup></select></main>`,
	})

	labels := s.OptgroupLabels()
	if len(labels) != 2 {
		t.Fatalf("OptgroupLabels() len = %d, want 2", len(labels))
	}
	if labels[0].Label != "Group" || labels[1].Label != "Legend Name" {
		t.Fatalf("OptgroupLabels() = %#v, want labels in document order", labels)
	}
	if labels[0].NodeID == 0 || labels[1].NodeID == 0 {
		t.Fatalf("OptgroupLabels() = %#v, want node IDs", labels)
	}

	labels[0].Label = "mutated"
	if fresh := s.OptgroupLabels(); len(fresh) != 2 || fresh[0].Label != "Group" || fresh[1].Label != "Legend Name" {
		t.Fatalf("OptgroupLabels() reread = %#v, want original labels", fresh)
	}

	var nilSession *Session
	if got := nilSession.OptgroupLabels(); got != nil {
		t.Fatalf("nil OptgroupLabels() = %#v, want nil", got)
	}
}
