package runtime

func (s *Session) AudioCount() int {
	return s.mediaCountForSelector("audio")
}

func (s *Session) VideoCount() int {
	return s.mediaCountForSelector("video")
}

func (s *Session) mediaCountForSelector(selector string) int {
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
