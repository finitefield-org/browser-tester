package runtime

import (
	"net/url"
	"strings"
	"testing"
)

func FuzzResolveLocationPropertyURL(f *testing.F) {
	seeds := []struct {
		baseURL  string
		property string
		value    string
	}{
		{"https://example.test/start", "href", "/next"},
		{"https://example.test/start", "hash", "#step-1"},
		{"https://example.test/start", "pathname", "detail"},
		{"https://example.test/start?mode=full#top", "search", "?page=2"},
		{"https://example.test/start", "protocol", "https:"},
		{"https://example.test/start", "host", "example.test:443"},
		{"https://example.test:8443/start", "hostname", "example.test"},
		{"https://example.test:8443/start", "port", "9443"},
		{"https://example.test/start", "username", "alice"},
		{"https://example.test/start", "password", "secret"},
		{"https://example.test/start", "origin", "https://example.test/other"},
	}
	for _, seed := range seeds {
		f.Add(seed.baseURL, seed.property, seed.value)
	}

	f.Fuzz(func(t *testing.T, baseURL, property, value string) {
		resolved, err := resolveLocationPropertyURL(baseURL, property, value)
		normalized := strings.ToLower(strings.TrimSpace(property))
		if normalized == "origin" {
			if err == nil {
				t.Fatalf("resolveLocationPropertyURL(%q, %q, %q) error = nil, want read-only failure", baseURL, property, value)
			}
			return
		}
		if err != nil {
			return
		}
		if strings.TrimSpace(resolved) == "" {
			t.Fatalf("resolveLocationPropertyURL(%q, %q, %q) returned an empty URL", baseURL, property, value)
		}
		if _, err := url.Parse(resolved); err != nil {
			t.Fatalf("resolveLocationPropertyURL(%q, %q, %q) returned unparsable URL %q: %v", baseURL, property, value, resolved, err)
		}
	})
}

func FuzzSessionSetLocationProperty(f *testing.F) {
	seeds := []struct {
		baseURL  string
		property string
		value    string
	}{
		{"https://example.test/start", "href", "/next"},
		{"https://example.test/start", "hash", "#step-1"},
		{"https://example.test/start", "pathname", "detail"},
		{"https://example.test/start", "search", "?page=2"},
		{"https://example.test:8443/start", "hostname", "example.test"},
		{"https://example.test/start", "username", "alice"},
	}
	for _, seed := range seeds {
		f.Add(seed.baseURL, seed.property, seed.value)
	}

	f.Fuzz(func(t *testing.T, baseURL, property, value string) {
		s := NewSession(SessionConfig{URL: baseURL})
		beforeURL := s.URL()

		expected, expectedErr := resolveLocationPropertyURL(beforeURL, property, value)
		err := s.SetLocationProperty(property, value)
		if expectedErr != nil {
			if err == nil {
				t.Fatalf("SetLocationProperty(%q, %q, %q) error = nil, want %v", beforeURL, property, value, expectedErr)
			}
			if got := s.URL(); got != beforeURL {
				t.Fatalf("SetLocationProperty(%q, %q, %q) mutated URL to %q, want %q on error", beforeURL, property, value, got, beforeURL)
			}
			return
		}
		if err != nil {
			t.Fatalf("SetLocationProperty(%q, %q, %q) error = %v, want nil", beforeURL, property, value, err)
		}
		if got, want := s.URL(), expected; got != want {
			t.Fatalf("SetLocationProperty(%q, %q, %q) URL() = %q, want %q", beforeURL, property, value, got, want)
		}
		if got, want := s.HistoryLength(), 2; got != want {
			t.Fatalf("HistoryLength() after SetLocationProperty = %d, want %d", got, want)
		}
		logs := s.NavigationLog()
		if len(logs) != 1 {
			t.Fatalf("NavigationLog() after SetLocationProperty = %#v, want 1 entry", logs)
		}
		if got, want := logs[0], expected; got != want {
			t.Fatalf("NavigationLog()[0] = %q, want %q", got, want)
		}
	})
}

func FuzzSessionReadLocationProperties(f *testing.F) {
	seeds := []string{
		"https://example.test/",
		"https://example.test:8443/path/name?mode=full#step-1",
		"/relative/path?x=1#frag",
		"file:///tmp/demo.txt",
		"mailto:hello@example.test",
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, baseURL string) {
		s := NewSession(SessionConfig{URL: baseURL})
		currentURL := s.URL()

		if got, err := s.LocationHref(); err != nil {
			t.Fatalf("LocationHref() error = %v", err)
		} else if got != currentURL {
			t.Fatalf("LocationHref() = %q, want %q", got, currentURL)
		}

		parsed, err := url.Parse(currentURL)
		if err != nil {
			for _, tc := range []struct {
				name string
				get  func() (string, error)
			}{
				{name: "origin", get: s.LocationOrigin},
				{name: "protocol", get: s.LocationProtocol},
				{name: "host", get: s.LocationHost},
				{name: "hostname", get: s.LocationHostname},
				{name: "port", get: s.LocationPort},
				{name: "pathname", get: s.LocationPathname},
				{name: "search", get: s.LocationSearch},
				{name: "hash", get: s.LocationHash},
			} {
				if got, err := tc.get(); err == nil {
					t.Fatalf("%s() = %q, want parse error for current URL %q", tc.name, got, currentURL)
				}
			}
			return
		}

		wantOrigin := "null"
		if parsed.Scheme != "" && parsed.Host != "" {
			wantOrigin = parsed.Scheme + "://" + parsed.Host
		}
		if got, err := s.LocationOrigin(); err != nil {
			t.Fatalf("LocationOrigin() error = %v", err)
		} else if got != wantOrigin {
			t.Fatalf("LocationOrigin() = %q, want %q", got, wantOrigin)
		}

		wantProtocol := ""
		if parsed.Scheme != "" {
			wantProtocol = parsed.Scheme + ":"
		}
		if got, err := s.LocationProtocol(); err != nil {
			t.Fatalf("LocationProtocol() error = %v", err)
		} else if got != wantProtocol {
			t.Fatalf("LocationProtocol() = %q, want %q", got, wantProtocol)
		}

		if got, err := s.LocationHost(); err != nil {
			t.Fatalf("LocationHost() error = %v", err)
		} else if got != parsed.Host {
			t.Fatalf("LocationHost() = %q, want %q", got, parsed.Host)
		}

		if got, err := s.LocationHostname(); err != nil {
			t.Fatalf("LocationHostname() error = %v", err)
		} else if got != parsed.Hostname() {
			t.Fatalf("LocationHostname() = %q, want %q", got, parsed.Hostname())
		}

		if got, err := s.LocationPort(); err != nil {
			t.Fatalf("LocationPort() error = %v", err)
		} else if got != parsed.Port() {
			t.Fatalf("LocationPort() = %q, want %q", got, parsed.Port())
		}

		wantPath := parsed.EscapedPath()
		if wantPath == "" {
			wantPath = "/"
		}
		if got, err := s.LocationPathname(); err != nil {
			t.Fatalf("LocationPathname() error = %v", err)
		} else if got != wantPath {
			t.Fatalf("LocationPathname() = %q, want %q", got, wantPath)
		}

		wantSearch := ""
		if parsed.RawQuery != "" {
			wantSearch = "?" + parsed.RawQuery
		} else if parsed.ForceQuery {
			wantSearch = "?"
		}
		if got, err := s.LocationSearch(); err != nil {
			t.Fatalf("LocationSearch() error = %v", err)
		} else if got != wantSearch {
			t.Fatalf("LocationSearch() = %q, want %q", got, wantSearch)
		}

		wantHash := ""
		if fragment := parsed.EscapedFragment(); fragment != "" {
			wantHash = "#" + fragment
		}
		if got, err := s.LocationHash(); err != nil {
			t.Fatalf("LocationHash() error = %v", err)
		} else if got != wantHash {
			t.Fatalf("LocationHash() = %q, want %q", got, wantHash)
		}
	})
}
