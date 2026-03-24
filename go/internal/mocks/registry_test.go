package mocks

import "testing"

func TestFetchFamilyResolvesAndCapturesCalls(t *testing.T) {
	var f FetchFamily

	f.RespondText("https://example.test/a", 200, "ok")
	f.Fail("https://example.test/a", "boom")
	if _, _, err := f.Resolve("https://example.test/a"); err == nil {
		t.Fatalf("Resolve() error = nil, want failure rule precedence")
	}

	f.Reset()
	f.RespondText("https://example.test/a", 200, "ok")
	status, body, err := f.Resolve("https://example.test/a")
	if err != nil {
		t.Fatalf("Resolve() error = %v", err)
	}
	if status != 200 || body != "ok" {
		t.Fatalf("Resolve() = (%d, %q), want (200, %q)", status, body, "ok")
	}

	calls := f.TakeCalls()
	if len(calls) != 1 || calls[0].URL != "https://example.test/a" {
		t.Fatalf("TakeCalls() = %#v, want one call", calls)
	}
	if got := f.Calls(); len(got) != 0 {
		t.Fatalf("Calls() after TakeCalls() = %#v, want empty", got)
	}
}

func TestDialogFamilyQueuesAndCapturesMessages(t *testing.T) {
	var f DialogFamily

	f.QueueConfirm(true)
	f.QueuePromptText("typed")
	f.QueuePromptCancel()
	f.RecordAlert("alert")
	f.RecordConfirm("confirm?")
	f.RecordPrompt("prompt?")

	confirm, ok := f.TakeConfirm()
	if !ok || !confirm {
		t.Fatalf("TakeConfirm() = (%v, %v), want (true, true)", confirm, ok)
	}

	value, submitted, ok := f.TakePrompt()
	if !ok || !submitted || value != "typed" {
		t.Fatalf("TakePrompt() #1 = (%q, %v, %v), want (%q, true, true)", value, submitted, ok, "typed")
	}

	value, submitted, ok = f.TakePrompt()
	if !ok || submitted || value != "" {
		t.Fatalf("TakePrompt() #2 = (%q, %v, %v), want (\"\", false, true)", value, submitted, ok)
	}

	if got := f.TakeAlerts(); len(got) != 1 || got[0] != "alert" {
		t.Fatalf("TakeAlerts() = %#v, want [\"alert\"]", got)
	}
	if got := f.TakeConfirmMessages(); len(got) != 1 || got[0] != "confirm?" {
		t.Fatalf("TakeConfirmMessages() = %#v, want [\"confirm?\"]", got)
	}
	if got := f.TakePromptMessages(); len(got) != 1 || got[0] != "prompt?" {
		t.Fatalf("TakePromptMessages() = %#v, want [\"prompt?\"]", got)
	}
}

func TestOpenClosePrintScrollFailureAndCapture(t *testing.T) {
	var open OpenFamily
	open.Fail("open blocked")
	if err := open.Invoke("https://example.test/new"); err == nil {
		t.Fatalf("Open Invoke() error = nil, want failure")
	}
	if got := open.TakeCalls(); len(got) != 1 || got[0].URL != "https://example.test/new" {
		t.Fatalf("Open TakeCalls() = %#v, want one call", got)
	}

	var close CloseFamily
	close.Fail("close blocked")
	if err := close.Invoke(); err == nil {
		t.Fatalf("Close Invoke() error = nil, want failure")
	}
	if got := close.TakeCalls(); len(got) != 1 {
		t.Fatalf("Close TakeCalls() = %#v, want one call", got)
	}

	var print PrintFamily
	print.Fail("print blocked")
	if err := print.Invoke(); err == nil {
		t.Fatalf("Print Invoke() error = nil, want failure")
	}
	if got := print.Take(); len(got) != 1 {
		t.Fatalf("Print Take() = %#v, want one call", got)
	}

	var scroll ScrollFamily
	scroll.Fail("scroll blocked")
	if err := scroll.Invoke("to", 1, 2); err == nil {
		t.Fatalf("Scroll Invoke() error = nil, want failure")
	}
	if got := scroll.TakeCalls(); len(got) != 1 || got[0].Method != "to" || got[0].X != 1 || got[0].Y != 2 {
		t.Fatalf("Scroll TakeCalls() = %#v, want one to-call", got)
	}
}

func TestMatchMediaResolveAndTakeCalls(t *testing.T) {
	var f MatchMediaFamily

	f.RespondMatches("(prefers-reduced-motion: reduce)", true)
	matches, err := f.Resolve("(prefers-reduced-motion: reduce)")
	if err != nil {
		t.Fatalf("Resolve() error = %v", err)
	}
	if !matches {
		t.Fatalf("Resolve() = false, want true")
	}

	f.RecordListenerCall("(prefers-reduced-motion: reduce)", "addListener")

	if got := f.TakeCalls(); len(got) != 1 || got[0].Query != "(prefers-reduced-motion: reduce)" {
		t.Fatalf("TakeCalls() = %#v, want one query call", got)
	}
	if got := f.TakeListenerCalls(); len(got) != 1 || got[0].Method != "addListener" {
		t.Fatalf("TakeListenerCalls() = %#v, want one listener call", got)
	}

	if _, err := f.Resolve("(prefers-color-scheme: dark)"); err == nil {
		t.Fatalf("Resolve() for unknown query error = nil, want missing-rule error")
	}
}

func TestRegistryResetAllClearsAllFamilies(t *testing.T) {
	r := NewRegistry()

	r.Fetch().RespondText("https://example.test/a", 200, "ok")
	r.Dialogs().RecordAlert("alert")
	r.Clipboard().SeedText("seed")
	r.Location().RecordNavigation("https://example.test/n")
	r.Open().Fail("open blocked")
	r.Close().Fail("close blocked")
	r.Print().Fail("print blocked")
	r.Scroll().Fail("scroll blocked")
	r.MatchMedia().RespondMatches("(prefers-reduced-motion: reduce)", true)
	r.Downloads().Capture("a.txt", []byte("abc"))
	r.FileInput().SetFiles("#upload", []string{"a.txt"})
	r.Storage().SeedLocal("k", "v")
	r.Storage().SeedSession("s", "1")

	r.ResetAll()

	if got := r.Fetch().Calls(); len(got) != 0 {
		t.Fatalf("Fetch calls after ResetAll = %#v, want empty", got)
	}
	if got := r.Dialogs().Alerts(); len(got) != 0 {
		t.Fatalf("Dialog alerts after ResetAll = %#v, want empty", got)
	}
	if _, ok := r.Clipboard().SeededText(); ok {
		t.Fatalf("Clipboard seeded text should be cleared after ResetAll")
	}
	if got := r.Location().Navigations(); len(got) != 0 {
		t.Fatalf("Location navigations after ResetAll = %#v, want empty", got)
	}
	if got := r.Open().Calls(); len(got) != 0 {
		t.Fatalf("Open calls after ResetAll = %#v, want empty", got)
	}
	if got := r.Close().Calls(); len(got) != 0 {
		t.Fatalf("Close calls after ResetAll = %#v, want empty", got)
	}
	if got := r.Print().Calls(); len(got) != 0 {
		t.Fatalf("Print calls after ResetAll = %#v, want empty", got)
	}
	if got := r.Scroll().Calls(); len(got) != 0 {
		t.Fatalf("Scroll calls after ResetAll = %#v, want empty", got)
	}
	if got := r.MatchMedia().Calls(); len(got) != 0 {
		t.Fatalf("MatchMedia calls after ResetAll = %#v, want empty", got)
	}
	if got := r.Downloads().Artifacts(); len(got) != 0 {
		t.Fatalf("Download artifacts after ResetAll = %#v, want empty", got)
	}
	if got := r.FileInput().Selections(); len(got) != 0 {
		t.Fatalf("FileInput selections after ResetAll = %#v, want empty", got)
	}
	if got := r.Storage().Local(); len(got) != 0 {
		t.Fatalf("Storage local after ResetAll = %#v, want empty", got)
	}
	if got := r.Storage().Session(); len(got) != 0 {
		t.Fatalf("Storage session after ResetAll = %#v, want empty", got)
	}
}
