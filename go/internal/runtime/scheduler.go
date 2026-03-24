package runtime

import "math"

type Scheduler struct {
	nowMs int64
}

func (s *Scheduler) NowMs() int64 {
	if s == nil {
		return 0
	}
	return s.nowMs
}

func (s *Scheduler) Advance(deltaMs int64) {
	if s == nil {
		return
	}
	if deltaMs > 0 && s.nowMs > math.MaxInt64-deltaMs {
		s.nowMs = math.MaxInt64
		return
	}
	if deltaMs < 0 && s.nowMs < math.MinInt64-deltaMs {
		s.nowMs = math.MinInt64
		return
	}
	s.nowMs += deltaMs
}

func (s *Scheduler) SetNow(ms int64) {
	if s == nil {
		return
	}
	s.nowMs = ms
}

func (s *Scheduler) Reset() {
	if s == nil {
		return
	}
	s.nowMs = 0
}
