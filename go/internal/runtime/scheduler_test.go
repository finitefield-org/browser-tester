package runtime

import (
	"math"
	"testing"
)

func TestSchedulerAdvanceSaturatesOnOverflow(t *testing.T) {
	var s Scheduler

	s.SetNow(math.MaxInt64 - 1)
	s.Advance(10)
	if got, want := s.NowMs(), int64(math.MaxInt64); got != want {
		t.Fatalf("NowMs() after positive overflow = %d, want %d", got, want)
	}

	s.SetNow(math.MinInt64 + 1)
	s.Advance(-10)
	if got, want := s.NowMs(), int64(math.MinInt64); got != want {
		t.Fatalf("NowMs() after negative overflow = %d, want %d", got, want)
	}
}

func TestSchedulerSetNowAndReset(t *testing.T) {
	var s Scheduler

	s.SetNow(42)
	if got, want := s.NowMs(), int64(42); got != want {
		t.Fatalf("NowMs() after SetNow = %d, want %d", got, want)
	}

	s.Advance(8)
	if got, want := s.NowMs(), int64(50); got != want {
		t.Fatalf("NowMs() after Advance = %d, want %d", got, want)
	}

	s.Reset()
	if got, want := s.NowMs(), int64(0); got != want {
		t.Fatalf("NowMs() after Reset = %d, want %d", got, want)
	}
}
