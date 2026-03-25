package runtime

func (s *Session) TemplateCount() int {
	return s.templateCountForSelector("template")
}

func (s *Session) templateCountForSelector(selector string) int {
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
