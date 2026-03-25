package runtime

import "testing"

func TestSessionOptionCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><select id="mode"><option id="first" value="Display">Fallback</option><option id="second">Text</option><div><option id="third">Ignored</option></div></select></main>`,
	})

	if got, want := s.OptionCount(), 3; got != want {
		t.Fatalf("OptionCount() = %d, want %d", got, want)
	}
	if got, want := s.SelectedOptionCount(), 0; got != want {
		t.Fatalf("SelectedOptionCount() = %d, want %d", got, want)
	}

	var nilSession *Session
	if got := nilSession.OptionCount(); got != 0 {
		t.Fatalf("nil OptionCount() = %d, want 0", got)
	}
	if got := nilSession.SelectedOptionCount(); got != 0 {
		t.Fatalf("nil SelectedOptionCount() = %d, want 0", got)
	}
}

func TestSessionSelectedOptionCountInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><select id="mode" multiple><option id="first" value="Display" selected>Fallback</option><option id="second">Text</option><option id="third" selected>Third</option></select></main>`,
	})

	if got, want := s.OptionCount(), 3; got != want {
		t.Fatalf("OptionCount() = %d, want %d", got, want)
	}
	if got, want := s.SelectedOptionCount(), 2; got != want {
		t.Fatalf("SelectedOptionCount() = %d, want %d", got, want)
	}
}
