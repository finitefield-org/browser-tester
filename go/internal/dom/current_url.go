package dom

import (
	"net/url"
	"strings"
)

func (s *Store) SyncCurrentURL(currentURL string) {
	if s == nil {
		return
	}
	s.currentURL = strings.TrimSpace(currentURL)
}

func resolveDocumentLinkURL(baseURL, href string) string {
	baseURL = strings.TrimSpace(baseURL)
	href = strings.TrimSpace(href)
	if baseURL == "" || href == "" {
		return ""
	}

	base, err := url.Parse(baseURL)
	if err != nil {
		return ""
	}
	ref, err := url.Parse(href)
	if err != nil {
		return ""
	}
	return base.ResolveReference(ref).String()
}

func sameDocumentURL(leftURL, rightURL string) bool {
	leftURL = strings.TrimSpace(leftURL)
	rightURL = strings.TrimSpace(rightURL)
	if leftURL == "" || rightURL == "" {
		return false
	}

	left, err := url.Parse(leftURL)
	if err != nil {
		return false
	}
	right, err := url.Parse(rightURL)
	if err != nil {
		return false
	}
	left.Fragment = ""
	right.Fragment = ""
	return left.String() == right.String()
}
