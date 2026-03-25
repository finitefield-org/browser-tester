package runtime

import (
	"fmt"
	"strings"
	"testing"
)

type historyModelEntry struct {
	url   string
	state *string
}

type historyModel struct {
	entries []historyModelEntry
	index   int
}

func newHistoryModel(url string) *historyModel {
	return &historyModel{
		entries: []historyModelEntry{{url: url}},
		index:   0,
	}
}

func (m *historyModel) current() historyModelEntry {
	return m.entries[m.index]
}

func (m *historyModel) push(state, url string) {
	resolved := modelHistoryURL(m.current().url, url)
	if len(m.entries) == 0 {
		seeded := state
		m.entries = []historyModelEntry{{url: resolved, state: &seeded}}
		m.index = 0
		return
	}

	next := m.index + 1
	if next < len(m.entries) {
		m.entries = m.entries[:next]
	}
	seeded := state
	m.entries = append(m.entries, historyModelEntry{url: resolved, state: &seeded})
	m.index = len(m.entries) - 1
}

func (m *historyModel) replace(state, url string) {
	resolved := modelHistoryURL(m.current().url, url)
	seeded := state
	if len(m.entries) == 0 {
		m.entries = []historyModelEntry{{url: resolved, state: &seeded}}
		m.index = 0
		return
	}
	m.entries[m.index] = historyModelEntry{url: resolved, state: &seeded}
}

func (m *historyModel) move(delta int64) bool {
	if len(m.entries) == 0 || delta == 0 {
		return false
	}
	current := int64(m.index)
	target := current + delta
	if target < 0 {
		target = 0
	}
	maxIndex := int64(len(m.entries) - 1)
	if target > maxIndex {
		target = maxIndex
	}
	if target == current {
		return false
	}
	m.index = int(target)
	return true
}

func (m *historyModel) reload() bool {
	return len(m.entries) > 0
}

func modelHistoryURL(baseURL, href string) string {
	trimmed := strings.TrimSpace(href)
	if trimmed == "" {
		return baseURL
	}
	return resolveHyperlinkURL(baseURL, trimmed)
}

func modelVisitedURLs(entries []historyModelEntry, currentURL string) []string {
	seen := make(map[string]struct{}, len(entries)+1)
	out := make([]string, 0, len(entries)+1)
	add := func(candidate string) {
		trimmed := strings.TrimSpace(candidate)
		if trimmed == "" {
			return
		}
		if _, ok := seen[trimmed]; ok {
			return
		}
		seen[trimmed] = struct{}{}
		out = append(out, trimmed)
	}
	for i := range entries {
		add(entries[i].url)
	}
	add(currentURL)
	return out
}

func historyCandidateURL(step int, b byte) string {
	switch b % 4 {
	case 0:
		return ""
	case 1:
		return fmt.Sprintf("/history/%02x/%d", b, step)
	case 2:
		return fmt.Sprintf("?q=%02x", b)
	default:
		return fmt.Sprintf("#fragment-%02x-%d", b, step)
	}
}

func FuzzSessionHistoryNavigationSequence(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte{0, 1, 2, 3, 4, 5},
		[]byte("history"),
		[]byte{255, 254, 253, 252},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		s := NewSession(SessionConfig{URL: "https://example.test/start"})
		model := newHistoryModel(s.URL())
		navCount := 0

		for i, b := range data {
			changed := false
			switch b % 6 {
			case 0:
				state := fmt.Sprintf("push-%d-%02x", i, b)
				if err := s.windowHistoryPushState(state, "", historyCandidateURL(i, b)); err != nil {
					continue
				}
				model.push(state, historyCandidateURL(i, b))
				changed = true
			case 1:
				state := fmt.Sprintf("replace-%d-%02x", i, b)
				if err := s.windowHistoryReplaceState(state, "", historyCandidateURL(i, b)); err != nil {
					continue
				}
				model.replace(state, historyCandidateURL(i, b))
				changed = true
			case 2:
				if err := s.windowHistoryBack(); err != nil {
					continue
				}
				changed = model.move(-1)
			case 3:
				if err := s.windowHistoryForward(); err != nil {
					continue
				}
				changed = model.move(1)
			case 4:
				delta := int64(int(b%7)) - 3
				if err := s.windowHistoryGo(delta); err != nil {
					continue
				}
				changed = model.move(delta)
			case 5:
				if err := s.reloadNavigation(); err != nil {
					continue
				}
				changed = model.reload()
			}

			if got, want := s.windowHistoryLength(), len(model.entries); got != want {
				t.Fatalf("windowHistoryLength() after op %d = %d, want %d", i, got, want)
			}
			if got, want := s.URL(), model.current().url; got != want {
				t.Fatalf("URL() after op %d = %q, want %q", i, got, want)
			}
			if got, ok := s.windowHistoryState(); (model.current().state == nil) != (!ok) || (model.current().state != nil && got != *model.current().state) {
				wantState := "null"
				wantOK := false
				if model.current().state != nil {
					wantState = *model.current().state
					wantOK = true
				}
				t.Fatalf("windowHistoryState() after op %d = (%q, %v), want (%q, %v)", i, got, ok, wantState, wantOK)
			}

			if changed {
				navCount++
			}
			logs := s.NavigationLog()
			if len(logs) != navCount {
				t.Fatalf("NavigationLog() after op %d = %#v, want %d entries", i, logs, navCount)
			}
			if len(logs) > 0 && logs[len(logs)-1] != model.current().url {
				t.Fatalf("NavigationLog() last entry after op %d = %q, want %q", i, logs[len(logs)-1], model.current().url)
			}

			visited := s.VisitedURLs()
			wantVisited := modelVisitedURLs(model.entries, model.current().url)
			if len(visited) != len(wantVisited) {
				t.Fatalf("VisitedURLs() after op %d = %#v, want %#v", i, visited, wantVisited)
			}
			for idx := range wantVisited {
				if visited[idx] != wantVisited[idx] {
					t.Fatalf("VisitedURLs()[%d] after op %d = %q, want %q", idx, i, visited[idx], wantVisited[idx])
				}
			}
			if len(visited) > 0 {
				visited[0] = "mutated"
				if fresh := s.VisitedURLs(); len(fresh) != len(wantVisited) || fresh[0] != wantVisited[0] {
					t.Fatalf("VisitedURLs() reread after op %d = %#v, want %#v", i, fresh, wantVisited)
				}
			}
		}
	})
}
