package runtime

import "testing"

func TestSessionAppliesConfigSeedsDeterministically(t *testing.T) {
	local := map[string]string{"token": "abc"}
	sessionStorage := map[string]string{"tab": "main"}
	match := map[string]bool{"(prefers-reduced-motion: reduce)": true}
	cfg := SessionConfig{
		URL:            "https://example.test/",
		LocalStorage:   local,
		SessionStorage: sessionStorage,
		MatchMedia:     match,
		OpenFailure:    "open blocked",
		CloseFailure:   "close blocked",
		PrintFailure:   "print blocked",
		ScrollFailure:  "scroll blocked",
	}

	s := NewSession(cfg)

	// Mutate source maps after NewSession to ensure config cloning is effective.
	local["token"] = "mutated"
	sessionStorage["tab"] = "mutated"
	match["(prefers-reduced-motion: reduce)"] = false

	if got, want := s.URL(), "https://example.test/"; got != want {
		t.Fatalf("URL() = %q, want %q", got, want)
	}

	if got, want := s.Registry().Storage().Local()["token"], "abc"; got != want {
		t.Fatalf("Storage().Local()[token] = %q, want %q", got, want)
	}
	if got, want := s.Registry().Storage().Session()["tab"], "main"; got != want {
		t.Fatalf("Storage().Session()[tab] = %q, want %q", got, want)
	}

	matches, err := s.MatchMedia("(prefers-reduced-motion: reduce)")
	if err != nil {
		t.Fatalf("MatchMedia() error = %v", err)
	}
	if !matches {
		t.Fatalf("MatchMedia() = false, want true")
	}

	if err := s.Open("https://example.test/new"); err == nil {
		t.Fatalf("Open() error = nil, want seeded failure")
	}
	if err := s.Close(); err == nil {
		t.Fatalf("Close() error = nil, want seeded failure")
	}
	if err := s.Print(); err == nil {
		t.Fatalf("Print() error = nil, want seeded failure")
	}
	if err := s.ScrollTo(1, 2); err == nil {
		t.Fatalf("ScrollTo() error = nil, want seeded failure")
	}
}

func TestSessionSchedulerBackedTime(t *testing.T) {
	s := NewSession(DefaultSessionConfig())

	if got, want := s.NowMs(), int64(0); got != want {
		t.Fatalf("NowMs() = %d, want %d", got, want)
	}

	if err := s.AdvanceTime(25); err != nil {
		t.Fatalf("AdvanceTime() error = %v", err)
	}
	if got, want := s.NowMs(), int64(25); got != want {
		t.Fatalf("NowMs() after AdvanceTime = %d, want %d", got, want)
	}

	s.Scheduler().Advance(10)
	if got, want := s.NowMs(), int64(35); got != want {
		t.Fatalf("NowMs() after Scheduler().Advance = %d, want %d", got, want)
	}

	s.SetNowMs(7)
	if got, want := s.NowMs(), int64(7); got != want {
		t.Fatalf("NowMs() after SetNowMs = %d, want %d", got, want)
	}

	s.ResetTime()
	if got, want := s.NowMs(), int64(0); got != want {
		t.Fatalf("NowMs() after ResetTime = %d, want %d", got, want)
	}

	if err := s.AdvanceTime(-1); err == nil {
		t.Fatalf("AdvanceTime(-1) error = nil, want validation error")
	}
	if got, want := s.NowMs(), int64(0); got != want {
		t.Fatalf("NowMs() after rejected negative advance = %d, want %d", got, want)
	}
}

func TestSessionNavigateResetsScrollState(t *testing.T) {
	s := NewSession(DefaultSessionConfig())

	if err := s.ScrollTo(10, 20); err != nil {
		t.Fatalf("ScrollTo() error = %v", err)
	}
	if err := s.ScrollBy(3, -4); err != nil {
		t.Fatalf("ScrollBy() error = %v", err)
	}
	if gotX, gotY := s.scrollX, s.scrollY; gotX != 13 || gotY != 16 {
		t.Fatalf("scroll state = (%d, %d), want (13, 16)", gotX, gotY)
	}

	if err := s.Navigate("https://example.test/next"); err != nil {
		t.Fatalf("Navigate() error = %v", err)
	}
	if got, want := s.URL(), "https://example.test/next"; got != want {
		t.Fatalf("URL() after Navigate = %q, want %q", got, want)
	}
	if gotX, gotY := s.scrollX, s.scrollY; gotX != 0 || gotY != 0 {
		t.Fatalf("scroll state after Navigate = (%d, %d), want (0, 0)", gotX, gotY)
	}
}
