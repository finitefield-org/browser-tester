package runtime

func (s *Session) FieldsetCount() int {
	return s.fieldsetCountForSelector("fieldset")
}

func (s *Session) fieldsetCountForSelector(selector string) int {
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
