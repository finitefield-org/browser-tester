package runtime

import (
	"fmt"
	"sort"
	"strings"
	"unicode"
)

func (s *Session) documentCookie() string {
	if s == nil || len(s.cookieJar) == 0 {
		return ""
	}
	keys := make([]string, 0, len(s.cookieJar))
	for name := range s.cookieJar {
		keys = append(keys, name)
	}
	sort.Strings(keys)

	parts := make([]string, 0, len(keys))
	for _, name := range keys {
		parts = append(parts, name+"="+s.cookieJar[name])
	}
	return strings.Join(parts, "; ")
}

func (s *Session) DocumentCookie() string {
	if s == nil {
		return ""
	}
	if _, err := s.ensureDOM(); err != nil {
		return ""
	}
	return s.documentCookie()
}

func (s *Session) CookieJar() map[string]string {
	if s == nil {
		return nil
	}
	if _, err := s.ensureDOM(); err != nil {
		return nil
	}
	return cloneStringMap(s.cookieJar)
}

func (s *Session) setDocumentCookie(value string) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return fmt.Errorf("document.cookie requires a non-empty cookie string")
	}

	pair := trimmed
	if head, _, ok := strings.Cut(trimmed, ";"); ok {
		pair = head
	}
	name, cookieValue, ok := strings.Cut(pair, "=")
	if !ok {
		return fmt.Errorf("document.cookie requires `name=value`")
	}

	name = strings.TrimSpace(name)
	if name == "" {
		return fmt.Errorf("document.cookie requires a non-empty cookie name")
	}

	if s.cookieJar == nil {
		s.cookieJar = map[string]string{}
	}
	s.cookieJar[name] = strings.TrimLeftFunc(cookieValue, unicode.IsSpace)
	return nil
}

func (s *Session) navigatorCookieEnabled() bool {
	return s != nil
}
