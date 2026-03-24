package dom

import "strings"

func (s *Store) SyncVisitedURLs(urls []string) {
	if s == nil {
		return
	}
	if len(urls) == 0 {
		s.visitedURLs = map[string]struct{}{}
		return
	}

	visited := make(map[string]struct{}, len(urls))
	for _, candidate := range urls {
		trimmed := strings.TrimSpace(candidate)
		if trimmed == "" {
			continue
		}
		visited[trimmed] = struct{}{}
	}
	s.visitedURLs = visited
}

func (s *Store) HasVisitedURL(url string) bool {
	if s == nil {
		return false
	}
	_, ok := s.visitedURLs[strings.TrimSpace(url)]
	return ok
}
