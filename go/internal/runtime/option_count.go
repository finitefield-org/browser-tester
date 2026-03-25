package runtime

func (s *Session) OptionCount() int {
	return s.optionCountForSelector("option")
}

func (s *Session) SelectedOptionCount() int {
	return s.optionCountForSelector("option[selected]")
}

func (s *Session) optionCountForSelector(selector string) int {
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
