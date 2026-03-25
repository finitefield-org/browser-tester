package runtime

import "testing"

func FuzzSessionWindowNameRoundTrip(f *testing.F) {
	seeds := []string{
		"",
		"alpha",
		"with spaces",
		"line\nbreak",
		"🚀",
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, name string) {
		s := NewSession(DefaultSessionConfig())

		if err := s.setWindowName(name); err != nil {
			t.Fatalf("setWindowName(%q) error = %v", name, err)
		}
		if got := s.WindowName(); got != name {
			t.Fatalf("WindowName() after set = %q, want %q", got, name)
		}

		updated := name + "::updated"
		if err := s.setWindowName(updated); err != nil {
			t.Fatalf("setWindowName(%q) error = %v", updated, err)
		}
		if got := s.WindowName(); got != updated {
			t.Fatalf("WindowName() after update = %q, want %q", got, updated)
		}

		if err := s.Navigate("https://example.test/next"); err != nil {
			t.Fatalf("Navigate() error = %v", err)
		}
		if got := s.WindowName(); got != updated {
			t.Fatalf("WindowName() after Navigate = %q, want %q", got, updated)
		}

		if err := s.WriteHTML("<main></main>"); err != nil {
			t.Fatalf("WriteHTML() error = %v", err)
		}
		if got := s.WindowName(); got != updated {
			t.Fatalf("WindowName() after WriteHTML = %q, want %q", got, updated)
		}
	})
}
