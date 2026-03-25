package runtime

func (s *Session) TargetNodeID() int64 {
	if s == nil {
		return 0
	}
	if s.domStore == nil {
		store, err := s.ensureDOM()
		if err != nil || store == nil {
			return 0
		}
		return int64(store.TargetNodeID())
	}
	return int64(s.domStore.TargetNodeID())
}

func (s *Session) HistoryLength() int {
	if s == nil {
		return 0
	}
	if _, err := s.ensureDOM(); err != nil {
		return 0
	}
	return s.windowHistoryLength()
}

func (s *Session) HistoryState() (string, bool) {
	if s == nil {
		return "null", false
	}
	if _, err := s.ensureDOM(); err != nil {
		return "null", false
	}
	return s.windowHistoryState()
}

func (s *Session) HistoryScrollRestoration() string {
	if s == nil {
		return "auto"
	}
	if _, err := s.ensureDOM(); err != nil {
		return "auto"
	}
	return s.windowHistoryScrollRestoration()
}
