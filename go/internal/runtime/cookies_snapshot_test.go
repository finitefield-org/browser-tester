package runtime

import "testing"

func TestSessionCookieJarSnapshot(t *testing.T) {
	s := NewSession(DefaultSessionConfig())
	if err := s.setDocumentCookie("theme=dark"); err != nil {
		t.Fatalf("setDocumentCookie(theme=dark) error = %v", err)
	}
	if err := s.setDocumentCookie("lang=en; Path=/"); err != nil {
		t.Fatalf("setDocumentCookie(lang=en; Path=/) error = %v", err)
	}

	jar := s.CookieJar()
	if len(jar) != 2 || jar["theme"] != "dark" || jar["lang"] != "en" {
		t.Fatalf("CookieJar() = %#v, want theme/lang snapshot", jar)
	}

	jar["theme"] = "mutated"
	fresh := s.CookieJar()
	if fresh["theme"] != "dark" {
		t.Fatalf("CookieJar() reread = %#v, want original cookie jar", fresh)
	}

	var nilSession *Session
	if got := nilSession.CookieJar(); got != nil {
		t.Fatalf("nil CookieJar() = %#v, want nil", got)
	}
}
