package runtime

func (s *Session) OptgroupCount() int {
	return s.optgroupCountForSelector("optgroup")
}

func (s *Session) optgroupCountForSelector(selector string) int {
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
