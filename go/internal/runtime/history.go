package runtime

import (
	"fmt"
	"strings"
)

type historyEntry struct {
	url   string
	state *string
}

type HistoryEntry struct {
	URL      string
	State    string
	HasState bool
}

func (s *Session) windowHistoryLength() int {
	if s == nil {
		return 0
	}
	s.ensureHistoryInitialized()
	return len(s.historyEntries)
}

func (s *Session) windowHistoryState() (string, bool) {
	if s == nil {
		return "null", false
	}
	s.ensureHistoryInitialized()
	if len(s.historyEntries) == 0 || s.historyIndex < 0 || s.historyIndex >= len(s.historyEntries) {
		return "null", false
	}
	entry := s.historyEntries[s.historyIndex]
	if entry.state == nil {
		return "null", false
	}
	return *entry.state, true
}

func (s *Session) windowHistoryScrollRestoration() string {
	if s == nil {
		return "auto"
	}
	s.ensureHistoryInitialized()
	if s.historyScrollRestoration == "" {
		return "auto"
	}
	return s.historyScrollRestoration
}

func (s *Session) HistoryEntries() []HistoryEntry {
	if s == nil {
		return nil
	}
	if _, err := s.ensureDOM(); err != nil {
		return nil
	}
	if len(s.historyEntries) == 0 {
		return nil
	}
	out := make([]HistoryEntry, len(s.historyEntries))
	for i := range s.historyEntries {
		out[i].URL = s.historyEntries[i].url
		if s.historyEntries[i].state != nil {
			out[i].State = *s.historyEntries[i].state
			out[i].HasState = true
		}
	}
	return out
}

func (s *Session) HistoryIndex() int {
	if s == nil {
		return 0
	}
	if _, err := s.ensureDOM(); err != nil {
		return 0
	}
	s.ensureHistoryInitialized()
	if s.historyIndex < 0 {
		return 0
	}
	if s.historyIndex >= len(s.historyEntries) {
		if len(s.historyEntries) == 0 {
			return 0
		}
		return len(s.historyEntries) - 1
	}
	return s.historyIndex
}

func (s *Session) VisitedURLs() []string {
	if s == nil {
		return nil
	}
	if _, err := s.ensureDOM(); err != nil {
		return nil
	}
	urls := s.visitedHistoryURLs(s.URL())
	if len(urls) == 0 {
		return nil
	}
	out := make([]string, len(urls))
	copy(out, urls)
	return out
}

func (s *Session) setWindowHistoryScrollRestoration(value string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	trimmed := strings.ToLower(strings.TrimSpace(value))
	switch trimmed {
	case "auto", "manual":
		s.ensureHistoryInitialized()
		s.historyScrollRestoration = trimmed
		return nil
	default:
		return fmt.Errorf("unsupported history scroll restoration value: %s", value)
	}
}

func (s *Session) windowHistoryPushState(state, title, url string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	_ = title
	s.ensureHistoryInitialized()
	resolved := s.resolveHistoryURL(url)
	nextState := cloneHistoryState(state)

	if len(s.historyEntries) == 0 {
		s.historyEntries = []historyEntry{{url: resolved, state: nextState}}
		s.historyIndex = 0
	} else {
		nextIndex := s.historyIndex + 1
		if nextIndex < len(s.historyEntries) {
			s.historyEntries = s.historyEntries[:nextIndex]
		}
		s.historyEntries = append(s.historyEntries, historyEntry{
			url:   resolved,
			state: nextState,
		})
		s.historyIndex = len(s.historyEntries) - 1
	}

	s.applyHistoryURLUpdate(resolved)
	return nil
}

func (s *Session) windowHistoryReplaceState(state, title, url string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	_ = title
	s.ensureHistoryInitialized()
	resolved := s.resolveHistoryURL(url)
	nextState := cloneHistoryState(state)

	if len(s.historyEntries) == 0 {
		s.historyEntries = []historyEntry{{url: resolved, state: nextState}}
		s.historyIndex = 0
	} else {
		s.historyEntries[s.historyIndex] = historyEntry{
			url:   resolved,
			state: nextState,
		}
	}

	s.applyHistoryURLUpdate(resolved)
	return nil
}

func (s *Session) windowHistoryBack() error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	return s.windowHistoryGo(-1)
}

func (s *Session) windowHistoryForward() error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	return s.windowHistoryGo(1)
}

func (s *Session) windowHistoryGo(delta int64) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	s.ensureHistoryInitialized()
	if len(s.historyEntries) == 0 || delta == 0 {
		return nil
	}

	current := int64(s.historyIndex)
	maxIndex := int64(len(s.historyEntries) - 1)
	target := current + delta
	if target < 0 {
		target = 0
	}
	if target > maxIndex {
		target = maxIndex
	}
	if target == current {
		return nil
	}

	s.historyIndex = int(target)
	url := s.historyEntries[s.historyIndex].url
	s.applyHistoryNavigation(url)
	return nil
}

func (s *Session) pushHistoryNavigation(url string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	s.ensureHistoryInitialized()
	resolved := s.resolveHistoryURL(url)

	if len(s.historyEntries) == 0 {
		s.historyEntries = []historyEntry{{url: resolved}}
		s.historyIndex = 0
	} else {
		nextIndex := s.historyIndex + 1
		if nextIndex < len(s.historyEntries) {
			s.historyEntries = s.historyEntries[:nextIndex]
		}
		s.historyEntries = append(s.historyEntries, historyEntry{
			url: resolved,
		})
		s.historyIndex = len(s.historyEntries) - 1
	}

	s.applyHistoryNavigation(resolved)
	return nil
}

func (s *Session) replaceNavigation(url string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	s.ensureHistoryInitialized()
	resolved := s.resolveHistoryURL(url)

	if len(s.historyEntries) == 0 {
		s.historyEntries = []historyEntry{{url: resolved}}
		s.historyIndex = 0
	} else {
		s.historyEntries[s.historyIndex] = historyEntry{
			url: resolved,
		}
	}

	s.applyHistoryNavigation(resolved)
	return nil
}

func (s *Session) reloadNavigation() error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	s.ensureHistoryInitialized()
	current := strings.TrimSpace(s.currentHistoryURL())
	if current == "" {
		return fmt.Errorf("reload() requires a current URL")
	}
	s.applyHistoryNavigation(current)
	return nil
}

func (s *Session) applyHistoryNavigation(url string) {
	if s == nil {
		return
	}
	s.applyHistoryURLUpdate(url)
	s.scrollX = 0
	s.scrollY = 0
}

func (s *Session) applyHistoryURLUpdate(url string) {
	if s == nil {
		return
	}
	location := s.Registry().Location()
	if location != nil {
		location.RecordNavigation(url)
	}
	s.syncDocumentState(url)
}

func (s *Session) visitedHistoryURLs(currentURL string) []string {
	if s == nil {
		return nil
	}

	seen := make(map[string]struct{}, len(s.historyEntries)+1)
	urls := make([]string, 0, len(s.historyEntries)+1)
	add := func(candidate string) {
		trimmed := strings.TrimSpace(candidate)
		if trimmed == "" {
			return
		}
		if _, ok := seen[trimmed]; ok {
			return
		}
		seen[trimmed] = struct{}{}
		urls = append(urls, trimmed)
	}

	for _, entry := range s.historyEntries {
		add(entry.url)
	}
	add(currentURL)
	return urls
}

func (s *Session) resolveHistoryURL(url string) string {
	if s == nil {
		return ""
	}
	normalized := strings.TrimSpace(url)
	if normalized == "" {
		return s.currentHistoryURL()
	}
	return resolveHyperlinkURL(s.currentHistoryURL(), normalized)
}

func (s *Session) currentHistoryURL() string {
	if s == nil {
		return ""
	}
	if len(s.historyEntries) > 0 && s.historyIndex >= 0 && s.historyIndex < len(s.historyEntries) {
		return s.historyEntries[s.historyIndex].url
	}
	current := strings.TrimSpace(s.URL())
	if current != "" {
		return current
	}
	return DefaultSessionConfig().URL
}

func (s *Session) ensureHistoryInitialized() {
	if s == nil {
		return
	}
	if len(s.historyEntries) > 0 {
		if s.historyIndex < 0 || s.historyIndex >= len(s.historyEntries) {
			s.historyIndex = len(s.historyEntries) - 1
		}
		if s.historyScrollRestoration == "" {
			s.historyScrollRestoration = "auto"
		}
		return
	}

	current := strings.TrimSpace(s.currentHistoryURL())
	if current == "" {
		current = DefaultSessionConfig().URL
	}
	s.historyEntries = []historyEntry{
		{
			url: current,
		},
	}
	s.historyIndex = 0
	if location := s.Registry().Location(); location != nil {
		location.SetCurrentURL(current)
	}
	if s.historyScrollRestoration == "" {
		s.historyScrollRestoration = "auto"
	}
}

func cloneHistoryState(state string) *string {
	seeded := state
	return &seeded
}

func cloneHistoryEntries(entries []historyEntry) []historyEntry {
	if len(entries) == 0 {
		return nil
	}
	out := make([]historyEntry, len(entries))
	for i := range entries {
		out[i].url = entries[i].url
		if entries[i].state != nil {
			seeded := *entries[i].state
			out[i].state = &seeded
		}
	}
	return out
}
