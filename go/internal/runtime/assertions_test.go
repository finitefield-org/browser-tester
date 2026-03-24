package runtime

import (
	"errors"
	"strings"
	"testing"
)

func TestSessionAssertionsSucceed(t *testing.T) {
	cfg := DefaultSessionConfig()
	cfg.HTML = `<main><div id="out">hello</div><input id="name" value="Ada"><input id="agree" type="checkbox" checked><input id="upload" type="file"><select id="mode"><option value="a">A</option><option value="b" selected>B</option></select></main>`
	s := NewSession(cfg)

	if err := s.AssertExists("#out"); err != nil {
		t.Fatalf("AssertExists(#out) error = %v", err)
	}
	if err := s.AssertText("#out", "hello"); err != nil {
		t.Fatalf("AssertText(#out) error = %v", err)
	}
	if err := s.AssertValue("#name", "Ada"); err != nil {
		t.Fatalf("AssertValue(#name) error = %v", err)
	}
	if err := s.AssertChecked("#agree", true); err != nil {
		t.Fatalf("AssertChecked(#agree) error = %v", err)
	}
	if err := s.AssertValue("#mode", "b"); err != nil {
		t.Fatalf("AssertValue(#mode) error = %v", err)
	}

	if err := s.SetFiles("#upload", []string{"first.txt", "second.txt"}); err != nil {
		t.Fatalf("SetFiles() error = %v", err)
	}
	if err := s.AssertValue("#upload", "first.txt, second.txt"); err != nil {
		t.Fatalf("AssertValue(#upload) after SetFiles error = %v", err)
	}

	if err := s.SetFiles("#upload", []string{"report.csv"}); err != nil {
		t.Fatalf("SetFiles() #2 error = %v", err)
	}
	if err := s.AssertValue("#upload", "report.csv"); err != nil {
		t.Fatalf("AssertValue(#upload) after SetFiles #2 error = %v", err)
	}
}

func TestSessionAssertionsReturnSelectorErrorForInvalidSelectors(t *testing.T) {
	cfg := DefaultSessionConfig()
	cfg.HTML = `<main><div id="out"></div></main>`
	s := NewSession(cfg)

	err := s.AssertExists("div + p")
	if err == nil {
		t.Fatalf("AssertExists(div + p) error = nil, want SelectorError")
	}
	var sel SelectorError
	if !errors.As(err, &sel) {
		t.Fatalf("AssertExists(div + p) error = %T, want SelectorError", err)
	}
}

func TestSessionAssertionsIncludeDOMDumpOnFailure(t *testing.T) {
	cfg := DefaultSessionConfig()
	cfg.HTML = `<main><div id="out">hello</div><input id="name" value="Ada"></main>`
	s := NewSession(cfg)

	err := s.AssertExists("#missing")
	if err == nil {
		t.Fatalf("AssertExists(#missing) error = nil, want AssertionError")
	}
	var as AssertionError
	if !errors.As(err, &as) {
		t.Fatalf("AssertExists(#missing) error = %T, want AssertionError", err)
	}
	if !strings.Contains(err.Error(), "DOM:\n") || !strings.Contains(err.Error(), `<main>`) {
		t.Fatalf("AssertExists(#missing) error = %q, want DOM dump", err.Error())
	}

	err = s.AssertText("#out", "nope")
	if err == nil {
		t.Fatalf("AssertText(#out) error = nil, want AssertionError")
	}
	if !errors.As(err, &as) {
		t.Fatalf("AssertText(#out) error = %T, want AssertionError", err)
	}
	if !strings.Contains(err.Error(), "DOM:\n") || !strings.Contains(err.Error(), `<div id="out">hello</div>`) {
		t.Fatalf("AssertText(#out) error = %q, want DOM dump including #out", err.Error())
	}

	err = s.AssertChecked("#name", true)
	if err == nil {
		t.Fatalf("AssertChecked(#name) error = nil, want AssertionError")
	}
	if !errors.As(err, &as) {
		t.Fatalf("AssertChecked(#name) error = %T, want AssertionError", err)
	}
	if !strings.Contains(err.Error(), "checkable control") {
		t.Fatalf("AssertChecked(#name) error = %q, want non-checkable message", err.Error())
	}
}
