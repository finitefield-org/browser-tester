package runtime

import "testing"

func TestSessionOptionValuesInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><select id="mode"><option id="first" value="Display">Fallback</option><option id="second">Text</option></select></main>`,
	})

	values := s.OptionValues()
	if len(values) != 2 {
		t.Fatalf("OptionValues() len = %d, want 2", len(values))
	}
	if values[0].Value != "Display" || values[1].Value != "Text" {
		t.Fatalf("OptionValues() = %#v, want values in document order", values)
	}
	if values[0].NodeID == 0 || values[1].NodeID == 0 {
		t.Fatalf("OptionValues() = %#v, want node IDs", values)
	}

	values[0].Value = "mutated"
	if fresh := s.OptionValues(); len(fresh) != 2 || fresh[0].Value != "Display" || fresh[1].Value != "Text" {
		t.Fatalf("OptionValues() reread = %#v, want original values", fresh)
	}

	var nilSession *Session
	if got := nilSession.OptionValues(); got != nil {
		t.Fatalf("nil OptionValues() = %#v, want nil", got)
	}
}

func TestSessionSelectedOptionValuesInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><select id="mode" multiple><option id="first" value="Display" selected>Fallback</option><option id="second">Text</option><option id="third" selected>Third</option></select></main>`,
	})

	values := s.SelectedOptionValues()
	if len(values) != 2 {
		t.Fatalf("SelectedOptionValues() len = %d, want 2", len(values))
	}
	if values[0].Value != "Display" || values[1].Value != "Third" {
		t.Fatalf("SelectedOptionValues() = %#v, want selected values in document order", values)
	}
	if values[0].NodeID == 0 || values[1].NodeID == 0 {
		t.Fatalf("SelectedOptionValues() = %#v, want node IDs", values)
	}

	values[0].Value = "mutated"
	if fresh := s.SelectedOptionValues(); len(fresh) != 2 || fresh[0].Value != "Display" || fresh[1].Value != "Third" {
		t.Fatalf("SelectedOptionValues() reread = %#v, want original values", fresh)
	}

	var nilSession *Session
	if got := nilSession.SelectedOptionValues(); got != nil {
		t.Fatalf("nil SelectedOptionValues() = %#v, want nil", got)
	}
}
