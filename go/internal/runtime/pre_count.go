package runtime

func (s *Session) PreCount() int {
	return s.elementCountForSelector("pre")
}
