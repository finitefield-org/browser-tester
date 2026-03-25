package runtime

import "testing"

func TestSessionSetLocationPropertySupportsUsernameAndPassword(t *testing.T) {
	s := NewSession(SessionConfig{
		URL: "https://example.test/start",
	})

	if err := s.SetLocationProperty("username", "alice"); err != nil {
		t.Fatalf("SetLocationProperty(username) error = %v", err)
	}
	if got, want := s.URL(), "https://alice@example.test/start"; got != want {
		t.Fatalf("URL() after username assignment = %q, want %q", got, want)
	}
	if got, want := s.NavigationLog(), []string{"https://alice@example.test/start"}; len(got) != len(want) || got[0] != want[0] {
		t.Fatalf("NavigationLog() after username assignment = %#v, want %#v", got, want)
	}
	if got, want := s.HistoryLength(), 2; got != want {
		t.Fatalf("HistoryLength() after username assignment = %d, want %d", got, want)
	}

	if err := s.SetLocationProperty("password", "secret"); err != nil {
		t.Fatalf("SetLocationProperty(password) error = %v", err)
	}
	if got, want := s.URL(), "https://alice:secret@example.test/start"; got != want {
		t.Fatalf("URL() after password assignment = %q, want %q", got, want)
	}
	if got, want := s.NavigationLog(), []string{
		"https://alice@example.test/start",
		"https://alice:secret@example.test/start",
	}; len(got) != len(want) || got[0] != want[0] || got[1] != want[1] {
		t.Fatalf("NavigationLog() after password assignment = %#v, want %#v", got, want)
	}
	if got, want := s.HistoryLength(), 3; got != want {
		t.Fatalf("HistoryLength() after password assignment = %d, want %d", got, want)
	}
}
