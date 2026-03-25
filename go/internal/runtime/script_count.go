package runtime

func (s *Session) ScriptCount() int {
	if s == nil {
		return 0
	}
	store, err := s.ensureDOM()
	if err != nil || store == nil {
		return 0
	}
	nodes, err := store.Scripts()
	if err != nil {
		return 0
	}
	return nodes.Length()
}
