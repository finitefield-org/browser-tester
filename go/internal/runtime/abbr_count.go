package runtime

func (s *Session) AbbrCount() int {
	return s.elementCountForSelector("abbr")
}
