package runtime

import (
	"fmt"
	"sort"
	"strings"
	"testing"
)

func FuzzSessionDocumentCookieSequence(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte{0, 1, 2, 3},
		[]byte("cookie"),
		[]byte{255, 254, 253, 252},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		s := NewSession(DefaultSessionConfig())
		model := map[string]string{}

		for i, b := range data {
			name := fmt.Sprintf("k%02x_%d", b, i)
			value := fmt.Sprintf("v%02x_%d", b, i)
			op := b % 4

			var raw string
			switch op {
			case 0:
				raw = name + "=" + value
			case 1:
				raw = name + "=" + value + "; Path=/; Secure"
			case 2:
				raw = " =" + value
			default:
				raw = fmt.Sprintf("bad%02x_%d", b, i)
			}

			err := s.setDocumentCookie(raw)
			if op == 2 || op == 3 {
				if err == nil {
					t.Fatalf("setDocumentCookie(%q) error = nil, want validation failure", raw)
				}
				want := cookieModelString(model)
				if got := s.documentCookie(); got != want {
					t.Fatalf("documentCookie() after rejected op %d = %q, want %q", i, got, want)
				}
				if got := s.DocumentCookie(); got != want {
					t.Fatalf("DocumentCookie() after rejected op %d = %q, want %q", i, got, want)
				}
				continue
			}
			if err != nil {
				t.Fatalf("setDocumentCookie(%q) error = %v", raw, err)
			}

			model[name] = value
			want := cookieModelString(model)
			if got := s.documentCookie(); got != want {
				t.Fatalf("documentCookie() after op %d = %q, want %q", i, got, want)
			}
			if got := s.DocumentCookie(); got != want {
				t.Fatalf("DocumentCookie() after op %d = %q, want %q", i, got, want)
			}
			if !s.navigatorCookieEnabled() {
				t.Fatalf("navigatorCookieEnabled() = false, want true")
			}
		}
	})
}

func cookieModelString(entries map[string]string) string {
	if len(entries) == 0 {
		return ""
	}
	keys := make([]string, 0, len(entries))
	for key := range entries {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	parts := make([]string, 0, len(keys))
	for _, key := range keys {
		parts = append(parts, key+"="+entries[key])
	}
	return strings.Join(parts, "; ")
}
