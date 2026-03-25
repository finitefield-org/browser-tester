package mocks

import (
	"bytes"
	"fmt"
	"reflect"
	"testing"
)

func FuzzRegistryResetAllClearsState(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte("registry"),
		[]byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		r := NewRegistry()

		for i, b := range data {
			seedRegistryState(r, i, b)
		}

		r.ResetAll()
		assertRegistryEmpty(t, r)

		r.ResetAll()
		assertRegistryEmpty(t, r)
	})
}

func FuzzRegistrySnapshotCopiesAreStable(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte("snapshot"),
		[]byte{0, 1, 2, 3, 4, 5, 6, 7, 8},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		r := NewRegistry()

		base := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8}
		for i, b := range base {
			seedRegistryState(r, i, b)
		}
		for i, b := range data {
			seedRegistryState(r, i+len(base), b)
		}

		assertRegistrySnapshotCopies(t, r)
	})
}

func FuzzFetchFamilyResolutionModel(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte("fetch"),
		[]byte{0, 1, 2, 3, 4, 5, 6, 7, 8},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		const url = "https://example.test/fetch"
		var family FetchFamily

		var wantStatus int
		var wantBody string
		var wantErr string
		var sawResponse bool
		var sawFailure bool
		for i, b := range data {
			status := 200 + int(b%50)
			body := fmt.Sprintf("body-%02x-%d", b, i)
			message := fmt.Sprintf("failure-%02x-%d", b, i)

			if b%2 == 0 {
				family.RespondText(url, status, body)
				wantStatus = status
				wantBody = body
				sawResponse = true
			} else {
				family.Fail(url, message)
				wantErr = message
				sawFailure = true
			}
		}

		status, body, err := family.Resolve(url)
		if sawFailure {
			if err == nil || err.Error() != wantErr {
				t.Fatalf("Resolve() error = %v, want %q", err, wantErr)
			}
			if status != 0 || body != "" {
				t.Fatalf("Resolve() = (%d, %q) on failure, want zero values", status, body)
			}
		} else if sawResponse {
			if err != nil {
				t.Fatalf("Resolve() error = %v, want nil", err)
			}
			if status != wantStatus || body != wantBody {
				t.Fatalf("Resolve() = (%d, %q), want (%d, %q)", status, body, wantStatus, wantBody)
			}
		} else if err == nil {
			t.Fatalf("Resolve() error = nil for unconfigured URL, want missing rule error")
		}

		calls := family.Calls()
		if len(calls) != 1 || calls[0].URL != url {
			t.Fatalf("Calls() = %#v, want one resolve call for %q", calls, url)
		}
	})
}

func FuzzDialogFamilyQueueOrder(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte("dialogs"),
		[]byte{0, 1, 2, 3, 4, 5},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		var family DialogFamily
		var wantConfirms []bool
		var wantPrompts []struct {
			value     string
			submitted bool
		}

		for i, b := range data {
			switch b % 4 {
			case 0:
				value := b%2 == 0
				family.QueueConfirm(value)
				wantConfirms = append(wantConfirms, value)
			case 1:
				value := fmt.Sprintf("prompt-%02x-%d", b, i)
				family.QueuePromptText(value)
				wantPrompts = append(wantPrompts, struct {
					value     string
					submitted bool
				}{value: value, submitted: true})
			case 2:
				family.QueuePromptCancel()
				wantPrompts = append(wantPrompts, struct {
					value     string
					submitted bool
				}{value: "", submitted: false})
			default:
				seeded := fmt.Sprintf("seed-%02x-%d", b, i)
				family.QueuePrompt(&seeded)
				wantPrompts = append(wantPrompts, struct {
					value     string
					submitted bool
				}{value: seeded, submitted: true})
			}
		}

		for i, want := range wantConfirms {
			got, ok := family.TakeConfirm()
			if !ok {
				t.Fatalf("TakeConfirm() #%d ok = false, want true", i)
			}
			if got != want {
				t.Fatalf("TakeConfirm() #%d = %v, want %v", i, got, want)
			}
		}
		if got, ok := family.TakeConfirm(); ok {
			t.Fatalf("TakeConfirm() after queue drained = (%v, %v), want empty", got, ok)
		}

		for i, want := range wantPrompts {
			got, submitted, ok := family.TakePrompt()
			if !ok {
				t.Fatalf("TakePrompt() #%d ok = false, want true", i)
			}
			if got != want.value || submitted != want.submitted {
				t.Fatalf("TakePrompt() #%d = (%q, %v), want (%q, %v)", i, got, submitted, want.value, want.submitted)
			}
		}
		if got, submitted, ok := family.TakePrompt(); ok {
			t.Fatalf("TakePrompt() after queue drained = (%q, %v, %v), want empty", got, submitted, ok)
		}
	})
}

func FuzzMatchMediaFamilyResolutionModel(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte("match-media"),
		[]byte{0, 1, 2, 3, 4, 5, 6, 7},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		const query = "(prefers-color-scheme: dark)"
		var family MatchMediaFamily

		var want bool
		var sawRule bool
		for i, b := range data {
			if b%3 == 0 {
				matches := b%2 == 0
				family.RespondMatches(query, matches)
				want = matches
				sawRule = true
				continue
			}
			family.RecordListenerCall(query, fmt.Sprintf("listener-%d", i))
		}

		matches, err := family.Resolve(query)
		if !sawRule {
			if err == nil {
				t.Fatalf("Resolve() error = nil for unconfigured query, want missing rule error")
			}
		} else {
			if err != nil {
				t.Fatalf("Resolve() error = %v, want nil", err)
			}
			if matches != want {
				t.Fatalf("Resolve() = %v, want %v", matches, want)
			}
		}

		calls := family.Calls()
		if len(calls) != 1 || calls[0].Query != query {
			t.Fatalf("Calls() = %#v, want one query call for %q", calls, query)
		}
		listeners := family.ListenerCalls()
		for _, call := range listeners {
			if call.Query != query {
				t.Fatalf("ListenerCalls() contained mismatched query %#v", call)
			}
		}
	})
}

func FuzzClipboardFamilyStateModel(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte("clipboard"),
		[]byte{0, 1, 2, 3, 4, 5, 6},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		var family ClipboardFamily
		var wantSeed string
		var haveSeed bool
		wantWrites := make([]string, 0)

		for i, b := range data {
			value := fmt.Sprintf("clip-%02x-%d", b, i)
			switch b % 3 {
			case 0:
				family.SeedText(value)
				wantSeed = value
				haveSeed = true
			case 1:
				family.RecordWrite(value)
				wantWrites = append(wantWrites, value)
				wantSeed = value
				haveSeed = true
			default:
				family.SeedText(value)
				wantSeed = value
				haveSeed = true
			}
		}

		if got := family.Writes(); !reflect.DeepEqual(got, wantWrites) {
			t.Fatalf("Writes() = %#v, want %#v", got, wantWrites)
		}
		if got, ok := family.SeededText(); ok != haveSeed || got != wantSeed {
			t.Fatalf("SeededText() = (%q, %v), want (%q, %v)", got, ok, wantSeed, haveSeed)
		}

		writes := family.Writes()
		if len(writes) > 0 {
			writes[0] = "mutated-write"
			if got := family.Writes(); reflect.DeepEqual(got, writes) {
				t.Fatalf("Writes() returned a live slice, want a copy")
			}
		}

		family.Reset()
		if got := family.Writes(); len(got) != 0 {
			t.Fatalf("Writes() after Reset = %#v, want empty", got)
		}
		if _, ok := family.SeededText(); ok {
			t.Fatalf("SeededText() after Reset should be empty")
		}
	})
}

func FuzzLocationFamilyStateModel(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte("location"),
		[]byte{0, 1, 2, 3, 4, 5, 6},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		var family LocationFamily
		var wantCurrent string
		var haveCurrent bool
		wantNavigations := make([]string, 0)

		for i, b := range data {
			value := fmt.Sprintf("https://example.test/%02x/%d", b, i)
			if b%2 == 0 {
				family.SetCurrentURL(value)
			} else {
				family.RecordNavigation(value)
				wantNavigations = append(wantNavigations, value)
			}
			wantCurrent = value
			haveCurrent = true
		}

		if got, ok := family.CurrentURL(); ok != haveCurrent || got != wantCurrent {
			t.Fatalf("CurrentURL() = (%q, %v), want (%q, %v)", got, ok, wantCurrent, haveCurrent)
		}
		if got := family.Navigations(); !reflect.DeepEqual(got, wantNavigations) {
			t.Fatalf("Navigations() = %#v, want %#v", got, wantNavigations)
		}

		navigations := family.TakeNavigations()
		if !reflect.DeepEqual(navigations, wantNavigations) {
			t.Fatalf("TakeNavigations() = %#v, want %#v", navigations, wantNavigations)
		}
		if got := family.TakeNavigations(); len(got) != 0 {
			t.Fatalf("TakeNavigations() second read = %#v, want empty", got)
		}

		family.Reset()
		if got := family.Navigations(); len(got) != 0 {
			t.Fatalf("Navigations() after Reset = %#v, want empty", got)
		}
		if _, ok := family.CurrentURL(); ok {
			t.Fatalf("CurrentURL() after Reset should be empty")
		}
	})
}

func FuzzStorageFamilyStateModel(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte("storage"),
		[]byte{0, 1, 2, 3, 4, 5, 6, 7},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		var family StorageFamily
		wantLocal := map[string]string{}
		wantSession := map[string]string{}

		for i, b := range data {
			key := fmt.Sprintf("k-%02x-%d", b, i%4)
			value := fmt.Sprintf("v-%02x-%d", b, i)
			switch b % 3 {
			case 0:
				family.SeedLocal(key, value)
				wantLocal[key] = value
			case 1:
				family.SeedSession(key, value)
				wantSession[key] = value
			default:
				family.SeedLocal(key, value)
				family.SeedSession(key, value)
				wantLocal[key] = value
				wantSession[key] = value
			}
		}

		if got := family.Local(); !reflect.DeepEqual(got, wantLocal) {
			t.Fatalf("Local() = %#v, want %#v", got, wantLocal)
		}
		if got := family.Session(); !reflect.DeepEqual(got, wantSession) {
			t.Fatalf("Session() = %#v, want %#v", got, wantSession)
		}

		local := family.Local()
		if len(local) > 0 {
			for key := range local {
				local[key] = "mutated-local"
				break
			}
			if got := family.Local(); reflect.DeepEqual(got, local) {
				t.Fatalf("Local() returned a live map, want a copy")
			}
		}

		session := family.Session()
		if len(session) > 0 {
			for key := range session {
				session[key] = "mutated-session"
				break
			}
			if got := family.Session(); reflect.DeepEqual(got, session) {
				t.Fatalf("Session() returned a live map, want a copy")
			}
		}

		family.Reset()
		if got := family.Local(); len(got) != 0 {
			t.Fatalf("Local() after Reset = %#v, want empty", got)
		}
		if got := family.Session(); len(got) != 0 {
			t.Fatalf("Session() after Reset = %#v, want empty", got)
		}
	})
}

func FuzzDownloadFamilyCaptureModel(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte("download"),
		[]byte{0, 1, 2, 3, 4, 5, 6},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		var family DownloadFamily
		want := make([]DownloadCapture, 0)

		for i, b := range data {
			fileName := fmt.Sprintf("download-%02x-%d.bin", b, i)
			payload := []byte(fmt.Sprintf("artifact-%02x-%d", b, i))
			copied := append([]byte(nil), payload...)

			family.Capture(fileName, payload)
			want = append(want, DownloadCapture{FileName: fileName, Bytes: copied})

			if len(payload) > 0 {
				payload[0] ^= 0xff
			}
		}

		artifacts := family.Artifacts()
		if !reflect.DeepEqual(artifacts, want) {
			t.Fatalf("Artifacts() = %#v, want %#v", artifacts, want)
		}
		if len(artifacts) > 0 {
			artifacts[0].FileName = "mutated-file"
			if len(artifacts[0].Bytes) > 0 {
				artifacts[0].Bytes[0] ^= 0xff
			}
			if got := family.Artifacts(); !reflect.DeepEqual(got, want) {
				t.Fatalf("Artifacts() reread after mutation = %#v, want %#v", got, want)
			}
		}

		taken := family.Take()
		if !reflect.DeepEqual(taken, want) {
			t.Fatalf("Take() = %#v, want %#v", taken, want)
		}
		if got := family.Take(); len(got) != 0 {
			t.Fatalf("Take() second read = %#v, want empty", got)
		}

		family.Reset()
		if got := family.Artifacts(); len(got) != 0 {
			t.Fatalf("Artifacts() after Reset = %#v, want empty", got)
		}
	})
}

func FuzzFileInputFamilyCaptureModel(f *testing.F) {
	seeds := [][]byte{
		nil,
		[]byte("file-input"),
		[]byte{0, 1, 2, 3, 4, 5, 6},
	}
	for _, seed := range seeds {
		f.Add(seed)
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		var family FileInputFamily
		want := make([]FileInputSelection, 0)

		for i, b := range data {
			selector := fmt.Sprintf("#upload-%02x-%d", b, i)
			files := []string{
				fmt.Sprintf("file-%02x-%d-a.txt", b, i),
				fmt.Sprintf("file-%02x-%d-b.txt", b, i),
			}
			copied := append([]string(nil), files...)

			family.SetFiles(selector, files)
			want = append(want, FileInputSelection{Selector: selector, Files: copied})

			files[0] = "mutated-input.txt"
		}

		selections := family.Selections()
		if !reflect.DeepEqual(selections, want) {
			t.Fatalf("Selections() = %#v, want %#v", selections, want)
		}
		if len(selections) > 0 {
			selections[0].Selector = "mutated-selector"
			if len(selections[0].Files) > 0 {
				selections[0].Files[0] = "mutated-file"
			}
			if got := family.Selections(); !reflect.DeepEqual(got, want) {
				t.Fatalf("Selections() reread after mutation = %#v, want %#v", got, want)
			}
		}

		taken := family.TakeSelections()
		if !reflect.DeepEqual(taken, want) {
			t.Fatalf("TakeSelections() = %#v, want %#v", taken, want)
		}
		if got := family.TakeSelections(); len(got) != 0 {
			t.Fatalf("TakeSelections() second read = %#v, want empty", got)
		}

		family.Reset()
		if got := family.Selections(); len(got) != 0 {
			t.Fatalf("Selections() after Reset = %#v, want empty", got)
		}
	})
}

func seedRegistryState(r *Registry, step int, b byte) {
	if r == nil {
		return
	}

	url := fmt.Sprintf("https://example.test/%02x/%d", b, step)
	key := fmt.Sprintf("key-%02x-%d", b, step)
	value := fmt.Sprintf("value-%02x-%d", b, step)
	query := fmt.Sprintf("(prefers-color-scheme: %02x)", b)
	fileName := fmt.Sprintf("file-%02x-%d.txt", b, step)
	selector := fmt.Sprintf("#input-%02x-%d", b, step)

	switch b % 9 {
	case 0:
		r.Fetch().RespondText(url, 200, value)
		r.Fetch().Fail(url, "fetch blocked")
		_, _, _ = r.Fetch().Resolve(url)
	case 1:
		r.Dialogs().QueueConfirm(b%2 == 0)
		r.Dialogs().QueuePromptText(value)
		r.Dialogs().QueuePromptCancel()
		r.Dialogs().RecordAlert(value)
		r.Dialogs().RecordConfirm(value)
		r.Dialogs().RecordPrompt(value)
		_, _, _ = r.Dialogs().TakePrompt()
	case 2:
		r.Clipboard().SeedText(value)
		r.Clipboard().RecordWrite(value + "-write")
	case 3:
		r.Location().SetCurrentURL(url)
		r.Location().RecordNavigation(url + "#next")
	case 4:
		r.Open().Fail("open blocked")
		_ = r.Open().Invoke(url)
		r.Close().Fail("close blocked")
		_ = r.Close().Invoke()
		r.Print().Fail("print blocked")
		_ = r.Print().Invoke()
		r.Scroll().Fail("scroll blocked")
		_ = r.Scroll().Invoke("to", int64(step), int64(b))
	case 5:
		r.MatchMedia().RespondMatches(query, b%2 == 0)
		r.MatchMedia().RecordCall(query)
		r.MatchMedia().RecordListenerCall(query, "change")
		_, _ = r.MatchMedia().Resolve(query)
	case 6:
		r.Downloads().Capture(fileName, []byte(value))
	case 7:
		r.FileInput().SetFiles(selector, []string{fileName, value})
	default:
		r.Storage().SeedLocal(key, value)
		r.Storage().SeedSession(key, value)
		_ = r.Storage().Local()
		_ = r.Storage().Session()
	}
}

func assertRegistrySnapshotCopies(t *testing.T, r *Registry) {
	t.Helper()

	assertSliceSnapshotCopy(t, "Fetch response rules", r.Fetch().ResponseRules(), r.Fetch().ResponseRules, func(items []FetchResponseRule) {
		items[0].Body = "mutated-body"
	})
	assertSliceSnapshotCopy(t, "Fetch error rules", r.Fetch().ErrorRules(), r.Fetch().ErrorRules, func(items []FetchErrorRule) {
		items[0].Message = "mutated-message"
	})
	assertSliceSnapshotCopy(t, "Fetch calls", r.Fetch().Calls(), r.Fetch().Calls, func(items []FetchCall) {
		items[0].URL = "mutated-url"
	})

	assertSliceSnapshotCopy(t, "Dialog alerts", r.Dialogs().Alerts(), r.Dialogs().Alerts, func(items []string) {
		items[0] = "mutated-alert"
	})
	assertSliceSnapshotCopy(t, "Dialog confirm messages", r.Dialogs().ConfirmMessages(), r.Dialogs().ConfirmMessages, func(items []string) {
		items[0] = "mutated-confirm"
	})
	assertSliceSnapshotCopy(t, "Dialog prompt messages", r.Dialogs().PromptMessages(), r.Dialogs().PromptMessages, func(items []string) {
		items[0] = "mutated-prompt"
	})

	assertSliceSnapshotCopy(t, "Clipboard writes", r.Clipboard().Writes(), r.Clipboard().Writes, func(items []string) {
		items[0] = "mutated-write"
	})

	assertSliceSnapshotCopy(t, "Location navigations", r.Location().Navigations(), r.Location().Navigations, func(items []string) {
		items[0] = "mutated-navigation"
	})

	assertSliceSnapshotCopy(t, "Open calls", r.Open().Calls(), r.Open().Calls, func(items []OpenCall) {
		items[0].URL = "mutated-open"
	})
	assertSliceSnapshotCopy(t, "Close calls", r.Close().Calls(), r.Close().Calls, func(items []CloseCall) {
		items = append(items, CloseCall{})
	})
	assertSliceSnapshotCopy(t, "Print calls", r.Print().Calls(), r.Print().Calls, func(items []PrintCall) {
		items = append(items, PrintCall{})
	})
	assertSliceSnapshotCopy(t, "Scroll calls", r.Scroll().Calls(), r.Scroll().Calls, func(items []ScrollCall) {
		items[0].Method = "mutated-scroll"
		items[0].X = -1
		items[0].Y = -1
	})

	assertSliceSnapshotCopy(t, "MatchMedia calls", r.MatchMedia().Calls(), r.MatchMedia().Calls, func(items []MatchMediaCall) {
		items[0].Query = "mutated-query"
	})
	assertSliceSnapshotCopy(t, "MatchMedia listener calls", r.MatchMedia().ListenerCalls(), r.MatchMedia().ListenerCalls, func(items []MatchMediaListenerCall) {
		items[0].Query = "mutated-listener-query"
		items[0].Method = "mutated-listener-method"
	})
	assertSliceSnapshotCopy(t, "Storage events", r.Storage().Events(), r.Storage().Events, func(items []StorageEvent) {
		items[0].Scope = "mutated-scope"
		items[0].Key = "mutated-key"
		items[0].Value = "mutated-value"
	})

	assertDownloadSnapshotCopy(t, r)
	assertFileInputSnapshotCopy(t, r)
	assertMapSnapshotCopy(t, "Storage local", r.Storage().Local(), r.Storage().Local)
	assertMapSnapshotCopy(t, "Storage session", r.Storage().Session(), r.Storage().Session)
}

func assertSliceSnapshotCopy[T any](t *testing.T, label string, snapshot []T, reread func() []T, mutate func([]T)) {
	t.Helper()
	if len(snapshot) == 0 {
		return
	}

	baseline := append([]T(nil), snapshot...)
	mutate(snapshot)
	if got := reread(); !reflect.DeepEqual(got, baseline) {
		t.Fatalf("%s reread after mutating returned snapshot = %#v, want %#v", label, got, baseline)
	}
}

func assertMapSnapshotCopy(t *testing.T, label string, snapshot map[string]string, reread func() map[string]string) {
	t.Helper()
	if len(snapshot) == 0 {
		return
	}

	baseline := make(map[string]string, len(snapshot))
	for key, value := range snapshot {
		baseline[key] = value
	}

	for key := range snapshot {
		snapshot[key] = "mutated-" + key
		break
	}

	if got := reread(); !reflect.DeepEqual(got, baseline) {
		t.Fatalf("%s reread after mutating returned map = %#v, want %#v", label, got, baseline)
	}
}

func assertDownloadSnapshotCopy(t *testing.T, r *Registry) {
	t.Helper()
	snapshot := r.Downloads().Artifacts()
	if len(snapshot) == 0 {
		return
	}

	baseline := make([]DownloadCapture, len(snapshot))
	for i := range snapshot {
		item := snapshot[i]
		bytesCopy := make([]byte, len(item.Bytes))
		copy(bytesCopy, item.Bytes)
		baseline[i] = DownloadCapture{FileName: item.FileName, Bytes: bytesCopy}
	}

	firstFileName := baseline[0].FileName
	firstBytes := append([]byte(nil), baseline[0].Bytes...)

	snapshot[0].FileName = "mutated-file"
	if len(snapshot[0].Bytes) > 0 {
		snapshot[0].Bytes[0] ^= 0xff
	}

	reread := r.Downloads().Artifacts()
	if len(reread) != len(baseline) {
		t.Fatalf("Download artifacts reread length = %d, want %d", len(reread), len(baseline))
	}
	if reread[0].FileName != firstFileName {
		t.Fatalf("Download artifact file name reread = %q, want %q", reread[0].FileName, firstFileName)
	}
	if !bytes.Equal(reread[0].Bytes, firstBytes) {
		t.Fatalf("Download artifact bytes reread = %q, want %q", string(reread[0].Bytes), string(firstBytes))
	}
}

func assertFileInputSnapshotCopy(t *testing.T, r *Registry) {
	t.Helper()
	snapshot := r.FileInput().Selections()
	if len(snapshot) == 0 {
		return
	}

	baseline := make([]FileInputSelection, len(snapshot))
	for i := range snapshot {
		item := snapshot[i]
		files := make([]string, len(item.Files))
		copy(files, item.Files)
		baseline[i] = FileInputSelection{Selector: item.Selector, Files: files}
	}

	firstSelector := baseline[0].Selector
	firstFiles := append([]string(nil), baseline[0].Files...)

	snapshot[0].Selector = "mutated-selector"
	if len(snapshot[0].Files) > 0 {
		snapshot[0].Files[0] = "mutated-file"
	}

	reread := r.FileInput().Selections()
	if len(reread) != len(baseline) {
		t.Fatalf("FileInput selections reread length = %d, want %d", len(reread), len(baseline))
	}
	if reread[0].Selector != firstSelector {
		t.Fatalf("FileInput selection selector reread = %q, want %q", reread[0].Selector, firstSelector)
	}
	if !reflect.DeepEqual(reread[0].Files, firstFiles) {
		t.Fatalf("FileInput selection files reread = %#v, want %#v", reread[0].Files, firstFiles)
	}
}

func assertRegistryEmpty(t *testing.T, r *Registry) {
	t.Helper()

	if got := r.Fetch().Calls(); len(got) != 0 {
		t.Fatalf("Fetch calls after ResetAll = %#v, want empty", got)
	}
	if got := r.Fetch().ResponseRules(); len(got) != 0 {
		t.Fatalf("Fetch response rules after ResetAll = %#v, want empty", got)
	}
	if got := r.Fetch().ErrorRules(); len(got) != 0 {
		t.Fatalf("Fetch error rules after ResetAll = %#v, want empty", got)
	}

	if got := r.Dialogs().Alerts(); len(got) != 0 {
		t.Fatalf("Dialog alerts after ResetAll = %#v, want empty", got)
	}
	if got := r.Dialogs().ConfirmMessages(); len(got) != 0 {
		t.Fatalf("Dialog confirm messages after ResetAll = %#v, want empty", got)
	}
	if got := r.Dialogs().PromptMessages(); len(got) != 0 {
		t.Fatalf("Dialog prompt messages after ResetAll = %#v, want empty", got)
	}
	if value, ok := r.Dialogs().TakeConfirm(); ok {
		t.Fatalf("TakeConfirm() after ResetAll = (%v, %v), want empty queue", value, ok)
	}
	if value, submitted, ok := r.Dialogs().TakePrompt(); ok {
		t.Fatalf("TakePrompt() after ResetAll = (%q, %v, %v), want empty queue", value, submitted, ok)
	}

	if got := r.Clipboard().Writes(); len(got) != 0 {
		t.Fatalf("Clipboard writes after ResetAll = %#v, want empty", got)
	}
	if _, ok := r.Clipboard().SeededText(); ok {
		t.Fatalf("Clipboard seeded text should be cleared after ResetAll")
	}

	if got := r.Location().Navigations(); len(got) != 0 {
		t.Fatalf("Location navigations after ResetAll = %#v, want empty", got)
	}
	if got, ok := r.Location().CurrentURL(); ok || got != "" {
		t.Fatalf("Location current URL after ResetAll = (%q, %v), want empty", got, ok)
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
	if got := r.MatchMedia().ListenerCalls(); len(got) != 0 {
		t.Fatalf("MatchMedia listener calls after ResetAll = %#v, want empty", got)
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
	if got := r.Storage().Events(); len(got) != 0 {
		t.Fatalf("Storage events after ResetAll = %#v, want empty", got)
	}
}
