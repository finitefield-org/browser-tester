package runtime

import "testing"

func TestSessionTargetNodeIDIsNilSafe(t *testing.T) {
	var s *Session
	if got := s.TargetNodeID(); got != 0 {
		t.Fatalf("nil TargetNodeID() = %d, want 0", got)
	}
}

func TestSessionNavigationLogReturnsCopy(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/start",
		HTML: `<main><script>host:locationAssign("/next"); host:locationReplace("/replace")</script></main>`,
	})

	logs := s.NavigationLog()
	if len(logs) != 2 {
		t.Fatalf("NavigationLog() = %#v, want 2 entries", logs)
	}
	logs[0] = "mutated"

	fresh := s.NavigationLog()
	if len(fresh) != 2 {
		t.Fatalf("NavigationLog() after mutation = %#v, want 2 entries", fresh)
	}
	if fresh[0] != "https://example.test/next" || fresh[1] != "https://example.test/replace" {
		t.Fatalf("NavigationLog() reread = %#v, want original entries", fresh)
	}
}
