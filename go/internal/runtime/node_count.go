package runtime

func (s *Session) NodeCount() int {
	if s == nil {
		return 0
	}
	store, err := s.ensureDOM()
	if err != nil || store == nil {
		return 0
	}
	return store.NodeCount()
}
