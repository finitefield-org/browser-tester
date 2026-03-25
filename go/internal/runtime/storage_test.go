package runtime

import "testing"

func TestSessionWebStorageAccessors(t *testing.T) {
	s := NewSession(SessionConfig{
		LocalStorage:   map[string]string{"theme": "dark"},
		SessionStorage: map[string]string{"tab": "main"},
	})

	if got, ok := s.localStorageGetItem("theme"); !ok || got != "dark" {
		t.Fatalf("localStorageGetItem(theme) = (%q, %v), want (dark, true)", got, ok)
	}
	if got, ok := s.sessionStorageGetItem("tab"); !ok || got != "main" {
		t.Fatalf("sessionStorageGetItem(tab) = (%q, %v), want (main, true)", got, ok)
	}
	if got := s.localStorageLength(); got != 1 {
		t.Fatalf("localStorageLength() = %d, want 1", got)
	}
	if got := s.sessionStorageLength(); got != 1 {
		t.Fatalf("sessionStorageLength() = %d, want 1", got)
	}
	if got, ok := s.localStorageKey(0); !ok || got != "theme" {
		t.Fatalf("localStorageKey(0) = (%q, %v), want (theme, true)", got, ok)
	}
	if got, ok := s.sessionStorageKey(0); !ok || got != "tab" {
		t.Fatalf("sessionStorageKey(0) = (%q, %v), want (tab, true)", got, ok)
	}

	if err := s.localStorageSetItem("accent", "blue"); err != nil {
		t.Fatalf("localStorageSetItem(accent, blue) error = %v", err)
	}
	if err := s.localStorageSetItem("theme", "dark"); err != nil {
		t.Fatalf("localStorageSetItem(theme, dark) error = %v", err)
	}
	if err := s.sessionStorageRemoveItem("tab"); err != nil {
		t.Fatalf("sessionStorageRemoveItem(tab) error = %v", err)
	}
	if err := s.localStorageClear(); err != nil {
		t.Fatalf("localStorageClear() error = %v", err)
	}

	if got := s.Registry().Storage().Local(); len(got) != 0 {
		t.Fatalf("Storage().Local() after clear = %#v, want empty", got)
	}
	if got := s.Registry().Storage().Session(); len(got) != 0 {
		t.Fatalf("Storage().Session() after remove = %#v, want empty", got)
	}

	events := s.Registry().Storage().Events()
	if len(events) != 5 {
		t.Fatalf("Storage().Events() = %#v, want five storage events", events)
	}
	if events[0].Op != "seed" || events[1].Op != "seed" || events[2].Op != "set" || events[3].Op != "remove" || events[4].Op != "clear" {
		t.Fatalf("Storage().Events() ops = %#v, want seed/seed/set/remove/clear", events)
	}
}

func TestSessionStorageSnapshotsReturnCopies(t *testing.T) {
	s := NewSession(SessionConfig{
		LocalStorage:   map[string]string{"theme": "dark"},
		SessionStorage: map[string]string{"tab": "main"},
	})

	local := s.LocalStorage()
	if got, want := local["theme"], "dark"; got != want {
		t.Fatalf("LocalStorage()[theme] = %q, want %q", got, want)
	}
	local["theme"] = "mutated"
	if got, want := s.LocalStorage()["theme"], "dark"; got != want {
		t.Fatalf("LocalStorage()[theme] after mutation = %q, want %q", got, want)
	}

	session := s.SessionStorage()
	if got, want := session["tab"], "main"; got != want {
		t.Fatalf("SessionStorage()[tab] = %q, want %q", got, want)
	}
	session["tab"] = "mutated"
	if got, want := s.SessionStorage()["tab"], "main"; got != want {
		t.Fatalf("SessionStorage()[tab] after mutation = %q, want %q", got, want)
	}

	var nilSession *Session
	if got := nilSession.LocalStorage(); got != nil {
		t.Fatalf("nil LocalStorage() = %#v, want nil", got)
	}
	if got := nilSession.SessionStorage(); got != nil {
		t.Fatalf("nil SessionStorage() = %#v, want nil", got)
	}
}

func TestSessionReportsStorageEvents(t *testing.T) {
	s := NewSession(SessionConfig{
		LocalStorage:   map[string]string{"theme": "dark"},
		SessionStorage: map[string]string{"tab": "main"},
	})

	if err := s.localStorageSetItem("accent", "blue"); err != nil {
		t.Fatalf("localStorageSetItem(accent, blue) error = %v", err)
	}
	if err := s.sessionStorageRemoveItem("tab"); err != nil {
		t.Fatalf("sessionStorageRemoveItem(tab) error = %v", err)
	}
	if err := s.localStorageClear(); err != nil {
		t.Fatalf("localStorageClear() error = %v", err)
	}

	events := s.StorageEvents()
	if len(events) != 5 {
		t.Fatalf("StorageEvents() = %#v, want five events", events)
	}
	if events[0].Scope != "local" || events[0].Op != "seed" || events[0].Key != "theme" || events[0].Value != "dark" {
		t.Fatalf("StorageEvents()[0] = %#v, want local seed", events[0])
	}
	if events[1].Scope != "session" || events[1].Op != "seed" || events[1].Key != "tab" || events[1].Value != "main" {
		t.Fatalf("StorageEvents()[1] = %#v, want session seed", events[1])
	}
	if events[2].Scope != "local" || events[2].Op != "set" || events[2].Key != "accent" || events[2].Value != "blue" {
		t.Fatalf("StorageEvents()[2] = %#v, want local set", events[2])
	}
	if events[3].Scope != "session" || events[3].Op != "remove" || events[3].Key != "tab" {
		t.Fatalf("StorageEvents()[3] = %#v, want session remove", events[3])
	}
	if events[4].Scope != "local" || events[4].Op != "clear" {
		t.Fatalf("StorageEvents()[4] = %#v, want local clear", events[4])
	}

	events[0].Value = "mutated"
	if fresh := s.StorageEvents(); len(fresh) != 5 || fresh[0].Value != "dark" {
		t.Fatalf("StorageEvents() reread = %#v, want original events", fresh)
	}
}

func TestSessionWriteHTMLRollsBackWebStorage(t *testing.T) {
	s := NewSession(SessionConfig{
		HTML: `<main><script>host:localStorageSetItem("accent", "blue"); host:localStorageClear(); host:sessionStorageRemoveItem("tab"); host:doesNotExist()</script></main>`,
		LocalStorage: map[string]string{
			"theme": "dark",
		},
		SessionStorage: map[string]string{
			"tab": "main",
		},
	})

	if err := s.WriteHTML(`<main><script>host:localStorageSetItem("accent", "blue"); host:localStorageClear(); host:sessionStorageRemoveItem("tab"); host:doesNotExist()</script></main>`); err == nil {
		t.Fatalf("WriteHTML() error = nil, want inline script failure")
	}

	if got, want := s.Registry().Storage().Local()["theme"], "dark"; got != want {
		t.Fatalf("Storage().Local()[theme] after failed WriteHTML = %q, want %q", got, want)
	}
	if got, want := s.Registry().Storage().Session()["tab"], "main"; got != want {
		t.Fatalf("Storage().Session()[tab] after failed WriteHTML = %q, want %q", got, want)
	}
	events := s.Registry().Storage().Events()
	if len(events) != 2 || events[0].Op != "seed" || events[1].Op != "seed" {
		t.Fatalf("Storage().Events() after failed WriteHTML = %#v, want only seed events", events)
	}
}
