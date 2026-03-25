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

func TestFetchFamilyUsesLastWriteWinsForDuplicateRules(t *testing.T) {
	var f FetchFamily

	f.RespondText("https://example.test/a", 200, "first")
	f.RespondText("https://example.test/a", 201, "second")

	status, body, err := f.Resolve("https://example.test/a")
	if err != nil {
		t.Fatalf("Resolve() error = %v", err)
	}
	if status != 201 || body != "second" {
		t.Fatalf("Resolve() = (%d, %q), want (201, %q)", status, body, "second")
	}

	f.Reset()
	f.Fail("https://example.test/a", "first failure")
	f.Fail("https://example.test/a", "second failure")
	if _, _, err := f.Resolve("https://example.test/a"); err == nil || err.Error() != "second failure" {
		t.Fatalf("Resolve() error = %v, want second failure", err)
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

func TestDialogFamilyTakeSnapshotsClearState(t *testing.T) {
	var f DialogFamily

	f.RecordAlert("alert")
	f.RecordConfirm("confirm?")
	f.RecordPrompt("prompt?")

	alerts := f.TakeAlerts()
	if len(alerts) != 1 || alerts[0] != "alert" {
		t.Fatalf("TakeAlerts() = %#v, want [\"alert\"]", alerts)
	}
	alerts[0] = "mutated"
	if got := f.TakeAlerts(); len(got) != 0 {
		t.Fatalf("TakeAlerts() second read = %#v, want empty", got)
	}

	confirms := f.TakeConfirmMessages()
	if len(confirms) != 1 || confirms[0] != "confirm?" {
		t.Fatalf("TakeConfirmMessages() = %#v, want [\"confirm?\"]", confirms)
	}
	confirms[0] = "mutated"
	if got := f.TakeConfirmMessages(); len(got) != 0 {
		t.Fatalf("TakeConfirmMessages() second read = %#v, want empty", got)
	}

	prompts := f.TakePromptMessages()
	if len(prompts) != 1 || prompts[0] != "prompt?" {
		t.Fatalf("TakePromptMessages() = %#v, want [\"prompt?\"]", prompts)
	}
	prompts[0] = "mutated"
	if got := f.TakePromptMessages(); len(got) != 0 {
		t.Fatalf("TakePromptMessages() second read = %#v, want empty", got)
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

func TestOpenClosePrintScrollTakeCallsClearState(t *testing.T) {
	var open OpenFamily
	if err := open.Invoke("https://example.test/new"); err != nil {
		t.Fatalf("Open Invoke() error = %v", err)
	}
	openCalls := open.TakeCalls()
	if len(openCalls) != 1 || openCalls[0].URL != "https://example.test/new" {
		t.Fatalf("Open TakeCalls() = %#v, want one call", openCalls)
	}
	openCalls[0].URL = "mutated"
	if got := open.TakeCalls(); len(got) != 0 {
		t.Fatalf("Open TakeCalls() second read = %#v, want empty", got)
	}

	var close CloseFamily
	if err := close.Invoke(); err != nil {
		t.Fatalf("Close Invoke() error = %v", err)
	}
	closeCalls := close.TakeCalls()
	if len(closeCalls) != 1 {
		t.Fatalf("Close TakeCalls() = %#v, want one call", closeCalls)
	}
	if got := close.TakeCalls(); len(got) != 0 {
		t.Fatalf("Close TakeCalls() second read = %#v, want empty", got)
	}

	var print PrintFamily
	if err := print.Invoke(); err != nil {
		t.Fatalf("Print Invoke() error = %v", err)
	}
	printCalls := print.Take()
	if len(printCalls) != 1 {
		t.Fatalf("Print Take() = %#v, want one call", printCalls)
	}
	if got := print.Take(); len(got) != 0 {
		t.Fatalf("Print Take() second read = %#v, want empty", got)
	}

	var scroll ScrollFamily
	if err := scroll.Invoke("by", 3, 4); err != nil {
		t.Fatalf("Scroll Invoke() error = %v", err)
	}
	scrollCalls := scroll.TakeCalls()
	if len(scrollCalls) != 1 || scrollCalls[0].Method != "by" || scrollCalls[0].X != 3 || scrollCalls[0].Y != 4 {
		t.Fatalf("Scroll TakeCalls() = %#v, want one by-call", scrollCalls)
	}
	scrollCalls[0].Method = "mutated"
	if got := scroll.TakeCalls(); len(got) != 0 {
		t.Fatalf("Scroll TakeCalls() second read = %#v, want empty", got)
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

	rules := f.Rules()
	if len(rules) != 1 || rules[0].Query != "(prefers-reduced-motion: reduce)" || !rules[0].Matches {
		t.Fatalf("Rules() = %#v, want one seeded rule", rules)
	}
	rules[0].Query = "mutated"
	if got := f.Rules(); len(got) != 1 || got[0].Query != "(prefers-reduced-motion: reduce)" || !got[0].Matches {
		t.Fatalf("Rules() reread = %#v, want original rule", got)
	}

	if _, err := f.Resolve("(prefers-color-scheme: dark)"); err == nil {
		t.Fatalf("Resolve() for unknown query error = nil, want missing-rule error")
	}
}

func TestMatchMediaListenerCallsReturnCopies(t *testing.T) {
	var f MatchMediaFamily

	f.RecordListenerCall("(prefers-reduced-motion: reduce)", "addListener")
	f.RecordListenerCall("(prefers-reduced-motion: reduce)", "removeListener")

	listeners := f.ListenerCalls()
	if len(listeners) != 2 || listeners[0].Method != "addListener" || listeners[1].Method != "removeListener" {
		t.Fatalf("ListenerCalls() = %#v, want both listener calls", listeners)
	}

	listeners[0].Method = "mutated"
	listeners[1].Query = "mutated"

	fresh := f.ListenerCalls()
	if len(fresh) != 2 || fresh[0].Method != "addListener" || fresh[1].Method != "removeListener" || fresh[0].Query != "(prefers-reduced-motion: reduce)" || fresh[1].Query != "(prefers-reduced-motion: reduce)" {
		t.Fatalf("ListenerCalls() reread = %#v, want original listener calls", fresh)
	}

	taken := f.TakeListenerCalls()
	if len(taken) != 2 || taken[0].Method != "addListener" || taken[1].Method != "removeListener" {
		t.Fatalf("TakeListenerCalls() = %#v, want both listener calls", taken)
	}
	taken[0].Method = "mutated"
	if got := f.TakeListenerCalls(); len(got) != 0 {
		t.Fatalf("TakeListenerCalls() second read = %#v, want empty", got)
	}
}

func TestMatchMediaUsesLastWriteWinsForDuplicateRules(t *testing.T) {
	var f MatchMediaFamily

	f.RespondMatches("(prefers-color-scheme: dark)", false)
	f.RespondMatches("(prefers-color-scheme: dark)", true)

	matches, err := f.Resolve("(prefers-color-scheme: dark)")
	if err != nil {
		t.Fatalf("Resolve() error = %v", err)
	}
	if !matches {
		t.Fatalf("Resolve() = false, want true")
	}
}

func TestMatchMediaTakeSnapshotsClearState(t *testing.T) {
	var f MatchMediaFamily

	f.RecordCall("(prefers-reduced-motion: reduce)")
	f.RecordListenerCall("(prefers-reduced-motion: reduce)", "addListener")

	calls := f.TakeCalls()
	if len(calls) != 1 || calls[0].Query != "(prefers-reduced-motion: reduce)" {
		t.Fatalf("TakeCalls() = %#v, want one query call", calls)
	}
	calls[0].Query = "mutated"
	if got := f.TakeCalls(); len(got) != 0 {
		t.Fatalf("TakeCalls() second read = %#v, want empty", got)
	}

	listeners := f.TakeListenerCalls()
	if len(listeners) != 1 || listeners[0].Method != "addListener" {
		t.Fatalf("TakeListenerCalls() = %#v, want one listener call", listeners)
	}
	listeners[0].Method = "mutated"
	if got := f.TakeListenerCalls(); len(got) != 0 {
		t.Fatalf("TakeListenerCalls() second read = %#v, want empty", got)
	}
}

func TestLocationTakeNavigationsClearsState(t *testing.T) {
	var f LocationFamily

	f.RecordNavigation("https://example.test/a")
	f.RecordNavigation("https://example.test/b")

	navigations := f.TakeNavigations()
	if len(navigations) != 2 || navigations[0] != "https://example.test/a" || navigations[1] != "https://example.test/b" {
		t.Fatalf("TakeNavigations() = %#v, want both navigations", navigations)
	}
	navigations[0] = "mutated"
	if got := f.TakeNavigations(); len(got) != 0 {
		t.Fatalf("TakeNavigations() second read = %#v, want empty", got)
	}
}

func TestDownloadFamilyTakeReturnsDeepCopyAndClearsState(t *testing.T) {
	var f DownloadFamily

	bytes := []byte("abc")
	f.Capture("a.txt", bytes)
	bytes[0] = 'z'

	artifacts := f.Take()
	if len(artifacts) != 1 || artifacts[0].FileName != "a.txt" || string(artifacts[0].Bytes) != "abc" {
		t.Fatalf("Take() = %#v, want preserved artifact bytes", artifacts)
	}
	artifacts[0].Bytes[0] = 'y'
	if got := f.Take(); len(got) != 0 {
		t.Fatalf("Take() second read = %#v, want empty", got)
	}
}

func TestFileInputFamilyTakeSelectionsReturnsDeepCopyAndClearsState(t *testing.T) {
	var f FileInputFamily

	files := []string{"a.txt", "b.txt"}
	f.SetFiles("#upload", files)
	files[0] = "mutated.txt"

	selections := f.TakeSelections()
	if len(selections) != 1 || selections[0].Selector != "#upload" {
		t.Fatalf("TakeSelections() = %#v, want one selection", selections)
	}
	if len(selections[0].Files) != 2 || selections[0].Files[0] != "a.txt" || selections[0].Files[1] != "b.txt" {
		t.Fatalf("TakeSelections() files = %#v, want preserved file list", selections[0].Files)
	}
	selections[0].Files[0] = "returned-mutation.txt"
	if got := f.TakeSelections(); len(got) != 0 {
		t.Fatalf("TakeSelections() second read = %#v, want empty", got)
	}
}

func TestStorageFamilyLocalAndSessionReturnCopies(t *testing.T) {
	var f StorageFamily

	f.SeedLocal("token", "abc")
	f.SeedSession("tab", "main")
	f.SeedLocal("theme", "dark")

	local := f.Local()
	session := f.Session()
	local["token"] = "mutated"
	local["extra"] = "new"
	session["tab"] = "mutated"
	session["extra"] = "new"

	events := f.Events()
	if len(events) != 3 || events[0].Scope != "local" || events[0].Key != "token" || events[0].Value != "abc" || events[1].Scope != "session" || events[1].Key != "tab" || events[1].Value != "main" || events[2].Scope != "local" || events[2].Key != "theme" || events[2].Value != "dark" {
		t.Fatalf("Events() = %#v, want three storage events", events)
	}
	events[0].Value = "mutated"
	if got := f.Events(); len(got) != 3 || got[0].Value != "abc" || got[2].Value != "dark" {
		t.Fatalf("Events() reread = %#v, want original events", got)
	}

	freshLocal := f.Local()
	if got, want := freshLocal["token"], "abc"; got != want {
		t.Fatalf("Local()[token] = %q, want %q", got, want)
	}
	if _, ok := freshLocal["extra"]; ok {
		t.Fatalf("Local()[extra] should not exist")
	}

	freshSession := f.Session()
	if got, want := freshSession["tab"], "main"; got != want {
		t.Fatalf("Session()[tab] = %q, want %q", got, want)
	}
	if _, ok := freshSession["extra"]; ok {
		t.Fatalf("Session()[extra] should not exist")
	}

	f.Reset()

	if got := f.Local(); len(got) != 0 {
		t.Fatalf("Local() after Reset = %#v, want empty", got)
	}
	if got := f.Session(); len(got) != 0 {
		t.Fatalf("Session() after Reset = %#v, want empty", got)
	}
}

func TestStorageFamilySupportsWebStorageOperations(t *testing.T) {
	var f StorageFamily

	f.SeedLocal("theme", "dark")
	f.SeedSession("tab", "main")

	if got, ok := f.Get("local", "theme"); !ok || got != "dark" {
		t.Fatalf("Get(local, theme) = (%q, %v), want (dark, true)", got, ok)
	}
	if got, ok := f.Get("session", "tab"); !ok || got != "main" {
		t.Fatalf("Get(session, tab) = (%q, %v), want (main, true)", got, ok)
	}
	if got, ok := f.Length("local"); !ok || got != 1 {
		t.Fatalf("Length(local) = (%d, %v), want (1, true)", got, ok)
	}
	if got, ok := f.Key("local", 0); !ok || got != "theme" {
		t.Fatalf("Key(local, 0) = (%q, %v), want (theme, true)", got, ok)
	}

	if !f.Set("local", "theme", "dark") {
		t.Fatalf("Set(local, theme, dark) = false, want true")
	}
	if !f.Set("local", "accent", "blue") {
		t.Fatalf("Set(local, accent, blue) = false, want true")
	}
	if !f.Remove("session", "tab") {
		t.Fatalf("Remove(session, tab) = false, want true")
	}
	if !f.Clear("local") {
		t.Fatalf("Clear(local) = false, want true")
	}

	if got := f.Local(); len(got) != 0 {
		t.Fatalf("Local() after clear = %#v, want empty", got)
	}
	if got := f.Session(); len(got) != 0 {
		t.Fatalf("Session() after remove = %#v, want empty", got)
	}

	events := f.Events()
	if len(events) != 5 {
		t.Fatalf("Events() = %#v, want five storage events", events)
	}
	if events[0].Scope != "local" || events[0].Op != "seed" || events[0].Key != "theme" || events[0].Value != "dark" {
		t.Fatalf("Events()[0] = %#v, want local seed event", events[0])
	}
	if events[1].Scope != "session" || events[1].Op != "seed" || events[1].Key != "tab" || events[1].Value != "main" {
		t.Fatalf("Events()[1] = %#v, want session seed event", events[1])
	}
	if events[2].Op != "set" || events[2].Key != "accent" || events[2].Value != "blue" {
		t.Fatalf("Events()[2] = %#v, want local set event", events[2])
	}
	if events[3].Op != "remove" || events[3].Scope != "session" || events[3].Key != "tab" {
		t.Fatalf("Events()[3] = %#v, want session remove event", events[3])
	}
	if events[4].Op != "clear" || events[4].Scope != "local" {
		t.Fatalf("Events()[4] = %#v, want local clear event", events[4])
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
