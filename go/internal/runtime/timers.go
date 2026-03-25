package runtime

import (
	"fmt"
	"math"
	"sort"
	"strings"

	"browsertester/internal/dom"
)

type timerRecord struct {
	id         int64
	source     string
	dueAt      int64
	repeat     bool
	intervalMs int64
}

type animationFrameRecord struct {
	id     int64
	source string
}

type TimerSnapshot struct {
	ID         int64
	Source     string
	DueAtMs    int64
	Repeat     bool
	IntervalMs int64
}

type AnimationFrameSnapshot struct {
	ID     int64
	Source string
}

func (s *Session) scheduleTimeout(source string, timeoutMs int64) (int64, error) {
	return s.scheduleTimer(source, timeoutMs, false)
}

func (s *Session) scheduleInterval(source string, timeoutMs int64) (int64, error) {
	return s.scheduleTimer(source, timeoutMs, true)
}

func (s *Session) scheduleTimer(source string, timeoutMs int64, repeat bool) (int64, error) {
	if s == nil {
		return 0, fmt.Errorf("session is unavailable")
	}

	normalized := strings.TrimSpace(source)
	if normalized == "" {
		return 0, fmt.Errorf("timer source must not be empty")
	}
	if timeoutMs < 0 {
		timeoutMs = 0
	}

	if s.nextTimerID == math.MaxInt64 {
		return 0, fmt.Errorf("timer id space exhausted")
	}
	s.nextTimerID++
	id := s.nextTimerID

	if s.timers == nil {
		s.timers = make(map[int64]timerRecord)
	}
	s.timers[id] = timerRecord{
		id:         id,
		source:     normalized,
		dueAt:      saturatingAddInt64(s.scheduler.NowMs(), timeoutMs),
		repeat:     repeat,
		intervalMs: timeoutMs,
	}
	return id, nil
}

func (s *Session) clearTimeout(id int64) {
	if s == nil || id <= 0 || len(s.timers) == 0 {
		if s != nil && s.runningTimerID == id {
			s.runningTimerCancelled = true
		}
		return
	}
	delete(s.timers, id)
	if s.runningTimerID == id {
		s.runningTimerCancelled = true
	}
}

func (s *Session) clearInterval(id int64) {
	s.clearTimeout(id)
}

func (s *Session) requestAnimationFrame(source string) (int64, error) {
	if s == nil {
		return 0, fmt.Errorf("session is unavailable")
	}

	normalized := strings.TrimSpace(source)
	if normalized == "" {
		return 0, fmt.Errorf("animation frame source must not be empty")
	}
	if s.nextAnimationFrameID == math.MaxInt64 {
		return 0, fmt.Errorf("animation frame id space exhausted")
	}
	s.nextAnimationFrameID++
	id := s.nextAnimationFrameID
	if s.animationFrames == nil {
		s.animationFrames = make(map[int64]animationFrameRecord)
	}
	s.animationFrames[id] = animationFrameRecord{
		id:     id,
		source: normalized,
	}
	return id, nil
}

func (s *Session) cancelAnimationFrame(id int64) {
	if s == nil || id <= 0 || len(s.animationFrames) == 0 {
		return
	}
	delete(s.animationFrames, id)
}

func (s *Session) clearAnimationFrames() {
	if s == nil {
		return
	}
	s.animationFrames = nil
	s.nextAnimationFrameID = 0
}

func (s *Session) clearTimers() {
	if s == nil {
		return
	}
	s.timers = nil
	s.nextTimerID = 0
	s.runningTimerID = 0
	s.runningTimerCancelled = false
}

func cloneTimerMap(timers map[int64]timerRecord) map[int64]timerRecord {
	if len(timers) == 0 {
		return nil
	}
	out := make(map[int64]timerRecord, len(timers))
	for id, timer := range timers {
		out[id] = timer
	}
	return out
}

func cloneAnimationFrameMap(frames map[int64]animationFrameRecord) map[int64]animationFrameRecord {
	if len(frames) == 0 {
		return nil
	}
	out := make(map[int64]animationFrameRecord, len(frames))
	for id, frame := range frames {
		out[id] = frame
	}
	return out
}

func (s *Session) PendingTimers() []TimerSnapshot {
	if s == nil {
		return nil
	}
	if _, err := s.ensureDOM(); err != nil {
		return nil
	}
	if len(s.timers) == 0 {
		return nil
	}
	out := make([]TimerSnapshot, 0, len(s.timers))
	for _, timer := range s.timers {
		out = append(out, TimerSnapshot{
			ID:         timer.id,
			Source:     timer.source,
			DueAtMs:    timer.dueAt,
			Repeat:     timer.repeat,
			IntervalMs: timer.intervalMs,
		})
	}
	if len(out) < 2 {
		return out
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].DueAtMs != out[j].DueAtMs {
			return out[i].DueAtMs < out[j].DueAtMs
		}
		return out[i].ID < out[j].ID
	})
	return out
}

func (s *Session) PendingAnimationFrames() []AnimationFrameSnapshot {
	if s == nil {
		return nil
	}
	if _, err := s.ensureDOM(); err != nil {
		return nil
	}
	if len(s.animationFrames) == 0 {
		return nil
	}
	out := make([]AnimationFrameSnapshot, 0, len(s.animationFrames))
	for _, frame := range s.animationFrames {
		out = append(out, AnimationFrameSnapshot{
			ID:     frame.id,
			Source: frame.source,
		})
	}
	if len(out) < 2 {
		return out
	}
	sort.Slice(out, func(i, j int) bool {
		return out[i].ID < out[j].ID
	})
	return out
}

func (s *Session) settlePendingWork(store *dom.Store) (err error) {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if store == nil {
		return fmt.Errorf("dom store is unavailable")
	}
	defer func() {
		if err != nil {
			s.discardMicrotasks()
		}
	}()

	if err = s.drainMicrotasks(store); err != nil {
		return err
	}
	if s.domStore != nil && s.domStore != store {
		store = s.domStore
	}

	for _, timer := range s.dueTimers() {
		if err = s.runTimer(store, timer); err != nil {
			return err
		}
		if err = s.drainMicrotasks(store); err != nil {
			return err
		}
		if s.domStore != nil && s.domStore != store {
			return nil
		}
	}

	for _, frame := range s.pendingAnimationFrames() {
		if err = s.runAnimationFrame(store, frame); err != nil {
			return err
		}
		if err = s.drainMicrotasks(store); err != nil {
			return err
		}
		if s.domStore != nil && s.domStore != store {
			return nil
		}
	}
	return nil
}

func (s *Session) dueTimers() []timerRecord {
	if s == nil || len(s.timers) == 0 {
		return nil
	}

	now := s.scheduler.NowMs()
	out := make([]timerRecord, 0, len(s.timers))
	for _, timer := range s.timers {
		if timer.dueAt <= now {
			out = append(out, timer)
		}
	}
	if len(out) < 2 {
		return out
	}

	sort.Slice(out, func(i, j int) bool {
		if out[i].dueAt != out[j].dueAt {
			return out[i].dueAt < out[j].dueAt
		}
		return out[i].id < out[j].id
	})
	return out
}

func (s *Session) runTimer(store *dom.Store, timer timerRecord) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if store == nil {
		return fmt.Errorf("dom store is unavailable")
	}
	if len(s.timers) == 0 {
		return nil
	}

	current, ok := s.timers[timer.id]
	if !ok {
		return nil
	}
	delete(s.timers, timer.id)
	prevRunningID := s.runningTimerID
	prevRunningCancelled := s.runningTimerCancelled
	s.runningTimerID = current.id
	s.runningTimerCancelled = false
	defer func() {
		s.runningTimerID = prevRunningID
		s.runningTimerCancelled = prevRunningCancelled
	}()

	if _, err := s.runScriptOnStore(store, current.source); err != nil {
		return err
	}
	if current.repeat && !s.runningTimerCancelled {
		if s.timers == nil {
			s.timers = make(map[int64]timerRecord)
		}
		s.timers[current.id] = timerRecord{
			id:         current.id,
			source:     current.source,
			dueAt:      saturatingAddInt64(s.scheduler.NowMs(), current.intervalMs),
			repeat:     true,
			intervalMs: current.intervalMs,
		}
	}
	return nil
}

func (s *Session) pendingAnimationFrames() []animationFrameRecord {
	if s == nil || len(s.animationFrames) == 0 {
		return nil
	}

	out := make([]animationFrameRecord, 0, len(s.animationFrames))
	for _, frame := range s.animationFrames {
		out = append(out, frame)
	}
	if len(out) < 2 {
		return out
	}

	sort.Slice(out, func(i, j int) bool {
		return out[i].id < out[j].id
	})
	return out
}

func (s *Session) runAnimationFrame(store *dom.Store, frame animationFrameRecord) error {
	if s == nil {
		return fmt.Errorf("session is unavailable")
	}
	if store == nil {
		return fmt.Errorf("dom store is unavailable")
	}
	if len(s.animationFrames) == 0 {
		return nil
	}

	current, ok := s.animationFrames[frame.id]
	if !ok {
		return nil
	}
	delete(s.animationFrames, frame.id)
	_, err := s.runScriptOnStore(store, current.source)
	return err
}

func saturatingAddInt64(base, delta int64) int64 {
	if delta > 0 && base > math.MaxInt64-delta {
		return math.MaxInt64
	}
	if delta < 0 && base < math.MinInt64-delta {
		return math.MinInt64
	}
	return base + delta
}
