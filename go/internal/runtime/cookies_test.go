package runtime

import "testing"

func TestSessionDocumentCookieGetterAndSetter(t *testing.T) {
	s := NewSession(DefaultSessionConfig())

	if got := s.documentCookie(); got != "" {
		t.Fatalf("documentCookie() = %q, want empty string", got)
	}
	if got := s.DocumentCookie(); got != "" {
		t.Fatalf("DocumentCookie() = %q, want empty string", got)
	}

	if err := s.setDocumentCookie("theme=dark"); err != nil {
		t.Fatalf("setDocumentCookie(theme=dark) error = %v", err)
	}
	if err := s.setDocumentCookie("lang=en; Path=/"); err != nil {
		t.Fatalf("setDocumentCookie(lang=en; Path=/) error = %v", err)
	}
	if err := s.setDocumentCookie("theme=light"); err != nil {
		t.Fatalf("setDocumentCookie(theme=light) error = %v", err)
	}

	if got, want := s.documentCookie(), "lang=en; theme=light"; got != want {
		t.Fatalf("documentCookie() = %q, want %q", got, want)
	}
	if got, want := s.DocumentCookie(), "lang=en; theme=light"; got != want {
		t.Fatalf("DocumentCookie() = %q, want %q", got, want)
	}
}

func TestSessionRejectsMalformedDocumentCookieAssignments(t *testing.T) {
	tests := []struct {
		name    string
		value   string
		wantErr string
	}{
		{
			name:    "empty",
			value:   "   ",
			wantErr: "document.cookie requires a non-empty cookie string",
		},
		{
			name:    "pair",
			value:   "badcookie",
			wantErr: "document.cookie requires `name=value`",
		},
		{
			name:    "name",
			value:   " =value",
			wantErr: "document.cookie requires a non-empty cookie name",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			s := NewSession(DefaultSessionConfig())

			if err := s.setDocumentCookie(tc.value); err == nil {
				t.Fatalf("setDocumentCookie(%q) error = nil, want validation error", tc.value)
			} else if got := err.Error(); got != tc.wantErr {
				t.Fatalf("setDocumentCookie(%q) error = %q, want %q", tc.value, got, tc.wantErr)
			}
		})
	}
}

func TestSessionNavigatorCookieEnabled(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	if !s.navigatorCookieEnabled() {
		t.Fatalf("navigatorCookieEnabled() = false, want true")
	}
}
