package browsertester

import (
	"strings"
	"testing"
)

func TestHarnessAssertionHelpers(t *testing.T) {
	harness, err := FromHTML(`<main><input id="name" value="Ada"><input id="flag" type="checkbox" checked><select id="mode"><option value="a">A</option><option value="b" selected>B</option></select><div id="out">Hello</div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertText("#out", "Hello"); err != nil {
		t.Fatalf("AssertText() error = %v", err)
	}
	if err := harness.AssertValue("#name", "Ada"); err != nil {
		t.Fatalf("AssertValue(#name) error = %v", err)
	}
	if err := harness.AssertValue("#mode", "b"); err != nil {
		t.Fatalf("AssertValue(#mode) error = %v", err)
	}
	if err := harness.AssertChecked("#flag", true); err != nil {
		t.Fatalf("AssertChecked() error = %v", err)
	}
	if err := harness.AssertExists("main > #out"); err != nil {
		t.Fatalf("AssertExists() error = %v", err)
	}
}

func TestHarnessAssertionHelpersClassifyFailures(t *testing.T) {
	harness, err := FromHTML(`<main><input id="flag" type="checkbox"><div id="out">Hello</div></main>`)
	if err != nil {
		t.Fatalf("FromHTML() error = %v", err)
	}

	if err := harness.AssertExists("main + div"); err == nil {
		t.Fatalf("AssertExists(unsupported selector) error = nil, want selector error")
	} else {
		got, ok := err.(Error)
		if !ok {
			t.Fatalf("AssertExists(unsupported selector) type = %T, want browsertester.Error", err)
		}
		if got.Kind != ErrorKindSelector {
			t.Fatalf("AssertExists(unsupported selector) kind = %q, want %q", got.Kind, ErrorKindSelector)
		}
	}

	if err := harness.AssertExists("#missing"); err == nil {
		t.Fatalf("AssertExists(#missing) error = nil, want assertion error")
	} else {
		got, ok := err.(Error)
		if !ok {
			t.Fatalf("AssertExists(#missing) type = %T, want browsertester.Error", err)
		}
		if got.Kind != ErrorKindAssertion {
			t.Fatalf("AssertExists(#missing) kind = %q, want %q", got.Kind, ErrorKindAssertion)
		}
		if !strings.Contains(got.Message, "DOM:\n") {
			t.Fatalf("AssertExists(#missing) message = %q, want DOM dump", got.Message)
		}
	}

	if err := harness.AssertChecked("#out", true); err == nil {
		t.Fatalf("AssertChecked(#out) error = nil, want assertion error")
	} else {
		got, ok := err.(Error)
		if !ok {
			t.Fatalf("AssertChecked(#out) type = %T, want browsertester.Error", err)
		}
		if got.Kind != ErrorKindAssertion {
			t.Fatalf("AssertChecked(#out) kind = %q, want %q", got.Kind, ErrorKindAssertion)
		}
	}
}
