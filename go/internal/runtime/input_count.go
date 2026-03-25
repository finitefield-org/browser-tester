package runtime

func (s *Session) InputCount() int {
	return s.inputCountForSelector("input")
}

func (s *Session) inputCountForSelector(selector string) int {
	if s == nil {
		return 0
	}
	store, err := s.ensureDOM()
	if err != nil || store == nil {
		return 0
	}
	nodes, err := store.QuerySelectorAll(selector)
	if err != nil {
		return 0
	}
	return nodes.Length()
}
