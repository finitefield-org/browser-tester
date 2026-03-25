package runtime

import (
	"strings"
	"testing"
)

func TestSessionHistoryNavigationAndState(t *testing.T) {
	s := NewSession(SessionConfig{
		URL: "https://example.test/app",
	})

	if got, want := s.windowHistoryLength(), 1; got != want {
		t.Fatalf("windowHistoryLength() = %d, want %d", got, want)
	}
	if got, ok := s.windowHistoryState(); ok || got != "null" {
		t.Fatalf("windowHistoryState() = (%q, %v), want (\"null\", false)", got, ok)
	}
	if got, want := s.windowHistoryScrollRestoration(), "auto"; got != want {
		t.Fatalf("windowHistoryScrollRestoration() = %q, want %q", got, want)
	}

	if err := s.windowHistoryPushState("step-1", "", "https://example.test/step-1"); err != nil {
		t.Fatalf("windowHistoryPushState() error = %v", err)
	}
	if got, want := s.windowHistoryLength(), 2; got != want {
		t.Fatalf("windowHistoryLength() after pushState = %d, want %d", got, want)
	}
	if got, ok := s.windowHistoryState(); !ok || got != "step-1" {
		t.Fatalf("windowHistoryState() after pushState = (%q, %v), want (\"step-1\", true)", got, ok)
	}
	if got, want := s.URL(), "https://example.test/step-1"; got != want {
		t.Fatalf("URL() after pushState = %q, want %q", got, want)
	}

	if err := s.windowHistoryReplaceState("step-2", "", "https://example.test/step-2"); err != nil {
		t.Fatalf("windowHistoryReplaceState() error = %v", err)
	}
	if got, want := s.windowHistoryLength(), 2; got != want {
		t.Fatalf("windowHistoryLength() after replaceState = %d, want %d", got, want)
	}
	if got, ok := s.windowHistoryState(); !ok || got != "step-2" {
		t.Fatalf("windowHistoryState() after replaceState = (%q, %v), want (\"step-2\", true)", got, ok)
	}
	if got, want := s.URL(), "https://example.test/step-2"; got != want {
		t.Fatalf("URL() after replaceState = %q, want %q", got, want)
	}

	if err := s.ReloadLocation(); err != nil {
		t.Fatalf("ReloadLocation() error = %v", err)
	}
	if got, want := s.windowHistoryLength(), 2; got != want {
		t.Fatalf("windowHistoryLength() after reload = %d, want %d", got, want)
	}
	if got, ok := s.windowHistoryState(); !ok || got != "step-2" {
		t.Fatalf("windowHistoryState() after reload = (%q, %v), want (\"step-2\", true)", got, ok)
	}
	if got := s.Registry().Location().Navigations(); len(got) != 3 {
		t.Fatalf("Location().Navigations() after reload = %#v, want 3 records", got)
	}

	if err := s.windowHistoryBack(); err != nil {
		t.Fatalf("windowHistoryBack() error = %v", err)
	}
	if got, ok := s.windowHistoryState(); ok || got != "null" {
		t.Fatalf("windowHistoryState() after back = (%q, %v), want (\"null\", false)", got, ok)
	}
	if got, want := s.URL(), "https://example.test/app"; got != want {
		t.Fatalf("URL() after back = %q, want %q", got, want)
	}

	if err := s.windowHistoryForward(); err != nil {
		t.Fatalf("windowHistoryForward() error = %v", err)
	}
	if got, ok := s.windowHistoryState(); !ok || got != "step-2" {
		t.Fatalf("windowHistoryState() after forward = (%q, %v), want (\"step-2\", true)", got, ok)
	}
	if got, want := s.URL(), "https://example.test/step-2"; got != want {
		t.Fatalf("URL() after forward = %q, want %q", got, want)
	}

	if err := s.windowHistoryGo(-1); err != nil {
		t.Fatalf("windowHistoryGo(-1) error = %v", err)
	}
	if got, ok := s.windowHistoryState(); ok || got != "null" {
		t.Fatalf("windowHistoryState() after go(-1) = (%q, %v), want (\"null\", false)", got, ok)
	}
	if got, want := s.URL(), "https://example.test/app"; got != want {
		t.Fatalf("URL() after go(-1) = %q, want %q", got, want)
	}

	if got := s.Registry().Location().Navigations(); len(got) != 6 {
		t.Fatalf("Location().Navigations() = %#v, want 6 records", got)
	}
}

func TestSessionHistoryInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/app",
		HTML: `<main><script>host:historyPushState("step-1", "", "#step-1")</script></main>`,
	})

	if got := s.HistoryLength(); got != 2 {
		t.Fatalf("HistoryLength() = %d, want 2", got)
	}
	if got, ok := s.HistoryState(); !ok || got != "step-1" {
		t.Fatalf("HistoryState() = (%q, %v), want (\"step-1\", true)", got, ok)
	}

	var nilSession *Session
	if got := nilSession.HistoryLength(); got != 0 {
		t.Fatalf("nil HistoryLength() = %d, want 0", got)
	}
	if got, ok := nilSession.HistoryState(); ok || got != "null" {
		t.Fatalf("nil HistoryState() = (%q, %v), want (\"null\", false)", got, ok)
	}
	if got := s.HistoryScrollRestoration(); got != "auto" {
		t.Fatalf("HistoryScrollRestoration() = %q, want %q", got, "auto")
	}
	if got := nilSession.HistoryScrollRestoration(); got != "auto" {
		t.Fatalf("nil HistoryScrollRestoration() = %q, want %q", got, "auto")
	}
}

func TestSessionHistoryEntriesInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/app",
		HTML: `<main><script>host:historyPushState("step-1", "", "#step-1"); host:historyReplaceState("step-2", "", "#step-2")</script></main>`,
	})

	entries := s.HistoryEntries()
	if len(entries) != 2 {
		t.Fatalf("HistoryEntries() = %#v, want 2 entries", entries)
	}
	if entries[0].URL != "https://example.test/app" || entries[0].HasState {
		t.Fatalf("HistoryEntries()[0] = %#v, want initial entry without state", entries[0])
	}
	if entries[1].URL != "https://example.test/app#step-2" || !entries[1].HasState || entries[1].State != "step-2" {
		t.Fatalf("HistoryEntries()[1] = %#v, want current entry with step-2 state", entries[1])
	}

	entries[0].URL = "mutated"
	entries[1].State = "mutated"
	if fresh := s.HistoryEntries(); len(fresh) != 2 || fresh[0].URL != "https://example.test/app" || fresh[1].State != "step-2" {
		t.Fatalf("HistoryEntries() reread = %#v, want original history entries", fresh)
	}

	var nilSession *Session
	if got := nilSession.HistoryEntries(); got != nil {
		t.Fatalf("nil HistoryEntries() = %#v, want nil", got)
	}
}

func TestSessionHistoryIndexInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/app",
		HTML: `<main><script>host:historyPushState("step-1", "", "#step-1")</script></main>`,
	})

	if got := s.HistoryIndex(); got != 1 {
		t.Fatalf("HistoryIndex() = %d, want 1", got)
	}
	if err := s.windowHistoryBack(); err != nil {
		t.Fatalf("windowHistoryBack() error = %v", err)
	}
	if got := s.HistoryIndex(); got != 0 {
		t.Fatalf("HistoryIndex() after back = %d, want 0", got)
	}

	var nilSession *Session
	if got := nilSession.HistoryIndex(); got != 0 {
		t.Fatalf("nil HistoryIndex() = %d, want 0", got)
	}
}

func TestSessionVisitedURLsInspectionHelpers(t *testing.T) {
	s := NewSession(SessionConfig{
		URL: "https://example.test/app",
	})

	visited := s.VisitedURLs()
	if len(visited) != 1 || visited[0] != "https://example.test/app" {
		t.Fatalf("VisitedURLs() = %#v, want current URL snapshot", visited)
	}

	if err := s.windowHistoryPushState("step-1", "", "#step-1"); err != nil {
		t.Fatalf("windowHistoryPushState() error = %v", err)
	}
	visited = s.VisitedURLs()
	if len(visited) != 2 || visited[0] != "https://example.test/app" || visited[1] != "https://example.test/app#step-1" {
		t.Fatalf("VisitedURLs() after pushState = %#v, want history-derived snapshot", visited)
	}

	visited[0] = "mutated"
	visited[1] = "mutated"
	if fresh := s.VisitedURLs(); len(fresh) != 2 || fresh[0] != "https://example.test/app" || fresh[1] != "https://example.test/app#step-1" {
		t.Fatalf("VisitedURLs() reread = %#v, want original visited URLs", fresh)
	}

	var nilSession *Session
	if got := nilSession.VisitedURLs(); got != nil {
		t.Fatalf("nil VisitedURLs() = %#v, want nil", got)
	}
}

func TestSessionNavigationLogInspection(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/start",
		HTML: `<main><script>host:locationAssign("/next"); host:locationReplace("/replace")</script></main>`,
	})

	if got, want := s.NavigationLog(), []string{
		"https://example.test/next",
		"https://example.test/replace",
	}; len(got) != len(want) {
		t.Fatalf("NavigationLog() = %#v, want %#v", got, want)
	} else {
		for i := range want {
			if got[i] != want[i] {
				t.Fatalf("NavigationLog()[%d] = %q, want %q", i, got[i], want[i])
			}
		}
	}

	var nilSession *Session
	if got := nilSession.NavigationLog(); got != nil {
		t.Fatalf("nil NavigationLog() = %#v, want nil", got)
	}
}

func TestSessionHistoryNavigationSyncsTargetState(t *testing.T) {
	s := NewSession(SessionConfig{
		URL:  "https://example.test/app#legacy",
		HTML: `<main id="root"><a name="legacy">legacy</a><div id="space target">space</div><p id="tail">tail</p></main>`,
	})

	store, err := s.ensureDOM()
	if err != nil {
		t.Fatalf("ensureDOM() error = %v", err)
	}

	legacyNodes, err := store.Select("a")
	if err != nil {
		t.Fatalf("Select(a) error = %v", err)
	}
	if len(legacyNodes) != 1 {
		t.Fatalf("Select(a) len = %d, want 1", len(legacyNodes))
	}
	spaceNodes, err := store.Select("div")
	if err != nil {
		t.Fatalf("Select(div) error = %v", err)
	}
	if len(spaceNodes) != 1 {
		t.Fatalf("Select(div) len = %d, want 1", len(spaceNodes))
	}

	if got := store.TargetNodeID(); got != legacyNodes[0] {
		t.Fatalf("TargetNodeID() after bootstrap = %d, want %d", got, legacyNodes[0])
	}

	if err := s.windowHistoryPushState("step-1", "", "https://example.test/app#space%20target"); err != nil {
		t.Fatalf("windowHistoryPushState() error = %v", err)
	}
	if got := store.TargetNodeID(); got != spaceNodes[0] {
		t.Fatalf("TargetNodeID() after pushState = %d, want %d", got, spaceNodes[0])
	}

	if err := s.windowHistoryReplaceState("step-2", "", "https://example.test/app#missing"); err != nil {
		t.Fatalf("windowHistoryReplaceState() error = %v", err)
	}
	if got := store.TargetNodeID(); got != 0 {
		t.Fatalf("TargetNodeID() after replaceState = %d, want 0", got)
	}

	if err := s.windowHistoryBack(); err != nil {
		t.Fatalf("windowHistoryBack() error = %v", err)
	}
	if got := store.TargetNodeID(); got != legacyNodes[0] {
		t.Fatalf("TargetNodeID() after back = %d, want %d", got, legacyNodes[0])
	}

	if err := s.windowHistoryForward(); err != nil {
		t.Fatalf("windowHistoryForward() error = %v", err)
	}
	if got := store.TargetNodeID(); got != 0 {
		t.Fatalf("TargetNodeID() after forward = %d, want 0", got)
	}
}

func TestSessionHistoryScrollRestorationValidation(t *testing.T) {
	s := NewSession(SessionConfig{
		URL: "https://example.test/app",
	})

	if err := s.setWindowHistoryScrollRestoration("manual"); err != nil {
		t.Fatalf("setWindowHistoryScrollRestoration(manual) error = %v", err)
	}
	if got, want := s.windowHistoryScrollRestoration(), "manual"; got != want {
		t.Fatalf("windowHistoryScrollRestoration() = %q, want %q", got, want)
	}

	if err := s.setWindowHistoryScrollRestoration("sideways"); err == nil {
		t.Fatalf("setWindowHistoryScrollRestoration(sideways) error = nil, want validation error")
	}
}

func TestSessionRejectsHistoryArityMismatchesFromInlineScript(t *testing.T) {
	tests := []struct {
		name    string
		html    string
		wantErr string
	}{
		{
			name:    "pushState",
			html:    `<main><script>host:historyPushState("step")</script></main>`,
			wantErr: "history.pushState() expects 2 or 3 arguments",
		},
		{
			name:    "replaceState",
			html:    `<main><script>host:historyReplaceState("step")</script></main>`,
			wantErr: "history.replaceState() expects 2 or 3 arguments",
		},
		{
			name:    "back",
			html:    `<main><script>host:historyBack(1)</script></main>`,
			wantErr: "history.back() expects no arguments",
		},
		{
			name:    "go",
			html:    `<main><script>host:historyGo(1, 2)</script></main>`,
			wantErr: "history.go() accepts at most 1 argument",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			s := NewSession(SessionConfig{
				URL:  "https://example.test/app",
				HTML: tc.html,
			})

			if _, err := s.ensureDOM(); err == nil {
				t.Fatalf("ensureDOM() error = nil, want arity validation error")
			} else if got := err.Error(); !strings.Contains(got, tc.wantErr) {
				t.Fatalf("ensureDOM() error = %q, want substring %q", got, tc.wantErr)
			}
			if got := s.Registry().Location().Navigations(); len(got) != 0 {
				t.Fatalf("Location().Navigations() after rejected history script = %#v, want empty", got)
			}
			if got, want := s.URL(), "https://example.test/app"; got != want {
				t.Fatalf("URL() after rejected history script = %q, want %q", got, want)
			}
		})
	}
}
