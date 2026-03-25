package runtime

import (
	"math"
	"math/big"
	"testing"
)

func FuzzSchedulerAdvance(f *testing.F) {
	seeds := []struct {
		nowMs   int64
		deltaMs int64
	}{
		{0, 0},
		{0, 1},
		{5, 10},
		{10, -3},
		{math.MaxInt64 - 1, 1},
		{math.MinInt64 + 1, -1},
	}
	for _, seed := range seeds {
		f.Add(seed.nowMs, seed.deltaMs)
	}

	f.Fuzz(func(t *testing.T, nowMs int64, deltaMs int64) {
		var scheduler Scheduler
		scheduler.SetNow(nowMs)
		scheduler.Advance(deltaMs)

		if got, want := scheduler.NowMs(), expectedSaturatingAdd(nowMs, deltaMs); got != want {
			t.Fatalf("Scheduler.Advance(%d, %d) = %d, want %d", nowMs, deltaMs, got, want)
		}

		scheduler.Reset()
		if got := scheduler.NowMs(); got != 0 {
			t.Fatalf("Scheduler.Reset() = %d, want 0", got)
		}
	})
}

func FuzzScheduleTimerSeeds(f *testing.F) {
	seeds := []struct {
		nowMs     int64
		delayMs   int64
		repeating bool
	}{
		{0, 0, false},
		{0, 1, false},
		{5, 10, false},
		{10, -3, false},
		{math.MaxInt64 - 1, 1, false},
		{0, 5, true},
		{10, -3, true},
	}
	for _, seed := range seeds {
		f.Add(seed.nowMs, seed.delayMs, seed.repeating)
	}

	f.Fuzz(func(t *testing.T, nowMs int64, delayMs int64, repeating bool) {
		s := NewSession(DefaultSessionConfig())
		s.SetNowMs(nowMs)

		var (
			id  int64
			err error
		)
		if repeating {
			id, err = s.scheduleInterval("noop", delayMs)
		} else {
			id, err = s.scheduleTimeout("noop", delayMs)
		}
		if err != nil {
			t.Fatalf("scheduleTimer(repeating=%v, now=%d, delay=%d) error = %v", repeating, nowMs, delayMs, err)
		}

		record, ok := s.timers[id]
		if !ok {
			t.Fatalf("s.timers[%d] missing after scheduleTimer", id)
		}

		normalizedDelay := delayMs
		if normalizedDelay < 0 {
			normalizedDelay = 0
		}
		if got, want := record.dueAt, expectedSaturatingAdd(nowMs, normalizedDelay); got != want {
			t.Fatalf("timer dueAt = %d, want %d", got, want)
		}
		if got, want := record.repeat, repeating; got != want {
			t.Fatalf("timer repeat = %v, want %v", got, want)
		}
		if got, want := record.intervalMs, normalizedDelay; got != want {
			t.Fatalf("timer intervalMs = %d, want %d", got, want)
		}

		s.clearTimeout(id)
		if _, ok := s.timers[id]; ok {
			t.Fatalf("timer %d still present after clearTimeout", id)
		}
	})
}

func expectedSaturatingAdd(base, delta int64) int64 {
	var sum big.Int
	sum.SetInt64(base)
	sum.Add(&sum, big.NewInt(delta))

	if sum.Cmp(big.NewInt(math.MaxInt64)) > 0 {
		return math.MaxInt64
	}
	if sum.Cmp(big.NewInt(math.MinInt64)) < 0 {
		return math.MinInt64
	}
	return sum.Int64()
}
