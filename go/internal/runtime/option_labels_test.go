package runtime

import "testing"

func TestSessionOptionLabelsInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><select id="mode"><option id="first" label="Display">Fallback</option><option id="second">Text</option></select></main>`,
	})

	labels := s.OptionLabels()
	if len(labels) != 2 {
		t.Fatalf("OptionLabels() len = %d, want 2", len(labels))
	}
	if labels[0].Label != "Display" || labels[1].Label != "Text" {
		t.Fatalf("OptionLabels() = %#v, want labels in document order", labels)
	}
	if labels[0].NodeID == 0 || labels[1].NodeID == 0 {
		t.Fatalf("OptionLabels() = %#v, want node IDs", labels)
	}

	labels[0].Label = "mutated"
	if fresh := s.OptionLabels(); len(fresh) != 2 || fresh[0].Label != "Display" || fresh[1].Label != "Text" {
		t.Fatalf("OptionLabels() reread = %#v, want original labels", fresh)
	}

	var nilSession *Session
	if got := nilSession.OptionLabels(); got != nil {
		t.Fatalf("nil OptionLabels() = %#v, want nil", got)
	}
}

func TestSessionSelectedOptionLabelsInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><select id="mode" multiple><option id="first" label="Display" selected>Fallback</option><option id="second">Text</option><option id="third" selected>Third</option></select></main>`,
	})

	labels := s.SelectedOptionLabels()
	if len(labels) != 2 {
		t.Fatalf("SelectedOptionLabels() len = %d, want 2", len(labels))
	}
	if labels[0].Label != "Display" || labels[1].Label != "Third" {
		t.Fatalf("SelectedOptionLabels() = %#v, want selected labels in document order", labels)
	}
	if labels[0].NodeID == 0 || labels[1].NodeID == 0 {
		t.Fatalf("SelectedOptionLabels() = %#v, want node IDs", labels)
	}

	labels[0].Label = "mutated"
	if fresh := s.SelectedOptionLabels(); len(fresh) != 2 || fresh[0].Label != "Display" || fresh[1].Label != "Third" {
		t.Fatalf("SelectedOptionLabels() reread = %#v, want original labels", fresh)
	}

	var nilSession *Session
	if got := nilSession.SelectedOptionLabels(); got != nil {
		t.Fatalf("nil SelectedOptionLabels() = %#v, want nil", got)
	}
}
